use crate::select_all::common::{Order, assert, send};
use fluxion::select_all::select_all_ordered::SelectAllExt;
use fluxion::sequenced_channel::unbounded_channel;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;

async fn test(order1: Order, order2: Order, order3: Order) {
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
    send(order1.clone(), senders.clone());
    send(order2.clone(), senders.clone());
    send(order3.clone(), senders.clone());

    // Assert
    let mut results = Box::pin(results);

    assert(order1, &mut results).await;
    assert(order2, &mut results).await;
    assert(order3, &mut results).await;
}

#[tokio::test]
async fn test_all_combinations() {
    test(Order::Person, Order::Animal, Order::Plant).await;
    test(Order::Person, Order::Plant, Order::Animal).await;
    test(Order::Plant, Order::Animal, Order::Person).await;
    test(Order::Plant, Order::Person, Order::Animal).await;
    test(Order::Animal, Order::Person, Order::Plant).await;
    test(Order::Animal, Order::Plant, Order::Person).await;
}

#[tokio::test]
async fn test_simple_string_events() {
    // Create sequenced channels for different types of events
    let (tx1, rx1) = unbounded_channel();
    let (tx2, rx2) = unbounded_channel();
    let (tx3, rx3) = unbounded_channel();

    // Convert to streams
    let stream1 = UnboundedReceiverStream::new(rx1.into_inner());
    let stream2 = UnboundedReceiverStream::new(rx2.into_inner());
    let stream3 = UnboundedReceiverStream::new(rx3.into_inner());

    // Merge streams with temporal ordering
    let mut merged = vec![stream1, stream2, stream3].select_all_ordered();

    // Send events - sequence numbers are automatically assigned
    // Note: Sending to different streams in a specific temporal order
    tx2.send("First event").unwrap(); // Sent first (to stream2)
    tx1.send("Second event").unwrap(); // Sent second (to stream1)
    tx3.send("Third event").unwrap(); // Sent third (to stream3)

    // Events come out in send order, not stream order!
    assert_eq!(merged.next().await.unwrap().value, "First event"); // Sent first
    assert_eq!(merged.next().await.unwrap().value, "Second event"); // Sent second
    assert_eq!(merged.next().await.unwrap().value, "Third event"); // Sent third
}

#[tokio::test]
async fn test_custom_struct_events() {
    #[derive(Debug, PartialEq, Eq)]
    struct MyEvent {
        user_id: u64,
        action: String,
    }

    let (tx1, rx1) = unbounded_channel();
    let (tx2, rx2) = unbounded_channel();

    let stream1 = UnboundedReceiverStream::new(rx1.into_inner());
    let stream2 = UnboundedReceiverStream::new(rx2.into_inner());

    // Merge and verify temporal order
    let mut merged = vec![stream1, stream2].select_all_ordered();

    // Send events to different streams in interleaved order
    tx1.send(MyEvent {
        user_id: 1,
        action: "login".to_string(),
    })
    .unwrap();

    tx2.send(MyEvent {
        user_id: 2,
        action: "view_page".to_string(),
    })
    .unwrap();

    tx1.send(MyEvent {
        user_id: 1,
        action: "logout".to_string(),
    })
    .unwrap();

    let event1 = merged.next().await.unwrap();
    assert_eq!(event1.value.user_id, 1);
    assert_eq!(event1.value.action, "login");

    let event2 = merged.next().await.unwrap();
    assert_eq!(event2.value.user_id, 2);
    assert_eq!(event2.value.action, "view_page");

    let event3 = merged.next().await.unwrap();
    assert_eq!(event3.value.user_id, 1);
    assert_eq!(event3.value.action, "logout");
}

#[tokio::test]
async fn test_deref_access() {
    let (tx, rx) = unbounded_channel();
    let stream = UnboundedReceiverStream::new(rx.into_inner());

    let mut merged = vec![stream].select_all_ordered();

    tx.send("Hello World").unwrap();

    let seq_value = merged.next().await.unwrap();

    // Test Deref access
    assert_eq!(seq_value.len(), 11); // Derefs to &str
    assert_eq!(*seq_value, "Hello World");

    // Test direct field access
    assert_eq!(seq_value.value, "Hello World");
}

#[tokio::test]
async fn test_sequenced_channel_invisible() {
    // Create sequenced channels - like regular mpsc but with automatic sequencing
    let (tx1, rx1) = unbounded_channel();
    let (tx2, rx2) = unbounded_channel();
    let (tx3, rx3) = unbounded_channel();

    // Convert to streams
    let stream1 = UnboundedReceiverStream::new(rx1.into_inner());
    let stream2 = UnboundedReceiverStream::new(rx2.into_inner());
    let stream3 = UnboundedReceiverStream::new(rx3.into_inner());

    // Merge streams with temporal ordering
    let mut merged = vec![stream1, stream2, stream3].select_all_ordered();

    // Users just send regular values - no Sequenced wrapper needed!
    tx2.send("First event").unwrap(); // Sent first (to stream2)
    tx1.send("Second event").unwrap(); // Sent second (to stream1)
    tx3.send("Third event").unwrap(); // Sent third (to stream3)

    // Events come out in send order, accessing the value is transparent
    assert_eq!(merged.next().await.unwrap().value, "First event");
    assert_eq!(merged.next().await.unwrap().value, "Second event");
    assert_eq!(merged.next().await.unwrap().value, "Third event");
}
