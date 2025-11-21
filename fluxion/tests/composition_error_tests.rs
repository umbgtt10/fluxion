// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0
//! Error propagation tests for composed stream operations.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::{CombineLatestExt, EmitWhenExt, FluxionStream, TakeLatestWhenExt};
use fluxion_test_utils::{test_channel_with_errors, unwrap_stream, Sequenced};

#[tokio::test]
async fn test_error_propagation_through_multiple_operators() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Chain multiple operators
    let mut result = FluxionStream::new(stream)
        .filter_ordered(|x| *x > 1) // Filter out first item
        .combine_with_previous()
        .map_ordered(|x| x.current.value * 10);

    // Act & Assert Send value (filtered out)
    tx.send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;

    // Send value (passes)
    tx.send(StreamItem::Value(Sequenced::with_timestamp(2, 2)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(20)
    ));

    // Send error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Continue
    tx.send(StreamItem::Value(Sequenced::with_timestamp(4, 4)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(40)
    ));

    tx.send(StreamItem::Value(Sequenced::with_timestamp(5, 5)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(50)
    ));

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_error_in_long_operator_chain() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Long chain: filter -> combine_with_previous -> map
    let mut result = FluxionStream::new(stream)
        .filter_ordered(|x| *x >= 10)
        .combine_with_previous()
        .map_ordered(|x| x.current.value + 5);

    // Act & Assert Send value
    tx.send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(15)
    ));

    // Send error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Continue
    tx.send(StreamItem::Value(Sequenced::with_timestamp(30, 3)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(35)
    ));

    tx.send(StreamItem::Value(Sequenced::with_timestamp(40, 4)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(45)
    ));

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_multiple_errors_through_composition() -> anyhow::Result<()> {
    // ASrrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<i32>>();

    // Combine then transform
    let mut result = stream1
        .combine_latest(vec![stream2], |_| true)
        .combine_with_previous()
        .map_ordered(|x| {
            let state = &x.current;
            format!("Combined: {:?}", state.values())
        });

    // Send initial values
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    tx2.send(StreamItem::Value(Sequenced::with_timestamp(10, 4)))?;
    assert!(
        matches!(unwrap_stream(&mut result, 100).await, StreamItem::Value(ref s) if s.contains("Combined"))
    );

    // Send error
    tx1.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Continue
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(3, 3)))?;
    assert!(
        matches!(unwrap_stream(&mut result, 100).await, StreamItem::Value(ref s) if s.contains("Combined"))
    );

    drop(tx1);
    drop(tx2);

    Ok(())
}

#[tokio::test]
async fn test_error_recovery_in_composed_streams() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Complex composition
    let mut result = source_stream
        .take_latest_when(trigger_stream, |_| true)
        .combine_with_previous()
        .map_ordered(|x| x.current.value);

    // Send source values
    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(5, 1)))?;

    // Send error
    source_tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Continue
    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(15, 3)))?;
    trigger_tx.send(StreamItem::Value(Sequenced::with_timestamp(100, 5)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(15)
    ));

    drop(source_tx);
    drop(trigger_tx);
    Ok(())
}

#[tokio::test]
async fn test_error_with_emit_when_composition() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    // emit_when + map composition
    let mut result = source_stream
        .emit_when(filter_stream, |state| state.values()[0] > state.values()[1])
        .combine_with_previous()
        .map_ordered(|x| x.current.value * 2);

    // Arrange & Act Send filter value first
    filter_tx.send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;
    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(5, 2)))?;
    source_tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;

    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Send value that passes
    filter_tx.send(StreamItem::Value(Sequenced::with_timestamp(20, 4)))?;
    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(25, 3)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(50)
    ));

    drop(source_tx);
    drop(filter_tx);

    Ok(())
}
