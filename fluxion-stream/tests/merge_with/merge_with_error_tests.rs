// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `merge_with` operator.
//!
//! The `merge_with` operator now handles `StreamItem<T>` directly and passes errors
//! through unchanged, allowing error handling at the stream consumer level.

use fluxion_core::{into_stream::IntoStream, FluxionError, StreamItem};
use fluxion_stream::MergedStream;
use fluxion_test_utils::{assert_stream_ended, test_channel_with_errors, unwrap_stream, Sequenced};
use futures::StreamExt;

#[tokio::test]
async fn test_merge_with_propagates_errors_from_first_stream() -> anyhow::Result<()> {
    // Arrange: merge_with doesn't directly handle StreamItem, so we filter_map errors
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<i32>>();

    let mut merged = MergedStream::seed::<Sequenced<i32>>(0)
        .merge_with(stream1, |value, state| {
            *state += value;
            *state
        })
        .merge_with(stream2, |value, state| {
            *state += value;
            *state
        });

    // Act: Send value from stream1
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(5, 1)))?;
    let StreamItem::Value(result) = merged.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(result.into_inner(), 5);

    // Send error - will pass through
    tx1.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    let StreamItem::Error(err) = merged.next().await.unwrap() else {
        panic!("Expected Error");
    };
    assert_eq!(err.to_string(), "Stream processing error: Error");

    // Send value from stream2 - should still work
    tx2.send(StreamItem::Value(Sequenced::with_timestamp(10, 2)))?;
    let StreamItem::Value(result) = merged.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(result.into_inner(), 15);

    drop(tx1);
    drop(tx2);

    Ok(())
}

#[tokio::test]
async fn test_merge_with_error_at_start_filtered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut merged = MergedStream::seed::<Sequenced<i32>>(0).merge_with(stream, |value, state| {
        *state += value;
        *state
    });

    // Act: Send error before any values - passes through
    tx.send(StreamItem::Error(FluxionError::stream_error("Early error")))?;
    let StreamItem::Error(err) = merged.next().await.unwrap() else {
        panic!("Expected Error");
    };
    assert_eq!(err.to_string(), "Stream processing error: Early error");

    // Send value after error
    tx.send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;

    // Assert: Stream should process value
    let StreamItem::Value(result) = merged.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(result.into_inner(), 10);

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_merge_with_multiple_streams_error_filtering() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<i32>>();

    let mut merged = MergedStream::seed::<Sequenced<i32>>(0)
        .merge_with(stream1, |value, state| {
            *state += value;
            *state
        })
        .merge_with(stream2, |value, state| {
            *state += value;
            *state
        });

    // Act: Send errors from both streams - should pass through
    tx1.send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    tx2.send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;

    // Errors should be emitted in order
    let StreamItem::Error(err1) = merged.next().await.unwrap() else {
        panic!("Expected Error");
    };
    let StreamItem::Error(err2) = merged.next().await.unwrap() else {
        panic!("Expected Error");
    };
    assert!(err1.to_string().contains("Error"));
    assert!(err2.to_string().contains("Error"));

    // Send values
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;
    let StreamItem::Value(result) = merged.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(result.into_inner(), 10);

    drop(tx1);
    drop(tx2);

    Ok(())
}

#[tokio::test]
async fn test_merge_with_errors_interleaved_with_values() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<i32>>();

    let mut merged = MergedStream::seed::<Sequenced<i32>>(0)
        .merge_with(stream1, |value, state| {
            *state += value;
            *state
        })
        .merge_with(stream2, |value, state| {
            *state += value;
            *state
        });

    // Act & Assert: Value
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(5, 1)))?;
    let StreamItem::Value(result) = merged.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(result.into_inner(), 5);

    // Error - passes through
    tx2.send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    let StreamItem::Error(_) = merged.next().await.unwrap() else {
        panic!("Expected Error");
    };

    // Value
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(10, 2)))?;
    let StreamItem::Value(result) = merged.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(result.into_inner(), 15);

    // Error - passes through
    tx1.send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;
    let StreamItem::Error(_) = merged.next().await.unwrap() else {
        panic!("Expected Error");
    };

    // Value
    tx2.send(StreamItem::Value(Sequenced::with_timestamp(20, 3)))?;
    let StreamItem::Value(result) = merged.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(result.into_inner(), 35);

    drop(tx1);
    drop(tx2);

    Ok(())
}

#[tokio::test]
async fn test_merge_with_state_preserved_despite_filtered_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut merged = MergedStream::seed::<Sequenced<i32>>(0).merge_with(stream, |value, state| {
        *state += value;
        *state
    });

    // Act: Process values to build state
    tx.send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;
    let StreamItem::Value(result) = merged.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(result.into_inner(), 10);

    tx.send(StreamItem::Value(Sequenced::with_timestamp(20, 2)))?;
    let StreamItem::Value(result) = merged.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(result.into_inner(), 30);

    // Send error - passes through
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    let StreamItem::Error(_) = merged.next().await.unwrap() else {
        panic!("Expected Error");
    };

    // Process more values - state should be preserved
    tx.send(StreamItem::Value(Sequenced::with_timestamp(5, 3)))?;
    let StreamItem::Value(result) = merged.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(result.into_inner(), 35);

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_merge_with_error_before_stream_ends() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut merged = MergedStream::seed::<Sequenced<i32>>(0).merge_with(stream, |value, state| {
        *state += value;
        *state
    });

    // Act: Send value
    tx.send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;
    let StreamItem::Value(result) = merged.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(result.into_inner(), 10);

    // Send error - passes through
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    let StreamItem::Error(_) = merged.next().await.unwrap() else {
        panic!("Expected Error");
    };

    // Drop transmitter to end stream
    drop(tx);

    // Assert: Stream should end cleanly
    assert_stream_ended(&mut merged, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_merge_with_empty_stream_with_only_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut merged = MergedStream::seed::<Sequenced<i32>>(0).merge_with(stream, |value, state| {
        *state += value;
        *state
    });

    // Act: Send only errors, no values - all pass through
    tx.send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    tx.send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;

    // Expect both errors
    let StreamItem::Error(_) = merged.next().await.unwrap() else {
        panic!("Expected Error 1");
    };
    let StreamItem::Error(_) = merged.next().await.unwrap() else {
        panic!("Expected Error 2");
    };

    drop(tx);

    // Stream should end after errors
    assert_stream_ended(&mut merged, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_merge_with_three_streams_with_filtered_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel_with_errors::<Sequenced<i32>>();
    let (tx3, stream3) = test_channel_with_errors::<Sequenced<i32>>();

    let mut merged = MergedStream::seed::<Sequenced<i32>>(0)
        .merge_with(stream1, |value, state| {
            *state += value;
            *state
        })
        .merge_with(stream2, |value, state| {
            *state += value;
            *state
        })
        .merge_with(stream3, |value, state| {
            *state += value;
            *state
        });

    // Act: Send values from all streams with errors interspersed
    tx1.send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;
    let StreamItem::Value(result) = merged.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(result.into_inner(), 10);

    tx2.send(StreamItem::Value(Sequenced::with_timestamp(20, 2)))?;
    let StreamItem::Value(result) = merged.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(result.into_inner(), 30);

    // Error from stream 2 - passes through
    tx2.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    let StreamItem::Error(_) = merged.next().await.unwrap() else {
        panic!("Expected Error");
    };

    tx3.send(StreamItem::Value(Sequenced::with_timestamp(5, 3)))?;
    let StreamItem::Value(result) = merged.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(result.into_inner(), 35);

    drop(tx1);
    drop(tx2);
    drop(tx3);

    Ok(())
}

#[tokio::test]
async fn test_merge_with_into_fluxion_stream_error_handling() -> anyhow::Result<()> {
    // Arrange: Test error handling through into_fluxion_stream() chain
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Create a MergedStream that returns raw Sequenced values, wrap in FluxionStream
    let mut merged = MergedStream::seed::<Sequenced<i32>>(0)
        .merge_with(stream, |value, state| {
            *state += value;
            *state
        })
        .into_stream();

    // Act: Send value (error will be filtered by filter_map above)
    tx.send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;
    let StreamItem::Value(v) = merged.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(v.into_inner(), 10);

    // Send error - will pass through, stream continues
    tx.send(StreamItem::Error(FluxionError::stream_error("Filtered")))?;
    let StreamItem::Error(_) = merged.next().await.unwrap() else {
        panic!("Expected Error");
    };

    // Send another value
    tx.send(StreamItem::Value(Sequenced::with_timestamp(20, 2)))?;
    let StreamItem::Value(v) = merged.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(v.into_inner(), 30);

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_merge_with_poll_pending_simulation() -> anyhow::Result<()> {
    // Arrange: Test that stream handles Poll::Pending correctly

    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut merged = MergedStream::seed::<Sequenced<i32>>(0).merge_with(stream, |value, state| {
        *state += value;
        *state
    });

    // Act: Try to poll before data is available (will return Poll::Pending)
    // Then send data
    tokio::select! {
        _ = tokio::time::sleep(tokio::time::Duration::from_millis(10)) => {
            tx.send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;
        }
        result = merged.next() => {
            // If we get here, stream returned something (unlikely)
            if let Some(StreamItem::Value(v)) = result {
                assert_eq!(v.into_inner(), 10);
            }
        }
    }

    assert_eq!(
        unwrap_stream(&mut merged, 100).await.unwrap().into_inner(),
        10
    );

    drop(tx);

    Ok(())
}
