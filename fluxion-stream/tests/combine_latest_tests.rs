use fluxion_stream::{
    combine_latest::{CombineLatestExt, CombinedState},
    timestamped::Timestamped,
    timestamped_channel::unbounded_channel,
};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, expect_next_combined_equals},
    test_data::{
        DataVariant, TestData, animal_dog, animal_spider, person_alice, person_bob, person_charlie,
        person_diane, plant_rose, plant_sunflower, push, send_variant,
    },
};
use futures::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;

static FILTER: fn(&CombinedState<Timestamped<TestData>>) -> bool =
    |_: &CombinedState<Timestamped<TestData>>| true;

#[tokio::test]
async fn test_combine_latest_empty_streams() {
    // Arrange
    let (_, person_receiver) = unbounded_channel::<TestData>();
    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());

    let (_, animal_receiver) = unbounded_channel::<TestData>();
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());

    let (_, plant_receiver) = unbounded_channel::<TestData>();
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let combined_stream = person_stream.combine_latest(vec![animal_stream, plant_stream], FILTER);

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
    let (person_sender, person_receiver) = unbounded_channel();
    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());

    let (animal_sender, animal_receiver) = unbounded_channel();
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());

    let (_, plant_receiver) = unbounded_channel();
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let combined_stream = person_stream.combine_latest(vec![animal_stream, plant_stream], FILTER);

    // Act
    push(person_alice(), &person_sender);

    // Assert
    let mut combined_stream = Box::pin(combined_stream);
    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Act
    push(animal_dog(), &animal_sender);

    // Assert
    assert_no_element_emitted(&mut combined_stream, 100).await;
}

#[tokio::test]
async fn test_combine_latest_stream_closes_before_publish_no_output() {
    // Arrange
    let (person_sender, person_receiver) = unbounded_channel();
    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());

    let (animal_sender, animal_receiver) = unbounded_channel();
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());

    let (plant_sender, plant_receiver) = unbounded_channel();
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let combined_stream = person_stream.combine_latest(vec![animal_stream, plant_stream], FILTER);

    // Act: Close one stream before it ever publishes
    drop(plant_sender);

    // Publish on the other streams
    push(person_alice(), &person_sender);
    push(animal_dog(), &animal_sender);

    // Assert: No emission should occur because not all streams have published
    let mut combined_stream = Box::pin(combined_stream);
    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Close remaining senders; the stream should end cleanly
    drop(person_sender);
    drop(animal_sender);

    let next = combined_stream.next().await;
    assert!(next.is_none(), "Expected stream to end without emissions");
}

#[tokio::test]
async fn test_combine_latest_secondary_closes_after_initial_emission_continues() {
    // Arrange
    let (person_sender, person_receiver) = unbounded_channel();
    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());

    let (animal_sender, animal_receiver) = unbounded_channel();
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());

    let (plant_sender, plant_receiver) = unbounded_channel();
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let combined_stream = person_stream.combine_latest(vec![animal_stream, plant_stream], FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Initial full emission
    push(person_alice(), &person_sender);
    push(animal_dog(), &animal_sender);
    push(plant_rose(), &plant_sender);
    let state = combined_stream.next().await.unwrap();
    let actual: Vec<TestData> = state.get_state().iter().map(|s| s.value.clone()).collect();
    assert_eq!(actual, vec![person_alice(), animal_dog(), plant_rose()]);

    // Close one secondary (plant) after initial emission
    drop(plant_sender);

    // Subsequent updates on remaining streams should still emit using the last plant value
    push(person_bob(), &person_sender);
    let state = combined_stream.next().await.unwrap();
    let actual: Vec<TestData> = state.get_state().iter().map(|s| s.value.clone()).collect();
    assert_eq!(actual, vec![person_bob(), animal_dog(), plant_rose()]);

    push(animal_spider(), &animal_sender);
    let state = combined_stream.next().await.unwrap();
    let actual: Vec<TestData> = state.get_state().iter().map(|s| s.value.clone()).collect();
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
    let (person_sender, person_receiver) = unbounded_channel();
    let (animal_sender, animal_receiver) = unbounded_channel();
    let (plant_sender, plant_receiver) = unbounded_channel();

    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let senders = vec![person_sender, animal_sender, plant_sender];

    let combined_stream = person_stream.combine_latest(vec![animal_stream, plant_stream], FILTER);

    // Act
    send_variant(&order1, &senders);
    send_variant(&order2, &senders);
    send_variant(&order3, &senders);

    // Assert
    let mut combined_stream = Box::pin(combined_stream);

    let state = combined_stream.next().await.unwrap();
    let actual: Vec<TestData> = state.get_state().iter().map(|s| s.value.clone()).collect();
    let expected = vec![person_alice(), animal_dog(), plant_rose()];

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn test_combine_latest_all_streams_have_published_emits_updates() {
    // Arrange
    let (person_sender, person_receiver) = unbounded_channel::<TestData>();
    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());

    let (animal_sender, animal_receiver) = unbounded_channel::<TestData>();
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());

    let (plant_sender, plant_receiver) = unbounded_channel::<TestData>();
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let combined_stream = person_stream.combine_latest(vec![animal_stream, plant_stream], FILTER);

    // Act
    push(person_alice(), &person_sender);
    push(animal_dog(), &animal_sender);
    push(plant_rose(), &plant_sender);

    // Assert
    let mut combined_stream = Box::pin(combined_stream);

    expect_next_combined_equals(
        &mut combined_stream,
        &[person_alice(), animal_dog(), plant_rose()],
    )
    .await;

    // Act
    push(person_bob(), &person_sender);

    // Assert
    expect_next_combined_equals(
        &mut combined_stream,
        &[person_bob(), animal_dog(), plant_rose()],
    )
    .await;

    // Act
    push(animal_spider(), &animal_sender);

    // Assert
    expect_next_combined_equals(
        &mut combined_stream,
        &[person_bob(), animal_spider(), plant_rose()],
    )
    .await;

    // Act
    push(plant_sunflower(), &plant_sender);

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
    let (person_sender, person_receiver) = unbounded_channel();
    let (animal_sender, animal_receiver) = unbounded_channel();
    let (plant_sender, plant_receiver) = unbounded_channel();

    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let mut streams = vec![
        (DataVariant::Person, person_stream),
        (DataVariant::Animal, animal_stream),
        (DataVariant::Plant, plant_stream),
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
    push(person_alice(), &person_sender);
    push(animal_dog(), &animal_sender);
    push(plant_rose(), &plant_sender);

    // Assert
    let mut combined_stream = Box::pin(combined_stream);
    let state = combined_stream.next().await.unwrap();
    let actual: Vec<TestData> = state.get_state().iter().map(|s| s.value.clone()).collect();
    let expected = vec![person_alice(), animal_dog(), plant_rose()];

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn test_combine_latest_with_identical_streams_emits_updates() {
    // Arrange
    let (stream1_sender, stream1_receiver) = unbounded_channel();
    let stream1 = UnboundedReceiverStream::new(stream1_receiver.into_inner());

    let (stream2_sender, stream2_receiver) = unbounded_channel();
    let stream2 = UnboundedReceiverStream::new(stream2_receiver.into_inner());

    let combined_stream = stream1.combine_latest(vec![stream2], FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act
    push(person_alice(), &stream1_sender);
    push(person_bob(), &stream2_sender);

    // Assert
    expect_next_combined_equals(&mut combined_stream, &[person_alice(), person_bob()]).await;

    // Act
    push(person_charlie(), &stream1_sender);

    // Assert
    expect_next_combined_equals(&mut combined_stream, &[person_charlie(), person_bob()]).await;

    // Act
    push(person_diane(), &stream2_sender);

    // Assert
    expect_next_combined_equals(&mut combined_stream, &[person_charlie(), person_diane()]).await;
}
