// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream_multi::{CombineWithPreviousExt, FilterOrderedExt, MapOrderedExt};
use fluxion_test_utils::{assert_stream_ended, test_channel_with_errors, unwrap_stream, Sequenced};
use futures::StreamExt;

#[tokio::test]
async fn test_filter_ordered_propagates_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = stream.filter_ordered(|x| x % 2 == 0);

    // Act & Assert: First item (1) filtered out
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;

    // Error
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Value that passes filter
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 2)))?;

    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if v.value == 2
    ));

    // Value filtered out
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(3, 3)))?;

    // Value that passes
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(4, 4)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 4
    ));

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_predicate_after_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Only pass values > 18
    let mut result = stream.filter_ordered(|x| *x > 18);

    // Values that don't pass
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(15, 2)))?;

    // Error
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Values that pass filter
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(20, 3)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 20
    ));

    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(25, 4)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 25
    ));

    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(30, 5)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 30
    ));

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_error_at_start() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = stream.filter_ordered(|x| *x > 1);

    // Act & Assert: Error first
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Filtered values
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 2)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 2
    ));

    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(3, 3)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 3
    ));

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_all_filtered_except_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = stream.filter_ordered(|x| x % 2 == 0);

    // Act & Assert: Send odd number (filtered)
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;

    // Error
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // More odd numbers (filtered)
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(5, 3)))?;

    // Close stream
    drop(tx);

    // No more items (all filtered)
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_chain_with_map_after_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Chain filter and map
    let mut result = stream
        .filter_ordered(|x| *x >= 20)
        .combine_with_previous()
        .map_ordered(|x| Sequenced::new(x.current.value / 10));

    // Act & Assert: Send values
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?; // Filtered

    // Error
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Values that pass filter
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(20, 2)))?;
    assert!(matches!(result.next().await.unwrap(), StreamItem::Value(ref v) if v.value == 2));

    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(30, 3)))?;
    assert!(matches!(result.next().await.unwrap(), StreamItem::Value(ref v) if v.value == 3));

    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(40, 4)))?;
    assert!(matches!(result.next().await.unwrap(), StreamItem::Value(ref v) if v.value == 4));

    drop(tx);

    Ok(())
}
