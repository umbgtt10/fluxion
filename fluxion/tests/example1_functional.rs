// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_rx::FluxionStream;
use fluxion_test_utils::{sequenced::Sequenced, unwrap_stream};
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
    tx_int.send(Sequenced::with_sequence(Value::Int(10), 1))?;
    tx_int.send(Sequenced::with_sequence(Value::Int(20), 2))?;
    tx_int.send(Sequenced::with_sequence(Value::Int(30), 3))?;

    // Trigger with bool - should emit latest int value (30) with trigger's sequence
    tx_trigger.send(Sequenced::with_sequence(Value::Bool(true), 4))?;

    let result1 = unwrap_stream(&mut pipeline, 500).await.unwrap();
    assert!(matches!(result1.get(), Value::Int(30)));
    assert_eq!(result1.sequence(), 4);

    // After first trigger, send more int values
    tx_int.send(Sequenced::with_sequence(Value::Int(40), 5))?;

    // Need another trigger to emit the buffered value
    tx_trigger.send(Sequenced::with_sequence(Value::Bool(true), 6))?;

    let result2 = unwrap_stream(&mut pipeline, 500).await.unwrap();
    assert!(matches!(result2.get(), Value::Int(40)));
    assert_eq!(result2.sequence(), 6);

    // Send another int and trigger
    tx_int.send(Sequenced::with_sequence(Value::Int(50), 7))?;
    tx_trigger.send(Sequenced::with_sequence(Value::Bool(true), 8))?;

    let result3 = unwrap_stream(&mut pipeline, 500).await.unwrap();
    assert!(matches!(result3.get(), Value::Int(50)));
    assert_eq!(result3.sequence(), 8);

    Ok(())
}
