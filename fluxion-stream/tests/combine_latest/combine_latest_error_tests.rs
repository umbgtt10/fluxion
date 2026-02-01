// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `combine_latest` operator.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::CombineLatestExt;
use fluxion_test_utils::{
    helpers::{test_channel_with_errors, unwrap_stream},
    sequenced::Sequenced,
};

#[tokio::test]
async fn test_combine_latest_propagates_error_from_primary_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = stream1.combine_latest(vec![stream2], |_| true);

    // Act
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    tx2.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 2)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Act
    tx1.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Primary error",
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(3, 5)))?;
    tx2.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(20, 4)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(tx1);
    drop(tx2);

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_propagates_error_from_secondary_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = stream1.combine_latest(vec![stream2], |_| true);

    // Act
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    tx2.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 3)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Act
    tx2.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Secondary error",
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx2.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(30, 5)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(tx1);
    drop(tx2);

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_multiple_errors_from_different_streams() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = stream1.combine_latest(vec![stream2], |_| true);

    // Act
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    tx2.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 2)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Act
    tx1.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx2.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 3)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(tx1);
    drop(tx2);

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_error_at_start() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = stream1.combine_latest(vec![stream2], |_| true);

    // Act
    tx1.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Immediate error",
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    tx2.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 3)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(tx1);
    drop(tx2);

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_filter_predicate_continues_after_error() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<i32>>();

    // Filter predicate should still be evaluated after error
    let mut result = stream1.combine_latest(vec![stream2], |state| {
        state.values()[0] > 1 // Only pass values > 1
    });

    // Act
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    tx2.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 4)))?;

    // Act
    tx1.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Error in middle",
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 2)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Act
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(0, 3)))?;

    // Act
    tx1.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(3, 5)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(tx1);
    drop(tx2);

    Ok(())
}
