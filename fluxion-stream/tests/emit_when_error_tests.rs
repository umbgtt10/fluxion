// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `emit_when` operator.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::EmitWhenExt;
use fluxion_test_utils::{sequenced::Sequenced, test_channel_with_errors, unwrap_stream};
use futures::StreamExt;

#[tokio::test]
async fn test_emit_when_propagates_source_error() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result =
        source_stream.emit_when(filter_stream, |state| state.values()[0] > state.values()[1]);

    // Act
    filter_tx.send(StreamItem::Value(Sequenced::with_sequence(15, 1)))?;

    // Send source value
    source_tx.send(StreamItem::Value(Sequenced::with_sequence(10, 2)))?;

    // Send error in source
    source_tx.send(StreamItem::Error(FluxionError::stream_error(
        "Source error",
    )))?;

    // Assert
    assert!(matches!(unwrap_stream(&mut result, 100).await, StreamItem::Error(_)));

    // Continue with value
    source_tx.send(StreamItem::Value(Sequenced::with_sequence(30, 3)))?;
    assert!(matches!(unwrap_stream(&mut result, 100).await, StreamItem::Value(_)));

    drop(source_tx);
    drop(filter_tx);

    Ok(())
}

#[tokio::test]
async fn test_emit_when_propagates_filter_error() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result =
        source_stream.emit_when(filter_stream, |state| state.values()[0] > state.values()[1]);

    // Act
    filter_tx.send(StreamItem::Value(Sequenced::with_sequence(5, 1)))?;

    // Send source value
    source_tx.send(StreamItem::Value(Sequenced::with_sequence(10, 2)))?;

    assert!(matches!(unwrap_stream(&mut result, 100).await, StreamItem::Value(_)));

    // Send error in filter
    filter_tx.send(StreamItem::Error(FluxionError::stream_error(
        "Filter error",
    )))?;

    // Assert
    assert!(matches!(unwrap_stream(&mut result, 100).await, StreamItem::Error(_)));

    filter_tx.send(StreamItem::Value(Sequenced::with_sequence(15, 4)))?;
    source_tx.send(StreamItem::Value(Sequenced::with_sequence(20, 3)))?;

    assert!(matches!(unwrap_stream(&mut result, 100).await, StreamItem::Value(_)));

    drop(source_tx);
    drop(filter_tx);

    Ok(())
}

#[tokio::test]
async fn test_emit_when_predicate_continues_after_error() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Only emit when source > filter
    let mut result =
        source_stream.emit_when(filter_stream, |state| state.values()[0] > state.values()[1]);

    // Send filter value
    filter_tx.send(StreamItem::Value(Sequenced::with_sequence(15, 1)))?;

    // Send source value (doesn't pass predicate)
    source_tx.send(StreamItem::Value(Sequenced::with_sequence(10, 2)))?;

    // Send error in source
    source_tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(unwrap_stream(&mut result, 100).await, StreamItem::Error(_)));

    // Send value that passes predicate
    filter_tx.send(StreamItem::Value(Sequenced::with_sequence(25, 4)))?;
    source_tx.send(StreamItem::Value(Sequenced::with_sequence(30, 5)))?;

    assert!(matches!(unwrap_stream(&mut result, 100).await, StreamItem::Value(_)));

    drop(source_tx);
    drop(filter_tx);

    Ok(())
}

#[tokio::test]
async fn test_emit_when_both_streams_have_errors() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result =
        source_stream.emit_when(filter_stream, |state| state.values()[0] > state.values()[1]);

    // Act & Assert: Send initial values
    filter_tx.send(StreamItem::Value(Sequenced::with_sequence(5, 1)))?;
    source_tx.send(StreamItem::Value(Sequenced::with_sequence(10, 2)))?;
    assert!(matches!(unwrap_stream(&mut result, 100).await, StreamItem::Value(_)));

    // Error from source
    source_tx.send(StreamItem::Error(FluxionError::stream_error(
        "Source error",
    )))?;

    let result2 = result.next().await.unwrap();
    assert!(matches!(result2, StreamItem::Error(_)));

    // Error from filter
    filter_tx.send(StreamItem::Error(FluxionError::stream_error(
        "Filter error",
    )))?;

    let result3 = result.next().await.unwrap();
    assert!(matches!(result3, StreamItem::Error(_)));

    drop(source_tx);
    drop(filter_tx);

    Ok(())
}

#[tokio::test]
async fn test_emit_when_error_before_filter_ready() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result =
        source_stream.emit_when(filter_stream, |state| state.values()[0] > state.values()[1]);

    // Act & Assert: Error immediately before filter has value
    source_tx.send(StreamItem::Error(FluxionError::stream_error("Early error")))?;
    assert!(matches!(unwrap_stream(&mut result, 100).await, StreamItem::Error(_)));

    // Continue
    filter_tx.send(StreamItem::Value(Sequenced::with_sequence(5, 2)))?;
    source_tx.send(StreamItem::Value(Sequenced::with_sequence(10, 1)))?;
    assert!(matches!(unwrap_stream(&mut result, 100).await, StreamItem::Value(_)));

    drop(source_tx);
    drop(filter_tx);

    Ok(())
}
