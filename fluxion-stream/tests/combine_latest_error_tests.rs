// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `combine_latest` operator.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::CombineLatestExt;
use fluxion_test_utils::{sequenced::Sequenced, test_channel_with_errors};
use futures::StreamExt;

#[tokio::test]
async fn test_combine_latest_propagates_error_from_primary_stream() -> anyhow::Result<()> {
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<i32>>();

    let mut combined = stream1.combine_latest(vec![stream2], |_| true);

    // Send initial values
    tx1.send(StreamItem::Value(Sequenced::with_sequence(1, 1)))?;
    tx2.send(StreamItem::Value(Sequenced::with_sequence(10, 2)))?;

    // First emission combines both values
    let result1 = combined.next().await.unwrap();
    assert!(matches!(result1, StreamItem::Value(_)));

    // Send error from primary stream
    tx1.send(StreamItem::Error(FluxionError::stream_error(
        "Primary error",
    )))?;

    // Should propagate error
    let result2 = combined.next().await.unwrap();
    assert!(matches!(result2, StreamItem::Error(_)));

    // Continue with more values
    tx1.send(StreamItem::Value(Sequenced::with_sequence(3, 5)))?;
    tx2.send(StreamItem::Value(Sequenced::with_sequence(20, 4)))?;

    // Should emit value after error
    let result3 = combined.next().await.unwrap();
    assert!(matches!(result3, StreamItem::Value(_)));

    drop(tx1);
    drop(tx2);

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_propagates_error_from_secondary_stream() -> anyhow::Result<()> {
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<i32>>();

    let mut combined = stream1.combine_latest(vec![stream2], |_| true);

    // Send initial values
    tx1.send(StreamItem::Value(Sequenced::with_sequence(1, 1)))?;
    tx2.send(StreamItem::Value(Sequenced::with_sequence(10, 3)))?;

    // First emission should succeed
    let result1 = combined.next().await.unwrap();
    assert!(matches!(result1, StreamItem::Value(_)));

    // Send error from secondary stream
    tx2.send(StreamItem::Error(FluxionError::stream_error(
        "Secondary error",
    )))?;

    // Should propagate error from secondary
    let result2 = combined.next().await.unwrap();
    assert!(matches!(result2, StreamItem::Error(_)));

    // Continue with more values
    tx2.send(StreamItem::Value(Sequenced::with_sequence(30, 5)))?;

    // Stream should continue after error
    let result3 = combined.next().await.unwrap();
    assert!(matches!(result3, StreamItem::Value(_)));

    drop(tx1);
    drop(tx2);

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_multiple_errors_from_different_streams() -> anyhow::Result<()> {
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<i32>>();

    let mut combined = stream1.combine_latest(vec![stream2], |_| true);

    // Send initial values
    tx1.send(StreamItem::Value(Sequenced::with_sequence(1, 1)))?;
    tx2.send(StreamItem::Value(Sequenced::with_sequence(10, 2)))?;

    let result1 = combined.next().await.unwrap();
    assert!(matches!(result1, StreamItem::Value(_)));

    // Error from first stream
    tx1.send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;

    let result2 = combined.next().await.unwrap();
    assert!(matches!(result2, StreamItem::Error(_)));

    // Error from second stream
    tx2.send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;

    let result3 = combined.next().await.unwrap();
    assert!(matches!(result3, StreamItem::Error(_)));

    // Continue with values
    tx1.send(StreamItem::Value(Sequenced::with_sequence(2, 3)))?;

    let result4 = combined.next().await.unwrap();
    assert!(matches!(result4, StreamItem::Value(_)));

    drop(tx1);
    drop(tx2);

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_error_at_start() -> anyhow::Result<()> {
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<i32>>();

    let mut combined = stream1.combine_latest(vec![stream2], |_| true);

    // Send error immediately
    tx1.send(StreamItem::Error(FluxionError::stream_error(
        "Immediate error",
    )))?;

    // First item should be the error
    let result1 = combined.next().await.unwrap();
    assert!(matches!(result1, StreamItem::Error(_)));

    // Send values
    tx1.send(StreamItem::Value(Sequenced::with_sequence(1, 1)))?;
    tx2.send(StreamItem::Value(Sequenced::with_sequence(10, 3)))?;

    // Should continue processing
    let result2 = combined.next().await.unwrap();
    assert!(matches!(result2, StreamItem::Value(_)));

    drop(tx1);
    drop(tx2);

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_filter_predicate_continues_after_error() -> anyhow::Result<()> {
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<i32>>();

    // Filter predicate should still be evaluated after error
    let mut combined = stream1.combine_latest(vec![stream2], |state| {
        state.values()[0] > 1 // Only pass values > 1
    });

    // Send initial values (won't pass filter)
    tx1.send(StreamItem::Value(Sequenced::with_sequence(1, 1)))?;
    tx2.send(StreamItem::Value(Sequenced::with_sequence(10, 4)))?;

    // Send error
    tx1.send(StreamItem::Error(FluxionError::stream_error(
        "Error in middle",
    )))?;

    let result1 = combined.next().await.unwrap();
    assert!(matches!(result1, StreamItem::Error(_)));

    // Send value that passes filter
    tx1.send(StreamItem::Value(Sequenced::with_sequence(2, 2)))?;

    let result2 = combined.next().await.unwrap();
    assert!(matches!(result2, StreamItem::Value(_)));

    // Send value that doesn't pass filter
    tx1.send(StreamItem::Value(Sequenced::with_sequence(0, 3)))?;

    // Send another value that passes
    tx1.send(StreamItem::Value(Sequenced::with_sequence(3, 5)))?;

    let result3 = combined.next().await.unwrap();
    assert!(matches!(result3, StreamItem::Value(_)));

    drop(tx1);
    drop(tx2);

    Ok(())
}
