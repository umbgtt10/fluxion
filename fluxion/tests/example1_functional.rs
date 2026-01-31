// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use async_channel::unbounded;
use fluxion_core::HasTimestamp;
use fluxion_stream::prelude::*;
use fluxion_test_utils::{unwrap_stream, Sequenced};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Value {
    Int(i32),
    Bool(bool),
}

#[tokio::test]
async fn test_take_latest_when_int_bool() -> anyhow::Result<()> {
    // Arrange
    let (tx_int, rx_int) = unbounded::<Sequenced<Value>>();
    let (tx_trigger, rx_trigger) = unbounded::<Sequenced<Value>>();

    let int_stream = rx_int.into_fluxion_stream();
    let trigger_stream = rx_trigger.into_fluxion_stream();

    let mut pipeline = int_stream.take_latest_when(trigger_stream, |_| true);

    // Act
    tx_int.try_send(Sequenced::with_timestamp(Value::Int(10), 1))?;
    tx_int.try_send(Sequenced::with_timestamp(Value::Int(20), 2))?;
    tx_int.try_send(Sequenced::with_timestamp(Value::Int(30), 3))?;
    tx_trigger.try_send(Sequenced::with_timestamp(Value::Bool(true), 4))?;

    // Assert
    let result1 = unwrap_stream(&mut pipeline, 500).await.unwrap();
    assert!(matches!(&result1.value, Value::Int(30)));
    assert_eq!(result1.timestamp(), 4);

    // Act
    tx_int.try_send(Sequenced::with_timestamp(Value::Int(40), 5))?;
    tx_trigger.try_send(Sequenced::with_timestamp(Value::Bool(true), 6))?;

    // Assert
    let result2 = unwrap_stream(&mut pipeline, 500).await.unwrap();
    assert!(matches!(&result2.value, Value::Int(40)));
    assert_eq!(result2.timestamp(), 6);

    // Act
    tx_int.try_send(Sequenced::with_timestamp(Value::Int(50), 7))?;
    tx_trigger.try_send(Sequenced::with_timestamp(Value::Bool(true), 8))?;

    // Assert
    let result3 = unwrap_stream(&mut pipeline, 500).await.unwrap();
    assert!(matches!(&result3.value, Value::Int(50)));
    assert_eq!(result3.timestamp(), 8);

    Ok(())
}
