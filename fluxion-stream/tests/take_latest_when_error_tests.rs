// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `take_latest_when` operator.

use fluxion_core::Timestamped as TimestampedTrait;

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::TakeLatestWhenExt;
use fluxion_test_utils::{
    assert_no_element_emitted, test_channel_with_errors, unwrap_stream, Timestamped,
};

#[tokio::test]
async fn test_take_latest_when_propagates_source_error() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Timestamped<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Timestamped<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |_| true);

    // Act & Assert: Send source values
    source_tx.send(StreamItem::Value(Timestamped::with_timestamp(1, 1)))?;
    source_tx.send(StreamItem::Value(Timestamped::with_timestamp(2, 2)))?;

    // Send trigger
    trigger_tx.send(StreamItem::Value(Timestamped::with_timestamp(10, 4)))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    // Send error in source
    source_tx.send(StreamItem::Error(FluxionError::stream_error(
        "Source error",
    )))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Error(_)
    ));

    // Continue with values
    source_tx.send(StreamItem::Value(Timestamped::with_timestamp(3, 3)))?;
    trigger_tx.send(StreamItem::Value(Timestamped::with_timestamp(20, 5)))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    drop(source_tx);
    drop(trigger_tx);

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_propagates_trigger_error() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Timestamped<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Timestamped<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |_| true);

    // Send source values
    source_tx.send(StreamItem::Value(Timestamped::with_timestamp(1, 1)))?;
    assert_no_element_emitted(&mut triggered_stream, 500).await;
    source_tx.send(StreamItem::Value(Timestamped::with_timestamp(2, 2)))?;
    assert_no_element_emitted(&mut triggered_stream, 500).await;

    // Send trigger
    trigger_tx.send(StreamItem::Value(Timestamped::with_timestamp(10, 3)))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    // Send error in trigger
    trigger_tx.send(StreamItem::Error(FluxionError::stream_error(
        "Trigger error",
    )))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Error(_)
    ));

    // Continue
    trigger_tx.send(StreamItem::Value(Timestamped::with_timestamp(30, 5)))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    drop(source_tx);
    drop(trigger_tx);

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_filter_predicate_after_error() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Timestamped<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Timestamped<i32>>();

    // Only emit when trigger is > 50
    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |t| *t > 50);

    // Send source values
    source_tx.send(StreamItem::Value(Timestamped::with_timestamp(1, 1)))?;
    assert_no_element_emitted(&mut triggered_stream, 500).await;
    source_tx.send(StreamItem::Value(Timestamped::with_timestamp(2, 2)))?;
    assert_no_element_emitted(&mut triggered_stream, 500).await;

    // Send error in source
    source_tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Error(_)
    ));

    // Send value and trigger that passes filter
    source_tx.send(StreamItem::Value(Timestamped::with_timestamp(3, 3)))?;
    trigger_tx.send(StreamItem::Value(Timestamped::with_timestamp(100, 5)))?;

    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    drop(source_tx);
    drop(trigger_tx);

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_both_streams_have_errors() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Timestamped<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Timestamped<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |_| true);

    // Send initial values
    source_tx.send(StreamItem::Value(Timestamped::with_timestamp(1, 1)))?;
    assert_no_element_emitted(&mut triggered_stream, 500).await;
    trigger_tx.send(StreamItem::Value(Timestamped::with_timestamp(10, 2)))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    // Error from source
    source_tx.send(StreamItem::Error(FluxionError::stream_error(
        "Source error",
    )))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Error(_)
    ));

    // Error from trigger
    trigger_tx.send(StreamItem::Error(FluxionError::stream_error(
        "Trigger error",
    )))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Error(_)
    ));

    drop(source_tx);
    drop(trigger_tx);

    Ok(())
}
