// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Error propagation tests for `emit_when` operator.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::EmitWhenExt;
use fluxion_test_utils::{test_channel_with_errors, unwrap_stream, Sequenced};
use futures::StreamExt;

#[tokio::test]
async fn test_emit_when_propagates_source_error() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result =
        source_stream.emit_when(filter_stream, |state| state.values()[0] > state.values()[1]);

    // Act
    filter_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(15, 1)))?;

    // Send source value
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 2)))?;

    // Send error in source
    source_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Source error",
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Continue with value
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(30, 3)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

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
    filter_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(5, 1)))?;

    // Send source value
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 2)))?;

    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Send error in filter
    filter_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Filter error",
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    filter_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(15, 4)))?;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(20, 3)))?;

    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

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
    filter_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(15, 1)))?;

    // Send source value (doesn't pass predicate)
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 2)))?;

    // Send error in source
    source_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Send value that passes predicate
    filter_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(25, 4)))?;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(30, 5)))?;

    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

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
    filter_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(5, 1)))?;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 2)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Error from source
    source_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Source error",
    )))?;

    let result2 = result.next().await.unwrap();
    assert!(matches!(result2, StreamItem::Error(_)));

    // Error from filter
    filter_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
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
    source_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Early error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Continue
    filter_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(5, 2)))?;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(source_tx);
    drop(filter_tx);

    Ok(())
}

#[tokio::test]
async fn test_emit_when_source_none_on_filter_update() -> anyhow::Result<()> {
    // This tests the branch where filter updates but source hasn't emitted yet
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result =
        source_stream.emit_when(filter_stream, |state| state.values()[0] > state.values()[1]);

    // Send filter value first (source is None)
    filter_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(5, 1)))?;

    // Now send source value
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 2)))?;

    // Now it should emit
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(source_tx);
    drop(filter_tx);

    Ok(())
}

#[tokio::test]
async fn test_emit_when_filter_returns_false_on_source_update() -> anyhow::Result<()> {
    // This tests the branch where source updates but filter condition is false
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result =
        source_stream.emit_when(filter_stream, |state| state.values()[0] > state.values()[1]);

    // Setup: filter value
    filter_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(50, 1)))?;

    // Source value less than filter - should NOT emit
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 2)))?;

    // Send another source that passes
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(100, 3)))?;

    // Only this should emit
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(source_tx);
    drop(filter_tx);

    Ok(())
}

#[tokio::test]
async fn test_emit_when_filter_returns_false_on_filter_update() -> anyhow::Result<()> {
    // This tests the branch where filter updates causing condition to become false
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result =
        source_stream.emit_when(filter_stream, |state| state.values()[0] > state.values()[1]);

    // Setup initial state: source=10, filter=5 (passes)
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;
    filter_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(5, 2)))?;

    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Update filter to higher value: filter=20 (fails: 10 > 20 = false)
    filter_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(20, 3)))?;

    // Update filter back to passing: filter=8 (passes: 10 > 8 = true)
    filter_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(8, 4)))?;

    // Should emit
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(source_tx);
    drop(filter_tx);

    Ok(())
}

#[tokio::test]
async fn test_emit_when_source_none_on_source_update() -> anyhow::Result<()> {
    // This tests an edge case that shouldn't normally occur, but ensures robustness
    // When source updates, it sets source to Some, so this tests the initial state
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result =
        source_stream.emit_when(filter_stream, |state| state.values()[0] > state.values()[1]);

    // Send source first (filter is None) - no emission
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;

    // Now send filter
    filter_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(5, 2)))?;

    // Should emit now
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(source_tx);
    drop(filter_tx);

    Ok(())
}

#[tokio::test]
async fn test_emit_when_multiple_filter_updates_no_source() -> anyhow::Result<()> {
    // Test multiple filter updates before source emits
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result =
        source_stream.emit_when(filter_stream, |state| state.values()[0] > state.values()[1]);

    // Send multiple filter values (source is None)
    filter_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(5, 1)))?;
    filter_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 2)))?;
    filter_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(3, 3)))?;

    // Now send source value - should use latest filter (3)
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(15, 4)))?;

    // Should emit (15 > 3)
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(source_tx);
    drop(filter_tx);

    Ok(())
}

#[tokio::test]
async fn test_emit_when_alternating_false_conditions() -> anyhow::Result<()> {
    // Test alternating between passing and failing conditions
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result =
        source_stream.emit_when(filter_stream, |state| state.values()[0] > state.values()[1]);

    // Setup
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;
    filter_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(5, 2)))?;

    // Should emit (10 > 5)
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Update source lower
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(3, 3)))?;

    // Should NOT emit (3 > 5 = false)
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Update source higher
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(20, 4)))?;

    // Should emit (20 > 5)
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(source_tx);
    drop(filter_tx);

    Ok(())
}
