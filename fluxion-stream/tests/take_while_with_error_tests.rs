// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `take_while_with` operator.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::TakeWhileExt;
use fluxion_test_utils::{
    assert_no_element_emitted, assert_stream_ended, test_channel_with_errors, unwrap_stream,
    ChronoTimestamped,
};
use futures::StreamExt;

#[tokio::test]
async fn test_take_while_with_propagates_source_error() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<ChronoTimestamped<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<ChronoTimestamped<i32>>();

    let mut filtered_stream = source_stream.take_while_with(filter_stream, |f| *f > 0);

    // Act & Assert: Send filter value first then source value
    filter_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(100, 1)))?;
    assert_no_element_emitted(&mut filtered_stream, 100).await;
    source_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(1, 2)))?;
    assert!(matches!(
        unwrap_stream(&mut filtered_stream, 100).await,
        StreamItem::Value(_)
    ));

    // Send error in source
    source_tx.send(StreamItem::Error(FluxionError::stream_error(
        "Source error",
    )))?;

    // Error should be propagated
    assert!(matches!(
        unwrap_stream(&mut filtered_stream, 100).await,
        StreamItem::Error(_)
    ));

    // Stream continues after error
    source_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(2, 3)))?;
    assert!(matches!(
        unwrap_stream(&mut filtered_stream, 100).await,
        StreamItem::Value(_)
    ));

    // Drop channels to close streams
    drop(source_tx);
    drop(filter_tx);

    // stream terminates when channels close
    assert_stream_ended(&mut filtered_stream, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_propagates_filter_error() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<ChronoTimestamped<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<ChronoTimestamped<i32>>();

    let mut filtered_stream = source_stream.take_while_with(filter_stream, |f| *f > 0);

    // Act & Assert: Send filter value
    filter_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(100, 1)))?;
    assert_no_element_emitted(&mut filtered_stream, 100).await;
    source_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(1, 2)))?;
    assert!(matches!(
        unwrap_stream(&mut filtered_stream, 100).await,
        StreamItem::Value(_)
    ));

    // Send error in filter
    filter_tx.send(StreamItem::Error(FluxionError::stream_error(
        "Filter error",
    )))?;

    // Error should be propagated
    assert!(matches!(
        unwrap_stream(&mut filtered_stream, 100).await,
        StreamItem::Error(_)
    ));

    // Stream continues after error
    source_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(2, 3)))?;
    assert!(matches!(
        unwrap_stream(&mut filtered_stream, 100).await,
        StreamItem::Value(_)
    ));

    drop(source_tx);
    drop(filter_tx);

    // stream terminates when channels close
    assert_stream_ended(&mut filtered_stream, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_predicate_after_error() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<ChronoTimestamped<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<ChronoTimestamped<i32>>();

    let mut result = source_stream.take_while_with(filter_stream, |f| *f > 0);

    // Act & Assert: Send filter value
    filter_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(100, 1)))?;
    assert_no_element_emitted(&mut result, 100).await;
    source_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(1, 2)))?;
    assert!(matches!(result.next().await.unwrap(), StreamItem::Value(_)));

    // Send error in source
    source_tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;

    // Error should be propagated
    assert!(matches!(result.next().await.unwrap(), StreamItem::Error(_)));

    // Stream continues - send more values
    source_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(2, 3)))?;
    assert!(matches!(result.next().await.unwrap(), StreamItem::Value(_)));

    drop(source_tx);
    drop(filter_tx);

    // Stream terminates when channels close
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_error_at_start() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<ChronoTimestamped<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<ChronoTimestamped<i32>>();

    let mut result = source_stream.take_while_with(filter_stream, |f| *f > 0);

    // Act & Assert: Send filter value
    filter_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(100, 1)))?;
    assert_no_element_emitted(&mut result, 100).await;
    source_tx.send(StreamItem::Error(FluxionError::stream_error("Early error")))?;

    // Error should be propagated
    assert!(matches!(result.next().await.unwrap(), StreamItem::Error(_)));

    // Stream continues
    source_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(1, 2)))?;
    assert!(matches!(result.next().await.unwrap(), StreamItem::Value(_)));

    drop(source_tx);
    drop(filter_tx);

    // Stream terminates when channels close
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_stops_on_false_despite_errors() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<ChronoTimestamped<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<ChronoTimestamped<i32>>();

    let mut result = source_stream.take_while_with(filter_stream, |f| *f > 0);

    // Act & Assert: Send filter value that passes
    filter_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(100, 1)))?;
    assert_no_element_emitted(&mut result, 100).await;
    source_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(1, 2)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Send filter value that fails predicate (stops stream)
    filter_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(0, 3)))?;

    drop(source_tx);
    drop(filter_tx);

    // Stream should terminate when predicate returns false
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_source_before_filter() -> anyhow::Result<()> {
    // Test the branch where source arrives but filter is None
    let (source_tx, source_stream) = test_channel_with_errors::<ChronoTimestamped<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<ChronoTimestamped<i32>>();

    let mut result = source_stream.take_while_with(filter_stream, |f| *f > 0);

    // Send source first (filter is None) - should not emit
    source_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(100, 1)))?;

    assert_no_element_emitted(&mut result, 100).await;

    // Now send filter value
    filter_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(50, 2)))?;

    // Still no emission from the cached source (filter update doesn't emit)
    assert_no_element_emitted(&mut result, 100).await;

    // Send another source value - now should emit
    source_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(200, 3)))?;

    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(source_tx);
    drop(filter_tx);

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_filter_updates_then_source() -> anyhow::Result<()> {
    // Test multiple filter updates before source arrives
    let (source_tx, source_stream) = test_channel_with_errors::<ChronoTimestamped<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<ChronoTimestamped<i32>>();

    let mut result = source_stream.take_while_with(filter_stream, |f| *f > 10);

    // Send multiple filter values (source is None)
    filter_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(100, 1)))?;
    filter_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(50, 2)))?;
    filter_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(25, 3)))?;

    assert_no_element_emitted(&mut result, 100).await;

    // Now send source value - should use latest filter (25 > 10 = true)
    source_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(999, 4)))?;

    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(source_tx);
    drop(filter_tx);

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_already_terminated() -> anyhow::Result<()> {
    // Test the terminated flag branch
    let (source_tx, source_stream) = test_channel_with_errors::<ChronoTimestamped<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<ChronoTimestamped<i32>>();

    let mut result = source_stream.take_while_with(filter_stream, |f| *f > 0);

    // Setup: filter passes, source emits
    filter_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(100, 1)))?;
    source_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(1, 2)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Trigger termination: filter fails predicate (timestamp 3 < 4, so filter processes first)
    filter_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(0, 3)))?;
    source_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(2, 4)))?;

    // Source at timestamp 4 should still emit because filter at timestamp 3 doesn't change state until processed
    // But after filter is processed and predicate fails, it should terminate
    // Actually the source at timestamp 4 will be processed and check against filter at timestamp 3 which fails
    // So no emission and termination
    assert_no_element_emitted(&mut result, 100).await;

    // Send more values - should still get nothing (terminated)
    source_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(3, 5)))?;
    filter_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(100, 6)))?;

    assert_no_element_emitted(&mut result, 100).await;

    drop(source_tx);
    drop(filter_tx);

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_filter_false_on_first_source() -> anyhow::Result<()> {
    // Test termination on very first source value
    let (source_tx, source_stream) = test_channel_with_errors::<ChronoTimestamped<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<ChronoTimestamped<i32>>();

    let mut result = source_stream.take_while_with(filter_stream, |f| *f > 50);

    // Setup filter that will fail
    filter_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(10, 1)))?;

    // Send source - should terminate immediately (10 <= 50)
    source_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(100, 2)))?;

    // No emission
    assert_no_element_emitted(&mut result, 100).await;

    // Drop channels
    drop(source_tx);
    drop(filter_tx);

    // Stream terminated
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_alternating_filter_values() -> anyhow::Result<()> {
    // Test filter changing but never failing predicate
    let (source_tx, source_stream) = test_channel_with_errors::<ChronoTimestamped<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<ChronoTimestamped<i32>>();

    let mut result = source_stream.take_while_with(filter_stream, |f| *f > 0);

    // All filter values pass predicate
    filter_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(100, 1)))?;
    source_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(1, 2)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    filter_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(50, 3)))?;
    source_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(2, 4)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    filter_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(25, 5)))?;
    source_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(3, 6)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(source_tx);
    drop(filter_tx);

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_error_recovery() -> anyhow::Result<()> {
    // Test that stream continues after errors
    let (source_tx, source_stream) = test_channel_with_errors::<ChronoTimestamped<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<ChronoTimestamped<i32>>();

    let mut result = source_stream.take_while_with(filter_stream, |f| *f > 0);

    filter_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(100, 1)))?;
    source_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(1, 2)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Error from source
    source_tx.send(StreamItem::Error(FluxionError::stream_error("err1")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Continue
    source_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(2, 3)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));
    // Error from filter
    filter_tx.send(StreamItem::Error(FluxionError::stream_error("err2")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Continue
    source_tx.send(StreamItem::Value(ChronoTimestamped::with_timestamp(3, 4)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));
    drop(source_tx);
    drop(filter_tx);

    Ok(())
}
