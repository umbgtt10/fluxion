use fluxion_stream::{
    combine_latest::{CombineLatestExt, CombinedState},
    sequenced::Sequenced,
    sequenced_channel::unbounded_channel,
};
use fluxion_test_utils::{
    helpers::assert_no_element_emitted,
    test_value::{
        TestValue, Variant, animal_dog, animal_spider, person_alice, person_bob, person_charlie,
        person_diane, plant_rose, plant_sunflower, push, send,
    },
};
use futures::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;

static FILTER: fn(&CombinedState<Sequenced<TestValue>>) -> bool =
    |_: &CombinedState<Sequenced<TestValue>>| true;

#[tokio::test]
async fn test_combine_latest_empty_streams() {
    // Arrange
    let (_, person_receiver) = unbounded_channel::<TestValue>();
    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());

    let (_, animal_receiver) = unbounded_channel::<TestValue>();
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());

    let (_, plant_receiver) = unbounded_channel::<TestValue>();
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let combined_stream = person_stream.combine_latest(vec![animal_stream, plant_stream], FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Assert
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
    let mut combined_stream = Box::pin(combined_stream);

    // Act
    push(person_alice(), &person_sender);

    // Assert
    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Act
    push(animal_dog(), &animal_sender);

    // Assert
    assert_no_element_emitted(&mut combined_stream, 100).await;
}

#[tokio::test]
async fn test_combine_latest_all_streams_have_published_different_order_emits_updates() {
    combine_latest_template_test(Variant::Plant, Variant::Animal, Variant::Person).await;
    combine_latest_template_test(Variant::Plant, Variant::Person, Variant::Animal).await;
    combine_latest_template_test(Variant::Animal, Variant::Plant, Variant::Person).await;
    combine_latest_template_test(Variant::Animal, Variant::Person, Variant::Plant).await;
    combine_latest_template_test(Variant::Person, Variant::Animal, Variant::Plant).await;
    combine_latest_template_test(Variant::Person, Variant::Plant, Variant::Animal).await;
}

/// Test template for `combine_latest` that verifies consistent enum-based ordering regardless of send sequence.
/// Sets up channels for Person/Animal/Plant, combines streams, sends in specified Variant, and asserts
/// that results are always in enum variant Variant (Person, Animal, Plant) determined by the Ord trait,
/// not by send Variant. Called with different permutations to validate that ordering is based on enum
/// definition rather than temporal arrival sequence.
async fn combine_latest_template_test(order1: Variant, order2: Variant, order3: Variant) {
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
    send(&order1, &senders);
    send(&order2, &senders);
    send(&order3, &senders);

    // Assert
    let mut combined_stream = Box::pin(combined_stream);

    let state = combined_stream.next().await.unwrap();
    let actual: Vec<TestValue> = state.get_state().iter().map(|s| s.value.clone()).collect();
    let expected = vec![person_alice(), animal_dog(), plant_rose()];

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn test_combine_latest_all_streams_have_published_emits_updates() {
    // Arrange
    let (person_sender, person_receiver) = unbounded_channel::<TestValue>();
    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());

    let (animal_sender, animal_receiver) = unbounded_channel::<TestValue>();
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());

    let (plant_sender, plant_receiver) = unbounded_channel::<TestValue>();
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let combined_stream = person_stream.combine_latest(vec![animal_stream, plant_stream], FILTER);

    // Act
    push(person_alice(), &person_sender);
    push(animal_dog(), &animal_sender);
    push(plant_rose(), &plant_sender);

    // Assert
    let mut combined_stream = Box::pin(combined_stream);

    let state = combined_stream.next().await.unwrap();
    let actual: Vec<TestValue> = state.get_state().iter().map(|s| s.value.clone()).collect();
    let expected = vec![person_alice(), animal_dog(), plant_rose()];
    assert_eq!(actual, expected);

    // Act
    push(person_bob(), &person_sender);

    // Assert
    let state = combined_stream.next().await.unwrap();
    let actual: Vec<TestValue> = state.get_state().iter().map(|s| s.value.clone()).collect();
    let expected = vec![person_bob(), animal_dog(), plant_rose()];
    assert_eq!(actual, expected);

    // Act
    push(animal_spider(), &animal_sender);

    // Assert
    let state = combined_stream.next().await.unwrap();
    let actual: Vec<TestValue> = state.get_state().iter().map(|s| s.value.clone()).collect();
    let expected = vec![person_bob(), animal_spider(), plant_rose()];
    assert_eq!(actual, expected);

    // Act
    push(plant_sunflower(), &plant_sender);

    // Assert
    let state = combined_stream.next().await.unwrap();
    let actual: Vec<TestValue> = state.get_state().iter().map(|s| s.value.clone()).collect();
    let expected = vec![person_bob(), animal_spider(), plant_sunflower()];
    assert_eq!(actual, expected);
}

#[tokio::test]
async fn test_combine_latest_different_stream_order_emits_consistent_results() {
    combine_latest_stream_order_test(Variant::Person, Variant::Animal, Variant::Plant).await;
    combine_latest_stream_order_test(Variant::Person, Variant::Plant, Variant::Animal).await;
    combine_latest_stream_order_test(Variant::Animal, Variant::Person, Variant::Plant).await;
    combine_latest_stream_order_test(Variant::Animal, Variant::Plant, Variant::Person).await;
    combine_latest_stream_order_test(Variant::Plant, Variant::Person, Variant::Animal).await;
    combine_latest_stream_order_test(Variant::Plant, Variant::Animal, Variant::Person).await;
}

/// Test template for `combine_latest` that verifies consistent enum-based ordering regardless of stream Variant.
/// Sets up channels for Person/Animal/Plant in different orders, combines them, sends values, and asserts
/// that results are always in enum variant Variant (Person, Animal, Plant) determined by the Ord trait,
/// not by stream registration Variant. Called with different permutations to validate that ordering is based
/// on enum definition rather than the Variant streams were passed to combine_latest.
async fn combine_latest_stream_order_test(stream1: Variant, stream2: Variant, stream3: Variant) {
    // Arrange
    let (person_sender, person_receiver) = unbounded_channel();
    let (animal_sender, animal_receiver) = unbounded_channel();
    let (plant_sender, plant_receiver) = unbounded_channel();

    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let mut streams = vec![
        (Variant::Person, person_stream),
        (Variant::Animal, animal_stream),
        (Variant::Plant, plant_stream),
    ];

    let ordered_streams: Vec<_> = vec![&stream1, &stream2, &stream3]
        .into_iter()
        .map(|variant| {
            let idx = streams
                .iter()
                .position(|(o, _)| {
                    matches!(
                        (o, variant),
                        (Variant::Person, Variant::Person)
                            | (Variant::Animal, Variant::Animal)
                            | (Variant::Plant, Variant::Plant)
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
    let actual: Vec<TestValue> = state.get_state().iter().map(|s| s.value.clone()).collect();
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
    let state = combined_stream.next().await.unwrap();
    let actual: Vec<TestValue> = state.get_state().iter().map(|s| s.value.clone()).collect();
    let expected = vec![person_alice(), person_bob()];
    assert_eq!(actual, expected);

    // Act
    push(person_charlie(), &stream1_sender);

    // Assert
    let state = combined_stream.next().await.unwrap();
    let actual: Vec<TestValue> = state.get_state().iter().map(|s| s.value.clone()).collect();
    let expected = vec![person_charlie(), person_bob()];
    assert_eq!(actual, expected);

    // Act
    push(person_diane(), &stream2_sender);

    // Assert
    let state = combined_stream.next().await.unwrap();
    let actual: Vec<TestValue> = state.get_state().iter().map(|s| s.value.clone()).collect();
    let expected = vec![person_charlie(), person_diane()];
    assert_eq!(actual, expected);
}
