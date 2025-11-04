use crate::select_all::common::{Order, assert, send};
use fluxion::select_all::select_all_ordered::SelectAllExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::{ReceiverStream, UnboundedReceiverStream};

const BUFFER_SIZE: usize = 10;

async fn test(order1: Order, order2: Order, order3: Order) {
    // Arrange
    let (person_sender, person_receiver) = mpsc::unbounded_channel();
    let (animal_sender, animal_receiver) = mpsc::unbounded_channel();
    let (plant_sender, plant_receiver) = mpsc::unbounded_channel();

    let person_stream = UnboundedReceiverStream::new(person_receiver);
    let animal_stream = UnboundedReceiverStream::new(animal_receiver);
    let plant_stream = UnboundedReceiverStream::new(plant_receiver);

    let senders = vec![person_sender, animal_sender, plant_sender];
    let streams = vec![person_stream, animal_stream, plant_stream];

    send(order1.clone(), senders.clone());
    send(order2.clone(), senders.clone());
    send(order3.clone(), senders.clone());

    let results = streams.select_all_ordered();
    let mut results = Box::pin(results);

    assert(order1, &mut results).await;
    assert(order2, &mut results).await;
    assert(order3, &mut results).await;
}

#[tokio::test]
async fn test_all_combinations() {
    test(Order::Person, Order::Animal, Order::Plant).await;
    test(Order::Person, Order::Plant, Order::Animal).await;
    //test(Order::Plant, Order::Animal, Order::Person).await;
    //test(Order::Plant, Order::Person, Order::Animal).await;
    //test(Order::Animal, Order::Person, Order::Plant).await;
    //test(Order::Animal, Order::Plant, Order::Person).await;
}
