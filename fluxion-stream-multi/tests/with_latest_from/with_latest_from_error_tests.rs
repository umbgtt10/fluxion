// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `with_latest_from` operator.

use fluxion_core::HasTimestamp;
use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream_multi::{CombinedState, WithLatestFromExt};
use fluxion_test_utils::test_wrapper::TestWrapper;
use fluxion_test_utils::{
    assert_no_element_emitted, test_channel_with_errors, unwrap_stream, Sequenced,
};

#[tokio::test]
async fn test_with_latest_from_propagates_primary_error() -> anyhow::Result<()> {
    // Arrange
    let (primary_tx, primary_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (secondary_tx, secondary_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = primary_stream
        .with_latest_from(secondary_stream, |state: &CombinedState<i32, u64>| {
            TestWrapper::new(true, state.timestamp())
        });

    // Act & Assert: Send secondary first (required for with_latest_from)
    secondary_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;
    assert_no_element_emitted(&mut result, 100).await;
    primary_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 2)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Send error in primary
    primary_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Primary error",
    )))?;
    assert!(
        matches!(unwrap_stream(&mut result, 100).await, StreamItem::Error(_)),
        "Should propagate error from primary stream"
    );

    // Continue with more values
    primary_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(3, 4)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(primary_tx);
    drop(secondary_tx);

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_propagates_secondary_error() -> anyhow::Result<()> {
    // Arrange
    let (primary_tx, primary_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (secondary_tx, secondary_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = primary_stream
        .with_latest_from(secondary_stream, |state: &CombinedState<i32, u64>| {
            TestWrapper::new(true, state.timestamp())
        });

    // Act: Send secondary value first
    secondary_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;
    assert_no_element_emitted(&mut result, 100).await;
    primary_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 2)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Send error in secondary
    secondary_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Secondary error",
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Update secondary with new value
    secondary_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(30, 5)))?;
    assert_no_element_emitted(&mut result, 100).await;
    primary_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 6)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(primary_tx);
    drop(secondary_tx);

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_error_before_secondary_ready() -> anyhow::Result<()> {
    // Arrange
    let (primary_tx, primary_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (secondary_tx, secondary_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = primary_stream
        .with_latest_from(secondary_stream, |state: &CombinedState<i32, u64>| {
            TestWrapper::new(true, state.timestamp())
        });

    // Act: Send error in primary before secondary has value
    primary_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Early error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    drop(primary_tx);
    drop(secondary_tx);

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_selector_continues_after_error() -> anyhow::Result<()> {
    // Arrange
    let (primary_tx, primary_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (secondary_tx, secondary_stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Custom selector - pass through combined state
    let mut result = primary_stream.with_latest_from(secondary_stream, |combined| combined.clone());

    // Act & Assert: Send secondary first
    secondary_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(100, 1)))?;
    assert_no_element_emitted(&mut result, 100).await;
    primary_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 2)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Send error in primary
    primary_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Update secondary
    secondary_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(200, 4)))?;
    assert_no_element_emitted(&mut result, 100).await;
    primary_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(3, 5)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(primary_tx);
    drop(secondary_tx);

    Ok(())
}
