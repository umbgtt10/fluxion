// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::Ordered;
use fluxion_stream::combine_latest::CombineLatestExt;
use fluxion_stream::CombinedState;
use fluxion_test_utils::{
    helpers::assert_no_element_emitted,
    sequenced::Sequenced,
    test_channel,
    test_data::{
        animal_dog, animal_spider, person_alice, person_bob, person_charlie, person_diane,
        plant_rose, plant_sunflower, DataVariant, TestData,
    },
    unwrap_value,
};
use futures::{Stream, StreamExt};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

fn send_variant(
    variant: &DataVariant,
    senders: &[mpsc::UnboundedSender<Sequenced<TestData>>],
) -> anyhow::Result<()> {
    match variant {
        DataVariant::Person => senders[0].send(Sequenced::new(person_alice()))?,
        DataVariant::Animal => senders[1].send(Sequenced::new(animal_dog()))?,
        DataVariant::Plant => senders[2].send(Sequenced::new(plant_rose()))?,
    }

    Ok(())
}

async fn expect_next_combined_equals<S, T>(stream: &mut S, expected: &[TestData])
where
    S: Stream<Item = fluxion_core::StreamItem<T>> + Unpin,
    T: Ordered<Inner = CombinedState<TestData>>,
{
    let item = stream.next().await.expect("expected next combined state");
    let state = item.unwrap();
    let actual: Vec<TestData> = state.get().values().clone();
    assert_eq!(actual, expected);
}

static FILTER: fn(&CombinedState<TestData>) -> bool = |_: &CombinedState<TestData>| true;

#[tokio::test]
async fn test_combine_latest_empty_streams() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_stream) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_stream) = test_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_stream) = test_channel::<Sequenced<TestData>>();

    let mut combined_stream =
        person_stream.combine_latest(vec![animal_stream, plant_stream], FILTER);

    // Act
    drop(person_tx);
    drop(animal_tx);
    drop(plant_tx);

    // Assert
    let next_item = combined_stream.next().await;
    assert!(
        next_item.is_none(),
        "Expected no items from an empty combined stream"
    );
    Ok(())
}

#[tokio::test]
async fn test_combine_latest_not_all_streams_have_published_does_not_emit() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_stream) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_stream) = test_channel::<Sequenced<TestData>>();
    let (_plant_tx, plant_stream) = test_channel::<Sequenced<TestData>>();

    let mut combined_stream =
        person_stream.combine_latest(vec![animal_stream, plant_stream], FILTER);

    // Act
    person_tx.send(Sequenced::new(person_alice()))?;

    // Assert
    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Act
    animal_tx.send(Sequenced::new(animal_dog()))?;

    // Assert
    assert_no_element_emitted(&mut combined_stream, 100).await;
    Ok(())
}

#[tokio::test]
async fn test_combine_latest_stream_closes_before_publish_no_output() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_stream) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_stream) = test_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_stream) = test_channel::<Sequenced<TestData>>();

    let mut combined_stream =
        person_stream.combine_latest(vec![animal_stream, plant_stream], FILTER);

    // Act
    drop(plant_tx);

    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;

    // Assert
    assert_no_element_emitted(&mut combined_stream, 100).await;

    drop(person_tx);
    drop(animal_tx);

    let next = combined_stream.next().await;
    assert!(next.is_none(), "Expected stream to end without emissions");
    Ok(())
}

#[tokio::test]
async fn test_combine_latest_secondary_closes_after_initial_emission_continues(
) -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_stream) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_stream) = test_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_stream) = test_channel::<Sequenced<TestData>>();

    let mut combined_stream =
        person_stream.combine_latest(vec![animal_stream, plant_stream], FILTER);

    // Act
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;
    plant_tx.send(Sequenced::new(plant_rose()))?;

    // Assert
    let state = unwrap_value(combined_stream.next().await);
    let actual: Vec<TestData> = state.get().values().clone();
    assert_eq!(actual, vec![person_alice(), animal_dog(), plant_rose()]);

    drop(plant_tx);

    person_tx.send(Sequenced::new(person_bob()))?;
    let state = unwrap_value(combined_stream.next().await);
    let actual: Vec<TestData> = state.get().values().clone();
    assert_eq!(actual, vec![person_bob(), animal_dog(), plant_rose()]);

    animal_tx.send(Sequenced::new(animal_spider()))?;
    let state = unwrap_value(combined_stream.next().await);
    let actual: Vec<TestData> = state.get().values().clone();
    assert_eq!(actual, vec![person_bob(), animal_spider(), plant_rose()]);
    Ok(())
}

#[tokio::test]
async fn test_combine_latest_all_streams_have_published_different_order_emits_updates(
) -> anyhow::Result<()> {
    let _ =
        combine_latest_template_test(DataVariant::Plant, DataVariant::Animal, DataVariant::Person)
            .await;
    let _ =
        combine_latest_template_test(DataVariant::Plant, DataVariant::Person, DataVariant::Animal)
            .await;
    let _ =
        combine_latest_template_test(DataVariant::Animal, DataVariant::Plant, DataVariant::Person)
            .await;
    let _ =
        combine_latest_template_test(DataVariant::Animal, DataVariant::Person, DataVariant::Plant)
            .await;
    let _ =
        combine_latest_template_test(DataVariant::Person, DataVariant::Animal, DataVariant::Plant)
            .await;
    let _ =
        combine_latest_template_test(DataVariant::Person, DataVariant::Plant, DataVariant::Animal)
            .await;

    Ok(())
}

/// Test template for `combine_latest` that verifies consistent enum-based ordering regardless of send sequence.
/// Sets up channels for Person/Animal/Plant, combines streams, sends in specified Variant, and asserts
/// that results are always in enum variant Variant (Person, Animal, Plant) determined by the Ord trait,
/// not by send Variant. Called with different permutations to validate that ordering is based on enum
/// definition rather than temporal arrival sequence.
async fn combine_latest_template_test(
    order1: DataVariant,
    order2: DataVariant,
    order3: DataVariant,
) -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_stream) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_stream) = test_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_stream) = test_channel::<Sequenced<TestData>>();

    let senders = vec![person_tx, animal_tx, plant_tx];

    let mut combined_stream =
        person_stream.combine_latest(vec![animal_stream, plant_stream], FILTER);

    // Act
    send_variant(&order1, &senders)?;
    send_variant(&order2, &senders)?;
    send_variant(&order3, &senders)?;

    // Assert

    let state = unwrap_value(combined_stream.next().await);
    let actual: Vec<TestData> = state.get().values().clone();
    let expected = vec![person_alice(), animal_dog(), plant_rose()];

    assert_eq!(actual, expected);

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_all_streams_have_published_emits_updates() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_stream) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_stream) = test_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_stream) = test_channel::<Sequenced<TestData>>();

    let mut combined_stream =
        person_stream.combine_latest(vec![animal_stream, plant_stream], FILTER);

    // Act
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;
    plant_tx.send(Sequenced::new(plant_rose()))?;

    // Assert

    expect_next_combined_equals(
        &mut combined_stream,
        &[person_alice(), animal_dog(), plant_rose()],
    )
    .await;

    // Act
    person_tx.send(Sequenced::new(person_bob()))?;

    // Assert
    expect_next_combined_equals(
        &mut combined_stream,
        &[person_bob(), animal_dog(), plant_rose()],
    )
    .await;

    // Act
    animal_tx.send(Sequenced::new(animal_spider()))?;

    // Assert
    expect_next_combined_equals(
        &mut combined_stream,
        &[person_bob(), animal_spider(), plant_rose()],
    )
    .await;

    // Act
    plant_tx.send(Sequenced::new(plant_sunflower()))?;

    // Assert
    expect_next_combined_equals(
        &mut combined_stream,
        &[person_bob(), animal_spider(), plant_sunflower()],
    )
    .await;
    Ok(())
}

#[tokio::test]
async fn test_combine_latest_different_stream_order_emits_consistent_results() -> anyhow::Result<()>
{
    let _ = combine_latest_stream_order_test(
        DataVariant::Person,
        DataVariant::Animal,
        DataVariant::Plant,
    )
    .await;
    let _ = combine_latest_stream_order_test(
        DataVariant::Person,
        DataVariant::Plant,
        DataVariant::Animal,
    )
    .await;
    let _ = combine_latest_stream_order_test(
        DataVariant::Animal,
        DataVariant::Person,
        DataVariant::Plant,
    )
    .await;
    let _ = combine_latest_stream_order_test(
        DataVariant::Animal,
        DataVariant::Plant,
        DataVariant::Person,
    )
    .await;
    let _ = combine_latest_stream_order_test(
        DataVariant::Plant,
        DataVariant::Person,
        DataVariant::Animal,
    )
    .await;
    let _ = combine_latest_stream_order_test(
        DataVariant::Plant,
        DataVariant::Animal,
        DataVariant::Person,
    )
    .await;

    Ok(())
}

/// Test template for `combine_latest` that verifies consistent enum-based ordering regardless of stream Variant.
/// Sets up channels for Person/Animal/Plant in different orders, combines them, sends values, and asserts
/// that results are always in enum variant Variant (Person, Animal, Plant) determined by the Ord trait,
/// not by stream registration Variant. Called with different permutations to validate that ordering is based
/// on enum definition rather than the Variant streams were passed to combine_latest.
async fn combine_latest_stream_order_test(
    stream1: DataVariant,
    stream2: DataVariant,
    stream3: DataVariant,
) -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_stream) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_stream) = test_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_stream) = test_channel::<Sequenced<TestData>>();

    let mut registered_streams = vec![
        (DataVariant::Person, person_stream),
        (DataVariant::Animal, animal_stream),
        (DataVariant::Plant, plant_stream),
    ];

    let ordered_streams: Vec<_> = vec![&stream1, &stream2, &stream3]
        .into_iter()
        .map(|variant| {
            let idx = registered_streams
                .iter()
                .position(|(o, _)| {
                    matches!(
                        (o, variant),
                        (DataVariant::Person, DataVariant::Person)
                            | (DataVariant::Animal, DataVariant::Animal)
                            | (DataVariant::Plant, DataVariant::Plant)
                    )
                })
                .unwrap();
            registered_streams.remove(idx).1
        })
        .collect();

    let mut stream_iter = ordered_streams.into_iter();
    let first_stream = stream_iter.next().unwrap();
    let remaining_streams: Vec<_> = stream_iter.collect();

    let mut combined_stream = first_stream.combine_latest(remaining_streams, FILTER);

    // Act
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;
    plant_tx.send(Sequenced::new(plant_rose()))?;

    // Assert
    let state = unwrap_value(combined_stream.next().await);
    let actual: Vec<TestData> = state.get().values().clone();
    let expected = vec![person_alice(), animal_dog(), plant_rose()];

    assert_eq!(actual, expected);
    Ok(())
}

#[tokio::test]
async fn test_combine_latest_with_identical_streams_emits_updates() -> anyhow::Result<()> {
    // Arrange
    let (stream1_tx, stream1) = test_channel::<Sequenced<TestData>>();
    let (stream2_tx, stream2) = test_channel::<Sequenced<TestData>>();

    let mut combined_stream = stream1.combine_latest(vec![stream2], FILTER);

    // Act
    stream1_tx.send(Sequenced::new(person_alice()))?;
    stream2_tx.send(Sequenced::new(person_bob()))?;

    // Assert
    expect_next_combined_equals(&mut combined_stream, &[person_alice(), person_bob()]).await;

    // Act
    stream1_tx.send(Sequenced::new(person_charlie()))?;

    // Assert
    expect_next_combined_equals(&mut combined_stream, &[person_charlie(), person_bob()]).await;

    // Act
    stream2_tx.send(Sequenced::new(person_diane()))?;

    // Assert
    expect_next_combined_equals(&mut combined_stream, &[person_charlie(), person_diane()]).await;
    Ok(())
}

#[tokio::test]
async fn test_combine_latest_filter_rejects_initial_state() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_stream) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_stream) = test_channel::<Sequenced<TestData>>();

    // Filter that rejects the initial state (when both Alice and Dog are present)
    let filter = |state: &CombinedState<TestData>| {
        let values = state.values();
        if values.len() == 2 {
            // Reject if we have Alice and Dog
            !(values[0] == person_alice() && values[1] == animal_dog())
        } else {
            true
        }
    };

    let mut combined_stream = person_stream.combine_latest(vec![animal_stream], filter);

    // Act: Publish Alice and Dog (should be rejected)
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;

    // Assert: No emission due to filter rejection
    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Act: Update to Bob (should pass filter)
    person_tx.send(Sequenced::new(person_bob()))?;

    // Assert: Now we get an emission
    expect_next_combined_equals(&mut combined_stream, &[person_bob(), animal_dog()]).await;
    Ok(())
}

#[tokio::test]
async fn test_combine_latest_filter_alternates_between_true_false() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_stream) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_stream) = test_channel::<Sequenced<TestData>>();

    // Filter that only allows emissions when person is Alice or Charlie (rejects Bob and Diane)
    let filter = |state: &CombinedState<TestData>| {
        let values = state.values();
        if values.is_empty() {
            false
        } else {
            values[0] == person_alice() || values[0] == person_charlie()
        }
    };

    let mut combined_stream = person_stream.combine_latest(vec![animal_stream], filter);

    // Act: Alice + Dog (passes filter)
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;

    // Assert: Emission occurs
    expect_next_combined_equals(&mut combined_stream, &[person_alice(), animal_dog()]).await;

    // Act: Bob + Dog (rejected by filter)
    person_tx.send(Sequenced::new(person_bob()))?;

    // Assert: No emission
    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Act: Charlie + Dog (passes filter)
    person_tx.send(Sequenced::new(person_charlie()))?;

    // Assert: Emission occurs
    expect_next_combined_equals(&mut combined_stream, &[person_charlie(), animal_dog()]).await;

    // Act: Diane + Dog (rejected by filter)
    person_tx.send(Sequenced::new(person_diane()))?;

    // Assert: No emission
    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Act: Update animal to Spider while Diane is still latest (still rejected)
    animal_tx.send(Sequenced::new(animal_spider()))?;

    // Assert: Still no emission
    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Act: Alice + Spider (passes filter)
    person_tx.send(Sequenced::new(person_alice()))?;

    // Assert: Emission occurs
    expect_next_combined_equals(&mut combined_stream, &[person_alice(), animal_spider()]).await;
    Ok(())
}

#[tokio::test]
#[should_panic(expected = "Filter function panicked on purpose")]
async fn test_combine_latest_filter_function_panics() {
    // Arrange
    let (person_tx, person_stream) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_stream) = test_channel::<Sequenced<TestData>>();

    // Filter that panics on the second evaluation
    let call_count = Arc::new(AtomicUsize::new(0));
    let filter = move |_state: &CombinedState<TestData>| {
        let count = call_count.fetch_add(1, Ordering::SeqCst);
        if count == 1 {
            panic!("Filter function panicked on purpose");
        }
        true
    };

    let mut combined_stream = person_stream.combine_latest(vec![animal_stream], filter);

    // Act: First emission should succeed
    person_tx.send(Sequenced::new(person_alice())).unwrap();
    animal_tx.send(Sequenced::new(animal_dog())).unwrap();
    let _first = unwrap_value(combined_stream.next().await);

    // Act: Second emission triggers panic in filter
    person_tx.send(Sequenced::new(person_bob())).unwrap();
    let _second = combined_stream.next().await; // This will panic
}
