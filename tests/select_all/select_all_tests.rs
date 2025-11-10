use crate::select_all::common::{Order, assert, send};
use fluxion::sequenced_channel::unbounded_channel;
use futures::stream::select_all;
use tokio_stream::wrappers::UnboundedReceiverStream;

async fn select_all_template_test(order1: Order, order2: Order, order3: Order) {
    // Arrange
    let (person_sender, person_receiver) = unbounded_channel();
    let (animal_sender, animal_receiver) = unbounded_channel();
    let (plant_sender, plant_receiver) = unbounded_channel();

    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let senders = vec![person_sender, animal_sender, plant_sender];
    let streams = vec![person_stream, animal_stream, plant_stream];

    let results = select_all(streams);

    send(order1.clone(), senders.clone());
    send(order2.clone(), senders.clone());
    send(order3.clone(), senders.clone());

    let mut results = Box::pin(results);

    assert(order1, &mut results).await;
    assert(order2, &mut results).await;
    assert(order3, &mut results).await;
}

#[should_panic]
#[tokio::test]
async fn test_select_all_all_combinations() {
    /*

       These tests fail becuase select_all does not guarantee order of messages
       across multiple streams. Use select_all_ordered for that functionality (and pay the price!)

    */

    select_all_template_test(Order::Person, Order::Animal, Order::Plant).await;
    select_all_template_test(Order::Person, Order::Plant, Order::Animal).await;
    select_all_template_test(Order::Plant, Order::Animal, Order::Person).await;
    select_all_template_test(Order::Plant, Order::Person, Order::Animal).await;
    select_all_template_test(Order::Animal, Order::Person, Order::Plant).await;
    select_all_template_test(Order::Animal, Order::Plant, Order::Person).await;
}
