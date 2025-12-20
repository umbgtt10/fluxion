// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `window_by_count` operator.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::WindowByCountExt;
use fluxion_test_utils::{
    assert_no_element_emitted, assert_stream_ended, test_channel_with_errors,
    test_data::{animal_cat, animal_dog, person_alice, person_bob, person_charlie, TestData},
    unwrap_stream, unwrap_value, Sequenced,
};

#[tokio::test]
async fn test_window_by_count_propagates_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.window_by_count::<Sequenced<Vec<TestData>>>(3);

    // Act: Send one value, then error
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("test error")))?;

    // Assert: Error is propagated
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_window_by_count_continues_after_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.window_by_count::<Sequenced<Vec<TestData>>>(2);

    // Act & Assert
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("test error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        vec![person_alice(), person_bob()]
    );

    Ok(())
}

#[tokio::test]
async fn test_window_by_count_error_clears_partial_window() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.window_by_count::<Sequenced<Vec<TestData>>>(3);

    // Act: Send 2 values (partial window), then error, then complete window
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("test error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.unbounded_send(StreamItem::Value(Sequenced::new(animal_dog())))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(animal_cat())))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_charlie())))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        vec![animal_dog(), animal_cat(), person_charlie()]
    );

    Ok(())
}

#[tokio::test]
async fn test_window_by_count_multiple_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.window_by_count::<Sequenced<Vec<TestData>>>(2);

    // Act: Multiple errors interspersed with values
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error 1")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error 2")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.unbounded_send(StreamItem::Value(Sequenced::new(animal_dog())))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(animal_cat())))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        vec![animal_dog(), animal_cat()]
    );

    Ok(())
}

#[tokio::test]
async fn test_window_by_count_error_at_window_boundary() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.window_by_count::<Sequenced<Vec<TestData>>>(2);

    // Act & Assert
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        vec![person_alice(), person_bob()]
    );

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "boundary error",
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.unbounded_send(StreamItem::Value(Sequenced::new(animal_dog())))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(animal_cat())))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        vec![animal_dog(), animal_cat()]
    );

    Ok(())
}

#[tokio::test]
async fn test_window_by_count_error_preserves_error_details() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.window_by_count::<Sequenced<Vec<TestData>>>(2);

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "specific error message",
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(e) if e.to_string().contains("specific error message")
    ));

    Ok(())
}

#[tokio::test]
async fn test_window_by_count_stream_ends_after_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = Box::pin(stream.window_by_count::<Sequenced<Vec<TestData>>>(3));

    // Act: Send error then close
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("final error")))?;
    drop(tx);

    // Assert: Error propagated
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Stream ends
    assert_stream_ended(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_window_by_count_partial_after_error_on_close() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = Box::pin(stream.window_by_count::<Sequenced<Vec<TestData>>>(3));

    // Act: Error, then partial window, then close
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    drop(tx);
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        vec![person_alice(), person_bob()]
    );

    assert_stream_ended(&mut result, 100).await;

    Ok(())
}
