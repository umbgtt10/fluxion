// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::TakeLatestWhenExt;
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, test_channel_with_errors, unwrap_stream},
    sequenced::Sequenced,
};

#[tokio::test]
async fn test_take_latest_when_propagates_source_error() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |_| true);

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 2)))?;
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 4)))?;

    //
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    source_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Source error",
    )))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Error(_)
    ));

    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(3, 3)))?;
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(20, 5)))?;
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
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |_| true);

    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    assert_no_element_emitted(&mut triggered_stream, 500).await;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 2)))?;
    assert_no_element_emitted(&mut triggered_stream, 500).await;

    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 3)))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    trigger_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Trigger error",
    )))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Error(_)
    ));

    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(30, 5)))?;
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
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |t| *t > 50);

    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    assert_no_element_emitted(&mut triggered_stream, 500).await;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 2)))?;
    assert_no_element_emitted(&mut triggered_stream, 500).await;

    source_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Error(_)
    ));

    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(3, 3)))?;
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(100, 5)))?;

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
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |_| true);

    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    assert_no_element_emitted(&mut triggered_stream, 500).await;
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 2)))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    source_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Source error",
    )))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Error(_)
    ));

    trigger_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
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

#[tokio::test]
async fn test_take_latest_when_trigger_before_source() -> anyhow::Result<()> {
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |_| true);

    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;

    assert_no_element_emitted(&mut triggered_stream, 100).await;

    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(42, 2)))?;

    assert_no_element_emitted(&mut triggered_stream, 100).await;

    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(20, 3)))?;

    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    drop(source_tx);
    drop(trigger_tx);

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_filter_returns_false() -> anyhow::Result<()> {
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |val| *val > 50);

    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(100, 1)))?;

    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 2)))?;

    assert_no_element_emitted(&mut triggered_stream, 100).await;

    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(100, 3)))?;

    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    drop(source_tx);
    drop(trigger_tx);

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_multiple_triggers_no_source() -> anyhow::Result<()> {
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |_| true);

    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(20, 2)))?;
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(30, 3)))?;

    assert_no_element_emitted(&mut triggered_stream, 100).await;

    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(999, 4)))?;

    assert_no_element_emitted(&mut triggered_stream, 100).await;

    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(40, 5)))?;

    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    drop(source_tx);
    drop(trigger_tx);

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_alternating_filter_conditions() -> anyhow::Result<()> {
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |val| *val > 10);

    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(100, 1)))?;

    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(20, 2)))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(5, 3)))?;
    assert_no_element_emitted(&mut triggered_stream, 100).await;

    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(50, 4)))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(0, 5)))?;
    assert_no_element_emitted(&mut triggered_stream, 100).await;

    drop(source_tx);
    drop(trigger_tx);

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_source_updates_dont_emit() -> anyhow::Result<()> {
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |_| true);

    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 2)))?;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(3, 3)))?;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(4, 4)))?;

    assert_no_element_emitted(&mut triggered_stream, 100).await;

    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 5)))?;

    let result = unwrap_stream(&mut triggered_stream, 500).await;
    if let StreamItem::Value(val) = result {
        assert_eq!(val.clone().into_inner(), 4);
    } else {
        panic!("Expected value");
    }

    drop(source_tx);
    drop(trigger_tx);

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_error_before_any_trigger() -> anyhow::Result<()> {
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |_| true);

    source_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Early error")))?;

    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Error(_)
    ));

    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(42, 1)))?;
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 2)))?;

    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    drop(source_tx);
    drop(trigger_tx);

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_source_overwritten_between_triggers() -> anyhow::Result<()> {
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |_| true);

    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(100, 1)))?;
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 2)))?;

    let result = unwrap_stream(&mut triggered_stream, 500).await;
    if let StreamItem::Value(val) = result {
        assert_eq!(val.clone().into_inner(), 100);
    }

    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(200, 3)))?;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(300, 4)))?;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(400, 5)))?;

    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 6)))?;

    let result = unwrap_stream(&mut triggered_stream, 500).await;
    if let StreamItem::Value(val) = result {
        assert_eq!(val.clone().into_inner(), 400);
    }

    drop(source_tx);
    drop(trigger_tx);

    Ok(())
}
