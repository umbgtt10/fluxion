// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `distinct_until_changed` operator.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::DistinctUntilChangedExt;
use fluxion_test_utils::{
    assert_stream_ended, test_channel_with_errors,
    test_data::{person_alice, person_bob, TestData},
    unwrap_stream, Sequenced,
};

#[tokio::test]
async fn test_distinct_until_changed_propagates_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.distinct_until_changed();

    // Act & Assert: First value emitted
    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if v.value == person_alice()
    ));

    // Error should be propagated
    tx.send(StreamItem::Error(FluxionError::stream_error("Test error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Duplicate value after error - should be filtered
    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        2,
    )))?;

    // Different value - should be emitted
    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_bob(),
        3,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if v.value == person_bob()
    ));

    drop(tx);
    assert_stream_ended(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_error_at_start() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.distinct_until_changed();

    // Act & Assert: Error before any values
    tx.send(StreamItem::Error(FluxionError::stream_error(
        "Initial error",
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // First value should still be emitted (no previous to compare)
    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if v.value == person_alice()
    ));

    // Duplicate - filtered
    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        2,
    )))?;

    // Different value - emitted
    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_bob(),
        3,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if v.value == person_bob()
    ));

    drop(tx);
    assert_stream_ended(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_multiple_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.distinct_until_changed();

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if v.value == person_alice()
    ));

    // Error 1
    tx.send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Error 2
    tx.send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Duplicate value - filtered
    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        2,
    )))?;

    // Error 3
    tx.send(StreamItem::Error(FluxionError::stream_error("Error 3")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // New value - emitted
    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_bob(),
        3,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if v.value == person_bob()
    ));

    drop(tx);
    assert_stream_ended(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_error_between_duplicates() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.distinct_until_changed();

    // Act & Assert: First value
    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if v.value == person_alice()
    ));

    // Duplicate - filtered
    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        2,
    )))?;

    // Error in the middle of duplicates
    tx.send(StreamItem::Error(FluxionError::stream_error(
        "Mid-stream error",
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // More duplicates after error - still filtered
    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        3,
    )))?;
    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        4,
    )))?;

    // Different value - emitted
    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_bob(),
        5,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if v.value == person_bob()
    ));

    drop(tx);
    assert_stream_ended(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_preserves_state_after_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.distinct_until_changed();

    // Act & Assert: Establish state with alice
    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if v.value == person_alice()
    ));

    // Change to bob
    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_bob(),
        2,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if v.value == person_bob()
    ));

    // Error occurs
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // State should be preserved: last emitted was bob, so duplicate bob is filtered
    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_bob(),
        3,
    )))?;

    // Change back to alice - should be emitted (different from last emitted value bob)
    tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        4,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if v.value == person_alice()
    ));

    drop(tx);
    assert_stream_ended(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_alternating_errors_and_values() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<bool>>();
    let mut result = stream.distinct_until_changed();

    // Act & Assert: Interleave errors and values
    tx.send(StreamItem::Value(Sequenced::with_timestamp(true, 1)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if v.value
    ));

    tx.send(StreamItem::Error(FluxionError::stream_error("E1")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Value(Sequenced::with_timestamp(false, 2)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if !v.value
    ));

    tx.send(StreamItem::Error(FluxionError::stream_error("E2")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Value(Sequenced::with_timestamp(false, 3)))?; // Duplicate - filtered

    tx.send(StreamItem::Error(FluxionError::stream_error("E3")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Value(Sequenced::with_timestamp(true, 4)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if v.value
    ));

    drop(tx);
    assert_stream_ended(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_error_only_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.distinct_until_changed();

    // Act & Assert: Send only errors, no values
    tx.send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Error(FluxionError::stream_error("Error 3")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    drop(tx);
    assert_stream_ended(&mut result, 100).await;

    Ok(())
}
