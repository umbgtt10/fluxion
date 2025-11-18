// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for composed stream operations.

use fluxion_core::{FluxionError, Ordered, StreamItem};
use fluxion_stream::{CombineLatestExt, EmitWhenExt, FluxionStream, TakeLatestWhenExt};
use fluxion_test_utils::{sequenced::Sequenced, test_channel_with_errors};
use futures::StreamExt;

#[tokio::test]
async fn test_error_propagation_through_multiple_operators() -> anyhow::Result<()> {
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Chain multiple operators
    let mut result = FluxionStream::new(stream)
        .filter_ordered(|x| *x > 1) // Filter out first item
        .combine_with_previous()
        .map_ordered(|x| x.current.get() * 10);

    // Send value (filtered out)
    tx.send(StreamItem::Value(Sequenced::with_sequence(1, 1)))?;

    // Send value (passes)
    tx.send(StreamItem::Value(Sequenced::with_sequence(2, 2)))?;

    let item1 = result.next().await.unwrap();
    assert!(matches!(item1, StreamItem::Value(20)));

    // Send error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;

    let item2 = result.next().await.unwrap();
    assert!(matches!(item2, StreamItem::Error(_)));

    // Continue
    tx.send(StreamItem::Value(Sequenced::with_sequence(4, 4)))?;

    let item3 = result.next().await.unwrap();
    assert!(matches!(item3, StreamItem::Value(40)));

    tx.send(StreamItem::Value(Sequenced::with_sequence(5, 5)))?;

    let item4 = result.next().await.unwrap();
    assert!(matches!(item4, StreamItem::Value(50)));

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_error_in_long_operator_chain() -> anyhow::Result<()> {
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Long chain: filter -> combine_with_previous -> map
    let mut result = FluxionStream::new(stream)
        .filter_ordered(|x| *x >= 10)
        .combine_with_previous()
        .map_ordered(|x| x.current.get() + 5);

    // Send value
    tx.send(StreamItem::Value(Sequenced::with_sequence(10, 1)))?;
    let item1 = result.next().await.unwrap();
    assert!(matches!(item1, StreamItem::Value(15)));

    // Send error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;

    let item2 = result.next().await.unwrap();
    assert!(matches!(item2, StreamItem::Error(_)));

    // Continue
    tx.send(StreamItem::Value(Sequenced::with_sequence(30, 3)))?;

    let item3 = result.next().await.unwrap();
    assert!(matches!(item3, StreamItem::Value(35)));

    tx.send(StreamItem::Value(Sequenced::with_sequence(40, 4)))?;

    let item4 = result.next().await.unwrap();
    assert!(matches!(item4, StreamItem::Value(45)));

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_multiple_errors_through_composition() -> anyhow::Result<()> {
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<i32>>();

    // Combine then transform
    let mut result = stream1
        .combine_latest(vec![stream2], |_| true)
        .combine_with_previous()
        .map_ordered(|x| {
            let state = x.current.get();
            format!("Combined: {:?}", state.values())
        });

    // Send initial values
    tx1.send(StreamItem::Value(Sequenced::with_sequence(1, 1)))?;
    tx2.send(StreamItem::Value(Sequenced::with_sequence(10, 4)))?;

    let item1 = result.next().await.unwrap();
    assert!(matches!(item1, StreamItem::Value(ref s) if s.contains("Combined")));

    // Send error
    tx1.send(StreamItem::Error(FluxionError::stream_error("Error")))?;

    let item2 = result.next().await.unwrap();
    assert!(matches!(item2, StreamItem::Error(_)));

    // Continue
    tx1.send(StreamItem::Value(Sequenced::with_sequence(3, 3)))?;

    let item3 = result.next().await.unwrap();
    assert!(matches!(item3, StreamItem::Value(ref s) if s.contains("Combined")));

    drop(tx1);
    drop(tx2);

    Ok(())
}

#[tokio::test]
async fn test_error_recovery_in_composed_streams() -> anyhow::Result<()> {
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Complex composition
    let mut result = source_stream
        .take_latest_when(trigger_stream, |_| true)
        .combine_with_previous()
        .map_ordered(|x| *x.current.get());

    // Send source values
    source_tx.send(StreamItem::Value(Sequenced::with_sequence(5, 1)))?;

    // Send error
    source_tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;

    let item1 = result.next().await.unwrap();
    assert!(matches!(item1, StreamItem::Error(_)));

    // Continue
    source_tx.send(StreamItem::Value(Sequenced::with_sequence(15, 3)))?;
    trigger_tx.send(StreamItem::Value(Sequenced::with_sequence(100, 5)))?;

    let item2 = result.next().await.unwrap();
    assert!(matches!(item2, StreamItem::Value(15)));

    drop(source_tx);
    drop(trigger_tx);
    Ok(())
}

#[tokio::test]
async fn test_error_with_emit_when_composition() -> anyhow::Result<()> {
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    // emit_when + map composition
    let mut result = source_stream
        .emit_when(filter_stream, |state| state.values()[0] > state.values()[1])
        .combine_with_previous()
        .map_ordered(|x| x.current.get() * 2);

    // Send filter value first
    filter_tx.send(StreamItem::Value(Sequenced::with_sequence(10, 1)))?;

    // Send source value (doesn't pass predicate)
    source_tx.send(StreamItem::Value(Sequenced::with_sequence(5, 2)))?;

    // Send error
    source_tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;

    let item1 = result.next().await.unwrap();
    assert!(matches!(item1, StreamItem::Error(_)));

    // Send value that passes
    filter_tx.send(StreamItem::Value(Sequenced::with_sequence(20, 4)))?;
    source_tx.send(StreamItem::Value(Sequenced::with_sequence(25, 3)))?;

    let item2 = result.next().await.unwrap();
    assert!(matches!(item2, StreamItem::Value(50)));

    drop(source_tx);
    drop(filter_tx);

    Ok(())
}
