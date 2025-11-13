use fluxion_stream::combine_latest::{CombineLatestExt, CombinedState};
use fluxion_test_utils::FluxionChannel;
use fluxion_test_utils::{
    TestChannels,
    helpers::{assert_no_element_emitted, expect_next_combined_equals},
    test_data::{
        DataVariant, TestData, animal_dog, animal_spider, person_alice, person_bob, person_charlie,
        person_diane, plant_rose, plant_sunflower, push, send_variant,
    },
};
use futures::StreamExt;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

static FILTER: fn(&CombinedState<TestData>) -> bool = |_: &CombinedState<TestData>| true;

#[tokio::test]
async fn test_combine_latest_empty_streams() {
    // Arrange
    let (person, animal, plant) = TestChannels::three::<TestData>();

    let combined_stream = person
        .stream
        .combine_latest(vec![animal.stream, plant.stream], FILTER);

    // Act
    drop(person.sender);
    drop(animal.sender);
    drop(plant.sender);

    // Assert
    let mut combined_stream = Box::pin(combined_stream);
    let next_item = combined_stream.next().await;
    assert!(
        next_item.is_none(),
        "Expected no items from an empty combined stream"
    );
}

#[tokio::test]
async fn test_combine_latest_not_all_streams_have_published_does_not_emit() {
    // Arrange
    let person = FluxionChannel::new();
    let animal = FluxionChannel::new();
    let plant = FluxionChannel::new();

    let combined_stream = person
        .stream
        .combine_latest(vec![animal.stream, plant.stream], FILTER);

    // Act
    push(person_alice(), &person.sender);

    // Assert
    let mut combined_stream = Box::pin(combined_stream);
    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Act
    push(animal_dog(), &animal.sender);

    // Assert
    assert_no_element_emitted(&mut combined_stream, 100).await;
}

#[tokio::test]
async fn test_combine_latest_stream_closes_before_publish_no_output() {
    // Arrange
    let person = FluxionChannel::new();
    let animal = FluxionChannel::new();
    let plant = FluxionChannel::new();

    let combined_stream = person
        .stream
        .combine_latest(vec![animal.stream, plant.stream], FILTER);

    // Act
    drop(plant.sender);

    push(person_alice(), &person.sender);
    push(animal_dog(), &animal.sender);

    // Assert
    let mut combined_stream = Box::pin(combined_stream);
    assert_no_element_emitted(&mut combined_stream, 100).await;

    drop(person.sender);
    drop(animal.sender);

    let next = combined_stream.next().await;
    assert!(next.is_none(), "Expected stream to end without emissions");
}

#[tokio::test]
async fn test_combine_latest_secondary_closes_after_initial_emission_continues() {
    // Arrange
    let person = FluxionChannel::new();
    let animal = FluxionChannel::new();
    let plant = FluxionChannel::new();

    let combined_stream = person
        .stream
        .combine_latest(vec![animal.stream, plant.stream], FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act
    push(person_alice(), &person.sender);
    push(animal_dog(), &animal.sender);
    push(plant_rose(), &plant.sender);

    // Assert
    let state = combined_stream.next().await.unwrap();
    let actual: Vec<TestData> = state.value.get_state().to_vec();
    assert_eq!(actual, vec![person_alice(), animal_dog(), plant_rose()]);

    drop(plant.sender);

    push(person_bob(), &person.sender);
    let state = combined_stream.next().await.unwrap();
    let actual: Vec<TestData> = state.value.get_state().to_vec();
    assert_eq!(actual, vec![person_bob(), animal_dog(), plant_rose()]);

    push(animal_spider(), &animal.sender);
    let state = combined_stream.next().await.unwrap();
    let actual: Vec<TestData> = state.value.get_state().to_vec();
    assert_eq!(actual, vec![person_bob(), animal_spider(), plant_rose()]);
}

#[tokio::test]
async fn test_combine_latest_all_streams_have_published_different_order_emits_updates() {
    combine_latest_template_test(DataVariant::Plant, DataVariant::Animal, DataVariant::Person)
        .await;
    combine_latest_template_test(DataVariant::Plant, DataVariant::Person, DataVariant::Animal)
        .await;
    combine_latest_template_test(DataVariant::Animal, DataVariant::Plant, DataVariant::Person)
        .await;
    combine_latest_template_test(DataVariant::Animal, DataVariant::Person, DataVariant::Plant)
        .await;
    combine_latest_template_test(DataVariant::Person, DataVariant::Animal, DataVariant::Plant)
        .await;
    combine_latest_template_test(DataVariant::Person, DataVariant::Plant, DataVariant::Animal)
        .await;
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
) {
    // Arrange
    let (person, animal, plant) = TestChannels::three::<TestData>();

    let senders = vec![person.sender, animal.sender, plant.sender];

    let combined_stream = person
        .stream
        .combine_latest(vec![animal.stream, plant.stream], FILTER);

    // Act
    send_variant(&order1, &senders);
    send_variant(&order2, &senders);
    send_variant(&order3, &senders);

    // Assert
    let mut combined_stream = Box::pin(combined_stream);

    let state = combined_stream.next().await.unwrap();
    let actual: Vec<TestData> = state.value.get_state().to_vec();
    let expected = vec![person_alice(), animal_dog(), plant_rose()];

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn test_combine_latest_all_streams_have_published_emits_updates() {
    // Arrange
    let (person, animal, plant) = TestChannels::three::<TestData>();

    let combined_stream = person
        .stream
        .combine_latest(vec![animal.stream, plant.stream], FILTER);

    // Act
    push(person_alice(), &person.sender);
    push(animal_dog(), &animal.sender);
    push(plant_rose(), &plant.sender);

    // Assert
    let mut combined_stream = Box::pin(combined_stream);

    expect_next_combined_equals(
        &mut combined_stream,
        &[person_alice(), animal_dog(), plant_rose()],
    )
    .await;

    // Act
    push(person_bob(), &person.sender);

    // Assert
    expect_next_combined_equals(
        &mut combined_stream,
        &[person_bob(), animal_dog(), plant_rose()],
    )
    .await;

    // Act
    push(animal_spider(), &animal.sender);

    // Assert
    expect_next_combined_equals(
        &mut combined_stream,
        &[person_bob(), animal_spider(), plant_rose()],
    )
    .await;

    // Act
    push(plant_sunflower(), &plant.sender);

    // Assert
    expect_next_combined_equals(
        &mut combined_stream,
        &[person_bob(), animal_spider(), plant_sunflower()],
    )
    .await;
}

#[tokio::test]
async fn test_combine_latest_different_stream_order_emits_consistent_results() {
    combine_latest_stream_order_test(DataVariant::Person, DataVariant::Animal, DataVariant::Plant)
        .await;
    combine_latest_stream_order_test(DataVariant::Person, DataVariant::Plant, DataVariant::Animal)
        .await;
    combine_latest_stream_order_test(DataVariant::Animal, DataVariant::Person, DataVariant::Plant)
        .await;
    combine_latest_stream_order_test(DataVariant::Animal, DataVariant::Plant, DataVariant::Person)
        .await;
    combine_latest_stream_order_test(DataVariant::Plant, DataVariant::Person, DataVariant::Animal)
        .await;
    combine_latest_stream_order_test(DataVariant::Plant, DataVariant::Animal, DataVariant::Person)
        .await;
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
) {
    // Arrange
    let person = FluxionChannel::new();
    let animal = FluxionChannel::new();
    let plant = FluxionChannel::new();

    let mut streams = vec![
        (DataVariant::Person, person.stream),
        (DataVariant::Animal, animal.stream),
        (DataVariant::Plant, plant.stream),
    ];

    let ordered_streams: Vec<_> = vec![&stream1, &stream2, &stream3]
        .into_iter()
        .map(|variant| {
            let idx = streams
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
            streams.remove(idx).1
        })
        .collect();

    let mut stream_iter = ordered_streams.into_iter();
    let first_stream = stream_iter.next().unwrap();
    let remaining_streams: Vec<_> = stream_iter.collect();

    let combined_stream = first_stream.combine_latest(remaining_streams, FILTER);

    // Act
    push(person_alice(), &person.sender);
    push(animal_dog(), &animal.sender);
    push(plant_rose(), &plant.sender);

    // Assert
    let mut combined_stream = Box::pin(combined_stream);
    let state = combined_stream.next().await.unwrap();
    let actual: Vec<TestData> = state.value.get_state().to_vec();
    let expected = vec![person_alice(), animal_dog(), plant_rose()];

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn test_combine_latest_with_identical_streams_emits_updates() {
    // Arrange
    let (stream1, stream2) = TestChannels::two();

    let combined_stream = stream1.stream.combine_latest(vec![stream2.stream], FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act
    push(person_alice(), &stream1.sender);
    push(person_bob(), &stream2.sender);

    // Assert
    expect_next_combined_equals(&mut combined_stream, &[person_alice(), person_bob()]).await;

    // Act
    push(person_charlie(), &stream1.sender);

    // Assert
    expect_next_combined_equals(&mut combined_stream, &[person_charlie(), person_bob()]).await;

    // Act
    push(person_diane(), &stream2.sender);

    // Assert
    expect_next_combined_equals(&mut combined_stream, &[person_charlie(), person_diane()]).await;
}

#[tokio::test]
async fn test_combine_latest_filter_rejects_initial_state() {
    // Arrange
    let (person, animal) = TestChannels::two::<TestData>();

    // Filter that rejects the initial state (when both Alice and Dog are present)
    let filter = |state: &CombinedState<TestData>| {
        let values = state.get_state();
        if values.len() == 2 {
            // Reject if we have Alice and Dog
            !(values[0] == person_alice() && values[1] == animal_dog())
        } else {
            true
        }
    };

    let combined_stream = person.stream.combine_latest(vec![animal.stream], filter);
    let mut combined_stream = Box::pin(combined_stream);

    // Act: Publish Alice and Dog (should be rejected)
    push(person_alice(), &person.sender);
    push(animal_dog(), &animal.sender);

    // Assert: No emission due to filter rejection
    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Act: Update to Bob (should pass filter)
    push(person_bob(), &person.sender);

    // Assert: Now we get an emission
    expect_next_combined_equals(&mut combined_stream, &[person_bob(), animal_dog()]).await;
}

#[tokio::test]
async fn test_combine_latest_filter_alternates_between_true_false() {
    // Arrange
    let (person, animal) = TestChannels::two::<TestData>();

    // Filter that only allows emissions when person is Alice or Charlie (rejects Bob and Diane)
    let filter = |state: &CombinedState<TestData>| {
        let values = state.get_state();
        if !values.is_empty() {
            values[0] == person_alice() || values[0] == person_charlie()
        } else {
            false
        }
    };

    let combined_stream = person.stream.combine_latest(vec![animal.stream], filter);
    let mut combined_stream = Box::pin(combined_stream);

    // Act: Alice + Dog (passes filter)
    push(person_alice(), &person.sender);
    push(animal_dog(), &animal.sender);

    // Assert: Emission occurs
    expect_next_combined_equals(&mut combined_stream, &[person_alice(), animal_dog()]).await;

    // Act: Bob + Dog (rejected by filter)
    push(person_bob(), &person.sender);

    // Assert: No emission
    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Act: Charlie + Dog (passes filter)
    push(person_charlie(), &person.sender);

    // Assert: Emission occurs
    expect_next_combined_equals(&mut combined_stream, &[person_charlie(), animal_dog()]).await;

    // Act: Diane + Dog (rejected by filter)
    push(person_diane(), &person.sender);

    // Assert: No emission
    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Act: Update animal to Spider while Diane is still latest (still rejected)
    push(animal_spider(), &animal.sender);

    // Assert: Still no emission
    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Act: Alice + Spider (passes filter)
    push(person_alice(), &person.sender);

    // Assert: Emission occurs
    expect_next_combined_equals(&mut combined_stream, &[person_alice(), animal_spider()]).await;
}

#[tokio::test]
#[should_panic(expected = "Filter function panicked on purpose")]
async fn test_combine_latest_filter_function_panics() {
    // Arrange
    let (person, animal) = TestChannels::two::<TestData>();

    // Filter that panics on the second evaluation
    let call_count = Arc::new(AtomicUsize::new(0));
    let filter = move |_state: &CombinedState<TestData>| {
        let count = call_count.fetch_add(1, Ordering::SeqCst);
        if count == 1 {
            panic!("Filter function panicked on purpose");
        }
        true
    };

    let combined_stream = person.stream.combine_latest(vec![animal.stream], filter);
    let mut combined_stream = Box::pin(combined_stream);

    // Act: First emission should succeed
    push(person_alice(), &person.sender);
    push(animal_dog(), &animal.sender);
    let _first = combined_stream.next().await.unwrap();

    // Act: Second emission triggers panic in filter
    push(person_bob(), &person.sender);
    let _second = combined_stream.next().await; // This will panic
}
