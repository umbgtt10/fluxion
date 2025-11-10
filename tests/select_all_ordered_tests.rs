mod test_data;

use crate::test_data::simple_enum::{Order, assert, send};
use fluxion::select_all_ordered::SelectAllExt;
use fluxion::sequenced_channel::unbounded_channel;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[tokio::test]
async fn test_select_all_ordered_all_permutations() {
    select_all_ordered_template_test(Order::Person, Order::Animal, Order::Plant).await;
    select_all_ordered_template_test(Order::Person, Order::Plant, Order::Animal).await;
    select_all_ordered_template_test(Order::Plant, Order::Animal, Order::Person).await;
    select_all_ordered_template_test(Order::Plant, Order::Person, Order::Animal).await;
    select_all_ordered_template_test(Order::Animal, Order::Person, Order::Plant).await;
    select_all_ordered_template_test(Order::Animal, Order::Plant, Order::Person).await;
}

/// Test template for `select_all_ordered` that verifies temporal order preservation in stream merging.
/// Sets up channels for Person/Animal/Plant, merges streams, sends in specified order, and asserts correct reception.
/// Crucial for deterministic async processing; ensures `select_all_ordered` avoids race conditions of `select_all`.
/// Called with different permutations to validate robustness across send sequences.
/// Validates sequence-based ordering over poll-time randomness for reliable event handling.
async fn select_all_ordered_template_test(order1: Order, order2: Order, order3: Order) {
    // Arrange
    let (person_sender, person_receiver) = unbounded_channel();
    let (animal_sender, animal_receiver) = unbounded_channel();
    let (plant_sender, plant_receiver) = unbounded_channel();

    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let senders = vec![person_sender, animal_sender, plant_sender];
    let streams = vec![person_stream, animal_stream, plant_stream];

    let results = streams.select_all_ordered();

    // Act
    send(&order1, &senders);
    send(&order2, &senders);
    send(&order3, &senders);

    // Assert
    let mut results = Box::pin(results);

    assert(&order1, &mut results).await;
    assert(&order2, &mut results).await;
    assert(&order3, &mut results).await;
}
