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

    // Assert
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    // Act
    source_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Source error",
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Error(_)
    ));

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(3, 3)))?;
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(20, 5)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_propagates_trigger_error() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |_| true);

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;

    // Assert
    assert_no_element_emitted(&mut triggered_stream, 500).await;

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 2)))?;

    // Assert
    assert_no_element_emitted(&mut triggered_stream, 500).await;

    // Act
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 3)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    // Act
    trigger_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Trigger error",
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Error(_)
    ));

    // Act
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(30, 5)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_filter_predicate_after_error() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |t| *t > 50);

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;

    // Assert
    assert_no_element_emitted(&mut triggered_stream, 500).await;

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 2)))?;

    // Assert
    assert_no_element_emitted(&mut triggered_stream, 500).await;

    // Act
    source_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Error(_)
    ));

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(3, 3)))?;
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(100, 5)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_both_streams_have_errors() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |_| true);

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;

    // Assert
    assert_no_element_emitted(&mut triggered_stream, 500).await;

    // Act
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 2)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    // Act
    source_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Source error",
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Error(_)
    ));

    // Act
    trigger_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Trigger error",
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Error(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_trigger_before_source() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |_| true);

    // Act
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;

    // Assert
    assert_no_element_emitted(&mut triggered_stream, 100).await;

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(42, 2)))?;

    // Assert
    assert_no_element_emitted(&mut triggered_stream, 100).await;

    // Act
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(20, 3)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_filter_returns_false() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |val| *val > 50);

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(100, 1)))?;
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 2)))?;

    // Assert
    assert_no_element_emitted(&mut triggered_stream, 100).await;

    // Act
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(100, 3)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_multiple_triggers_no_source() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |_| true);

    // Act
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(20, 2)))?;
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(30, 3)))?;

    // Assert
    assert_no_element_emitted(&mut triggered_stream, 100).await;

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(999, 4)))?;

    // Assert
    assert_no_element_emitted(&mut triggered_stream, 100).await;

    // Act
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(40, 5)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_alternating_filter_conditions() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |val| *val > 10);

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(100, 1)))?;
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(20, 2)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    // Act
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(5, 3)))?;

    // Assert
    assert_no_element_emitted(&mut triggered_stream, 100).await;

    // Act
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(50, 4)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    // Act
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(0, 5)))?;

    // Assert
    assert_no_element_emitted(&mut triggered_stream, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_source_updates_dont_emit() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |_| true);

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 2)))?;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(3, 3)))?;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(4, 4)))?;

    // Assert
    assert_no_element_emitted(&mut triggered_stream, 100).await;

    // Act
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 5)))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut triggered_stream, 500)
            .await
            .unwrap()
            .into_inner(),
        4
    );

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_error_before_any_trigger() -> anyhow::Result<()> {
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |_| true);

    // Act
    source_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Early error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Error(_)
    ));

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(42, 1)))?;
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 2)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_source_overwritten_between_triggers() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |_| true);

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(100, 1)))?;
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 2)))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut triggered_stream, 500)
            .await
            .unwrap()
            .into_inner(),
        100
    );

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(200, 3)))?;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(300, 4)))?;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(400, 5)))?;
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 6)))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut triggered_stream, 500)
            .await
            .unwrap()
            .into_inner(),
        400
    );

    Ok(())
}
