// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use async_channel::unbounded;
use fluxion_core::Timestamped;
use fluxion_stream::prelude::*;
use fluxion_test_utils::{helpers::unwrap_stream, sequenced::Sequenced};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Value {
    Int(i32),
    Str(String),
}

#[tokio::test]
async fn test_combine_latest_int_string_filter_order() -> anyhow::Result<()> {
    // Arrange
    let (tx_int, rx_int) = unbounded::<Sequenced<Value>>();
    let (tx_str, rx_str) = unbounded::<Sequenced<Value>>();

    let int_stream = rx_int.into_fluxion_stream();
    let str_stream = rx_str.into_fluxion_stream();

    let mut pipeline = int_stream
        .combine_latest(vec![str_stream], |_| true)
        .filter_ordered(|combined| matches!(combined.values()[0], Value::Int(x) if x > 50));

    // Act
    tx_str.try_send(Sequenced::with_timestamp(Value::Str("initial".into()), 1))?;
    tx_int.try_send(Sequenced::with_timestamp(Value::Int(30), 2))?;
    tx_int.try_send(Sequenced::with_timestamp(Value::Int(60), 3))?;
    tx_str.try_send(Sequenced::with_timestamp(Value::Str("updated".into()), 4))?;
    tx_int.try_send(Sequenced::with_timestamp(Value::Int(75), 5))?;

    // Assert
    let result1 = unwrap_stream(&mut pipeline, 500).await.unwrap();
    let state1 = result1.into_inner();
    let combined1 = state1.values();
    assert!(matches!(combined1[0], Value::Int(60)));
    assert!(matches!(combined1[1], Value::Str(ref s) if s == "initial"));

    let result2 = unwrap_stream(&mut pipeline, 500).await.unwrap();
    let state2 = result2.into_inner();
    let combined2 = state2.values();
    assert!(matches!(combined2[0], Value::Int(60)));
    assert!(matches!(combined2[1], Value::Str(ref s) if s == "updated"));

    let result3 = unwrap_stream(&mut pipeline, 500).await.unwrap();
    let state3 = result3.into_inner();
    let combined3 = state3.values();
    assert!(matches!(combined3[0], Value::Int(75)));
    assert!(matches!(combined3[1], Value::Str(ref s) if s == "updated"));

    Ok(())
}
