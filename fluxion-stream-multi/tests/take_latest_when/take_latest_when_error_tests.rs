// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `take_latest_when` operator.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream_multi::TakeLatestWhenExt;
use fluxion_test_utils::{
    assert_no_element_emitted, test_channel_with_errors, unwrap_stream, Sequenced,
};

#[tokio::test]
async fn test_take_latest_when_propagates_source_error() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |_| true);

    // Act & Assert: Send source values
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 2)))?;

    // Send trigger
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 4)))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    // Send error in source
    source_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Source error",
    )))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Error(_)
    ));

    // Continue with values
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

    // Send source values
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    assert_no_element_emitted(&mut triggered_stream, 500).await;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 2)))?;
    assert_no_element_emitted(&mut triggered_stream, 500).await;

    // Send trigger
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 3)))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    // Send error in trigger
    trigger_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Trigger error",
    )))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Error(_)
    ));

    // Continue
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

    // Only emit when trigger is > 50
    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |t| *t > 50);

    // Send source values
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    assert_no_element_emitted(&mut triggered_stream, 500).await;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 2)))?;
    assert_no_element_emitted(&mut triggered_stream, 500).await;

    // Send error in source
    source_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Error(_)
    ));

    // Send value and trigger that passes filter
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

    // Send initial values
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    assert_no_element_emitted(&mut triggered_stream, 500).await;
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 2)))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    // Error from source
    source_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Source error",
    )))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Error(_)
    ));

    // Error from trigger
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
    // This tests the branch where filter updates but source is None
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |_| true);

    // Send trigger first (source is None) - should not emit
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;

    assert_no_element_emitted(&mut triggered_stream, 100).await;

    // Now send source value
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(42, 2)))?;

    // Still no emission (just cached)
    assert_no_element_emitted(&mut triggered_stream, 100).await;

    // Send another trigger - now should emit
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
    // This tests the branch where filter predicate returns false
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |val| *val > 50);

    // Setup source
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(100, 1)))?;

    // Send trigger that doesn't pass filter (10 <= 50)
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 2)))?;

    // Should NOT emit
    assert_no_element_emitted(&mut triggered_stream, 100).await;

    // Send trigger that passes filter (100 > 50)
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(100, 3)))?;

    // Should emit now
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
    // Test multiple filter emissions before source has value
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |_| true);

    // Send multiple triggers (source is None)
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(20, 2)))?;
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(30, 3)))?;

    // No emissions should occur
    assert_no_element_emitted(&mut triggered_stream, 100).await;

    // Now send source value
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(999, 4)))?;

    // Still no emission (waiting for trigger)
    assert_no_element_emitted(&mut triggered_stream, 100).await;

    // Send one more trigger
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(40, 5)))?;

    // Now should emit
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
    // Test alternating between passing and failing filter conditions
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |val| *val > 10);

    // Setup source
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(100, 1)))?;

    // Trigger that passes (20 > 10)
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(20, 2)))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    // Trigger that fails (5 <= 10)
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(5, 3)))?;
    assert_no_element_emitted(&mut triggered_stream, 100).await;

    // Trigger that passes (50 > 10)
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(50, 4)))?;
    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Value(_)
    ));

    // Trigger that fails (0 <= 10)
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(0, 5)))?;
    assert_no_element_emitted(&mut triggered_stream, 100).await;

    drop(source_tx);
    drop(trigger_tx);

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_source_updates_dont_emit() -> anyhow::Result<()> {
    // Verify that source updates are cached but don't trigger emissions
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |_| true);

    // Send many source updates
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 2)))?;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(3, 3)))?;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(4, 4)))?;

    // None should emit
    assert_no_element_emitted(&mut triggered_stream, 100).await;

    // Send trigger - should emit latest (4)
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
    // Test error propagation before filter stream has emitted
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |_| true);

    // Send error before any trigger
    source_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Early error")))?;

    assert!(matches!(
        unwrap_stream(&mut triggered_stream, 500).await,
        StreamItem::Error(_)
    ));

    // Continue normally
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
    // Verify that only the latest source value is emitted
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut triggered_stream = source_stream.take_latest_when(trigger_stream, |_| true);

    // First cycle
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(100, 1)))?;
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 2)))?;

    let result = unwrap_stream(&mut triggered_stream, 500).await;
    if let StreamItem::Value(val) = result {
        assert_eq!(val.clone().into_inner(), 100);
    }

    // Update source multiple times
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(200, 3)))?;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(300, 4)))?;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(400, 5)))?;

    // Trigger - should get latest (400)
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 6)))?;

    let result = unwrap_stream(&mut triggered_stream, 500).await;
    if let StreamItem::Value(val) = result {
        assert_eq!(val.clone().into_inner(), 400);
    }

    drop(source_tx);
    drop(trigger_tx);

    Ok(())
}
