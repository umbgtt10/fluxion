// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::Timestamped;
use fluxion_rx::FluxionStream;
use fluxion_test_utils::{unwrap_stream, Sequenced};
use tokio::sync::mpsc::unbounded_channel;

#[tokio::test]
async fn test_take_latest_when_int_bool() -> anyhow::Result<()> {
    // Define enum to hold int and bool types
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum Value {
        Int(i32),
        Bool(bool),
    }

    // Create int stream and bool trigger stream
    let (tx_int, rx_int) = unbounded_channel::<Sequenced<Value>>();
    let (tx_trigger, rx_trigger) = unbounded_channel::<Sequenced<Value>>();

    let int_stream = FluxionStream::from_unbounded_receiver(rx_int);
    let trigger_stream = FluxionStream::from_unbounded_receiver(rx_trigger);

    let mut pipeline = int_stream.take_latest_when(trigger_stream, |_| true);

    // Send int values first - they will be buffered
    // Use realistic nanosecond timestamps
    tx_int.send(Sequenced::with_timestamp(Value::Int(10), 1))?; // 1 sec
    tx_int.send(Sequenced::with_timestamp(Value::Int(20), 2))?; // 2 sec
    tx_int.send(Sequenced::with_timestamp(Value::Int(30), 3))?; // 3 sec

    // Trigger with bool - should emit latest int value (30) with trigger's sequence
    tx_trigger.send(Sequenced::with_timestamp(Value::Bool(true), 4))?; // 4 sec

    let result1 = unwrap_stream(&mut pipeline, 500).await.unwrap();
    assert!(matches!(&result1.value, Value::Int(30)));
    assert_eq!(result1.timestamp(), 4);

    // After first trigger, send more int values
    tx_int.send(Sequenced::with_timestamp(Value::Int(40), 5))?; // 5 sec

    // Need another trigger to emit the buffered value
    tx_trigger.send(Sequenced::with_timestamp(Value::Bool(true), 6))?; // 6 sec

    let result2 = unwrap_stream(&mut pipeline, 500).await.unwrap();
    assert!(matches!(&result2.value, Value::Int(40)));
    assert_eq!(result2.timestamp(), 6);
    // Send another int and trigger
    tx_int.send(Sequenced::with_timestamp(Value::Int(50), 7))?; // 7 sec
    tx_trigger.send(Sequenced::with_timestamp(Value::Bool(true), 8))?; // 8 sec

    let result3 = unwrap_stream(&mut pipeline, 500).await.unwrap();
    assert!(matches!(&result3.value, Value::Int(50)));
    assert_eq!(result3.timestamp(), 8);
    Ok(())
}
