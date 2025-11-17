// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_rx::FluxionStream;
use fluxion_test_utils::sequenced::Sequenced;
use futures::StreamExt;

#[tokio::test]
async fn test_take_latest_when_int_bool() {
    // Define enum to hold int and bool types
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum Value {
        Int(i32),
        Bool(bool),
    }

    // Create int stream and bool trigger stream
    let (tx_int, rx_int) = tokio::sync::mpsc::unbounded_channel::<Sequenced<Value>>();
    let (tx_trigger, rx_trigger) = tokio::sync::mpsc::unbounded_channel::<Sequenced<Value>>();

    let int_stream = FluxionStream::from_unbounded_receiver(rx_int);
    let trigger_stream = FluxionStream::from_unbounded_receiver(rx_trigger);

    let mut pipeline = int_stream.take_latest_when(trigger_stream, |_| true);

    // Send int values first - they will be buffered
    tx_int
        .send(Sequenced::with_sequence(Value::Int(10), 1))
        .unwrap();
    tx_int
        .send(Sequenced::with_sequence(Value::Int(20), 2))
        .unwrap();
    tx_int
        .send(Sequenced::with_sequence(Value::Int(30), 3))
        .unwrap();

    // Trigger with bool - should emit latest int value (30) with trigger's sequence
    tx_trigger
        .send(Sequenced::with_sequence(Value::Bool(true), 4))
        .unwrap();

    let result1 = pipeline.next().await.unwrap().unwrap();
    assert!(matches!(result1.get(), Value::Int(30)));
    assert_eq!(result1.sequence(), 4);

    // After first trigger, send more int values
    tx_int
        .send(Sequenced::with_sequence(Value::Int(40), 5))
        .unwrap();

    // Need another trigger to emit the buffered value
    tx_trigger
        .send(Sequenced::with_sequence(Value::Bool(true), 6))
        .unwrap();

    let result2 = pipeline.next().await.unwrap().unwrap();
    assert!(matches!(result2.get(), Value::Int(40)));
    assert_eq!(result2.sequence(), 6);

    // Send another int and trigger
    tx_int
        .send(Sequenced::with_sequence(Value::Int(50), 7))
        .unwrap();

    tx_trigger
        .send(Sequenced::with_sequence(Value::Bool(true), 8))
        .unwrap();

    let result3 = pipeline.next().await.unwrap().unwrap();
    assert!(matches!(result3.get(), Value::Int(50)));
    assert_eq!(result3.sequence(), 8);
}
