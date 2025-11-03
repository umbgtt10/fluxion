use futures::stream::select_all;
use std::f64::consts::PI;
use tokio::sync::mpsc;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};

#[derive(PartialEq, Debug)]
enum StreamValue {
    Integer { value: i32 },
    Float { value: f64 },
    Text { value: String },
}

#[derive(Debug, Clone)]
pub enum Order {
    Integer,
    Float,
    Text,
}

async fn test(order1: Order, order2: Order, order3: Order) {
    // Arrange
    let (int_sender, receiver) = mpsc::unbounded_channel();
    let (float_sender, float_receiver) = mpsc::unbounded_channel();
    let (text_sender, text_receiver) = mpsc::unbounded_channel();

    let int_stream = UnboundedReceiverStream::new(receiver);
    let float_stream = UnboundedReceiverStream::new(float_receiver);
    let text_stream = UnboundedReceiverStream::new(text_receiver);

    let senders = vec![int_sender, float_sender, text_sender];
    let streams = vec![int_stream, float_stream, text_stream];

    let mut results = select_all(streams);

    send(order1.clone(), &senders);
    send(order2.clone(), &senders);
    send(order3.clone(), &senders);

    assert(order1, &mut results).await;
    assert(order2, &mut results).await;
    assert(order3, &mut results).await;
}

fn send(order: Order, senders: &[mpsc::UnboundedSender<StreamValue>]) {
    match order {
        Order::Integer => senders[0].send(StreamValue::Integer { value: 42 }).unwrap(),
        Order::Float => senders[1].send(StreamValue::Float { value: PI }).unwrap(),
        Order::Text => senders[2]
            .send(StreamValue::Text {
                value: "Hello".to_string(),
            })
            .unwrap(),
    }
}

async fn assert(order: Order, results: impl futures::Stream<Item = StreamValue> + Unpin) {
    let state = Box::pin(results).next().await.unwrap();
    match order {
        Order::Integer => assert_eq!(state, StreamValue::Integer { value: 42 }),
        Order::Float => assert_eq!(state, StreamValue::Float { value: PI }),
        Order::Text => assert_eq!(
            state,
            StreamValue::Text {
                value: "Hello".to_string()
            }
        ),
    }
}

#[tokio::test]
async fn test_all_combinations() {
    test(Order::Integer, Order::Float, Order::Text).await;

    // These tests fail!
    //test(Order::Integer, Order::Text, Order::Float).await;
    //test(Order::Text, Order::Integer, Order::Float).await;
    //test(Order::Text, Order::Float, Order::Integer).await;
    //test(Order::Float, Order::Integer, Order::Text).await;
    //test(Order::Float, Order::Text, Order::Integer).await;
}
