mod infra;
mod test_data;

use fluxion::{
    combine_latest::{CombineLatestExt, CombinedState},
    sequenced::Sequenced,
    sequenced_channel::unbounded_channel,
};
use futures::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{
    infra::infrastructure::assert_no_element_emitted,
    test_data::simple_enum::{
        Order, SimpleEnum, alice, bob, charlie, diane, dog, rose, send, send_alice, send_bob,
        send_charlie, send_diane, send_dog, send_rose, send_spider, send_sunflower, spider,
        sunflower,
    },
};

static FILTER2: fn(&CombinedState<Sequenced<SimpleEnum>>) -> bool =
    |_: &CombinedState<Sequenced<SimpleEnum>>| true;

#[tokio::test]
async fn test_combine_latest_empty_streams() {
    // Arrange
    let (_, person_receiver) = unbounded_channel::<SimpleEnum>();
    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());

    let (_, animal_receiver) = unbounded_channel::<SimpleEnum>();
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());

    let (_, plant_receiver) = unbounded_channel::<SimpleEnum>();
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let combined_stream = person_stream.combine_latest(vec![animal_stream, plant_stream], FILTER2);
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

    let combined_stream = person_stream.combine_latest(vec![animal_stream, plant_stream], FILTER2);
    let mut combined_stream = Box::pin(combined_stream);

    // Act
    send_alice(&person_sender);

    // Assert
    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Act
    send_dog(&animal_sender);

    // Assert
    assert_no_element_emitted(&mut combined_stream, 100).await;
}

#[tokio::test]
async fn test_combine_latest_all_streams_have_published_different_order_emits_updates() {
    combine_latest_template_test(Order::Plant, Order::Animal, Order::Person).await;
    combine_latest_template_test(Order::Plant, Order::Person, Order::Animal).await;
    combine_latest_template_test(Order::Animal, Order::Plant, Order::Person).await;
    combine_latest_template_test(Order::Animal, Order::Person, Order::Plant).await;
    combine_latest_template_test(Order::Person, Order::Animal, Order::Plant).await;
    combine_latest_template_test(Order::Person, Order::Plant, Order::Animal).await;
}

/// Test template for `combine_latest` that verifies temporal ordering based on send sequence.
/// Sets up channels for Person/Animal/Plant, combines streams, sends in specified order, and asserts
/// that results are in temporal order (matching the send order) due to Sequenced wrapper and select_all_ordered.
/// Called with different permutations to validate temporal ordering is maintained across different send sequences.
async fn combine_latest_template_test(order1: Order, order2: Order, order3: Order) {
    // Arrange
    let (person_sender, person_receiver) = unbounded_channel();
    let (animal_sender, animal_receiver) = unbounded_channel();
    let (plant_sender, plant_receiver) = unbounded_channel();

    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let senders = vec![person_sender, animal_sender, plant_sender];

    let combined_stream = person_stream.combine_latest(vec![animal_stream, plant_stream], FILTER2);

    // Act
    send(&order1, &senders);
    send(&order2, &senders);
    send(&order3, &senders);

    // Assert
    let mut combined_stream = Box::pin(combined_stream);

    let state = combined_stream.next().await.unwrap();
    let actual: Vec<SimpleEnum> = state.get_state().iter().map(|s| s.value.clone()).collect();

    let order_to_value = |order: &Order| match order {
        Order::Person => alice(),
        Order::Animal => dog(),
        Order::Plant => rose(),
    };

    let expected = vec![
        order_to_value(&order1),
        order_to_value(&order2),
        order_to_value(&order3),
    ];
    assert_eq!(actual, expected);
}

#[tokio::test]
async fn test_combine_latest_all_streams_have_published_emits_updates() {
    // Arrange
    let (person_sender, person_receiver) = unbounded_channel::<SimpleEnum>();
    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());

    let (animal_sender, animal_receiver) = unbounded_channel::<SimpleEnum>();
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());

    let (plant_sender, plant_receiver) = unbounded_channel::<SimpleEnum>();
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let combined_stream = person_stream.combine_latest(vec![animal_stream, plant_stream], FILTER2);

    // Act
    send_alice(&person_sender);
    send_dog(&animal_sender);
    send_rose(&plant_sender);

    // Assert
    let mut combined_stream = Box::pin(combined_stream);

    let state = combined_stream.next().await.unwrap();
    let actual: Vec<SimpleEnum> = state.get_state().iter().map(|s| s.value.clone()).collect();
    let expected = vec![alice(), dog(), rose()];
    assert_eq!(actual, expected);

    // Act
    send_bob(&person_sender);

    // Assert
    let state = combined_stream.next().await.unwrap();
    let actual: Vec<SimpleEnum> = state.get_state().iter().map(|s| s.value.clone()).collect();
    let expected = vec![dog(), rose(), bob()];
    assert_eq!(actual, expected);

    // Act
    send_spider(&animal_sender);

    // Assert
    let state = combined_stream.next().await.unwrap();
    let actual: Vec<SimpleEnum> = state.get_state().iter().map(|s| s.value.clone()).collect();
    let expected = vec![rose(), bob(), spider()];
    assert_eq!(actual, expected);

    // Act
    send_sunflower(&plant_sender);

    // Assert
    let state = combined_stream.next().await.unwrap();
    let actual: Vec<SimpleEnum> = state.get_state().iter().map(|s| s.value.clone()).collect();
    let expected = vec![bob(), spider(), sunflower()];
    assert_eq!(actual, expected);
}

#[tokio::test]
async fn test_combine_latest_with_identical_streams_emits_updates() {
    // Arrange
    let (stream1_sender, stream1_receiver) = unbounded_channel();
    let stream1 = UnboundedReceiverStream::new(stream1_receiver.into_inner());

    let (stream2_sender, stream2_receiver) = unbounded_channel();
    let stream2 = UnboundedReceiverStream::new(stream2_receiver.into_inner());

    let combined_stream = stream1.combine_latest(vec![stream2], FILTER2);
    let mut combined_stream = Box::pin(combined_stream);

    // Act
    send_alice(&stream1_sender);
    send_bob(&stream2_sender);

    // Assert
    let state = combined_stream.next().await.unwrap();
    let actual: Vec<SimpleEnum> = state.get_state().iter().map(|s| s.value.clone()).collect();
    let expected = vec![alice(), bob()];
    assert_eq!(actual, expected);

    // Act
    send_charlie(&stream1_sender);

    // Assert
    let state = combined_stream.next().await.unwrap();
    let actual: Vec<SimpleEnum> = state.get_state().iter().map(|s| s.value.clone()).collect();
    let expected = vec![bob(), charlie()];
    assert_eq!(actual, expected);

    // Act
    send_diane(&stream2_sender);

    // Assert
    let state = combined_stream.next().await.unwrap();
    let actual: Vec<SimpleEnum> = state.get_state().iter().map(|s| s.value.clone()).collect();
    let expected = vec![charlie(), diane()];
    assert_eq!(actual, expected);
}
