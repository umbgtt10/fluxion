use futures::Stream;
use std::f64::consts::PI;
use stream_processing::merge_ordered::VecStreamMergeExt;
use tokio::sync::mpsc;
use tokio::task::yield_now;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};

#[derive(PartialEq, Debug, Clone)]
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
    let (int_sender, int_receiver) = mpsc::unbounded_channel();
    let (float_sender, float_receiver) = mpsc::unbounded_channel();
    let (text_sender, text_receiver) = mpsc::unbounded_channel();

    let int_stream = UnboundedReceiverStream::new(int_receiver);
    let float_stream = UnboundedReceiverStream::new(float_receiver);
    let text_stream = UnboundedReceiverStream::new(text_receiver);

    let senders = vec![int_sender, float_sender, text_sender];
    let streams = vec![int_stream, float_stream, text_stream];

    let mut results = streams.merge_ordered().await;

    // Act
    yield_now().await;
    send(order1.clone(), &senders);
    yield_now().await;
    send(order2.clone(), &senders);
    yield_now().await;
    send(order3.clone(), &senders);

    // Assert
    assert_order(order1, &mut results).await;
    assert_order(order2, &mut results).await;
    assert_order(order3, &mut results).await;
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

async fn assert_order(order: Order, results: &mut (impl Stream<Item = StreamValue> + Unpin)) {
    let state = results.next().await.unwrap();
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
    test(Order::Text, Order::Integer, Order::Float).await;
    test(Order::Text, Order::Float, Order::Integer).await;
    test(Order::Integer, Order::Text, Order::Float).await;
    test(Order::Integer, Order::Float, Order::Text).await;
    test(Order::Float, Order::Text, Order::Integer).await;
    test(Order::Float, Order::Integer, Order::Text).await;
}
