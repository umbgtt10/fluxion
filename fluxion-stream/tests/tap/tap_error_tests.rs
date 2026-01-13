// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::TapExt;
use fluxion_test_utils::test_channel_with_errors;
use fluxion_test_utils::test_data::{person_alice, person_bob, person_charlie, TestData};
use fluxion_test_utils::{helpers::unwrap_stream, unwrap_value, Sequenced};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[tokio::test]
async fn test_tap_passes_through_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.tap(|_| {});

    // Act & Assert
    tx.try_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &person_alice()
    );

    // Act & Assert
    tx.try_send(StreamItem::Error(FluxionError::stream_error("test error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Error(_)
    ));

    // Act & Assert
    tx.try_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_tap_not_called_for_errors() -> anyhow::Result<()> {
    // Arrange
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.tap(move |_| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    // Act
    tx.try_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    unwrap_stream(&mut result, 500).await;

    tx.try_send(StreamItem::Error(FluxionError::stream_error("test error")))?;
    unwrap_stream(&mut result, 500).await;

    tx.try_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    unwrap_stream(&mut result, 500).await;

    // Assert
    assert_eq!(counter.load(Ordering::SeqCst), 2);

    Ok(())
}

#[tokio::test]
async fn test_tap_multiple_errors_pass_through() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.tap(|_| {});

    // Act & Assert
    tx.try_send(StreamItem::Error(FluxionError::stream_error("error 1")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Error(e) if e.to_string().contains("error 1")
    ));

    tx.try_send(StreamItem::Error(FluxionError::stream_error("error 2")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Error(e) if e.to_string().contains("error 2")
    ));

    tx.try_send(StreamItem::Error(FluxionError::stream_error("error 3")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Error(e) if e.to_string().contains("error 3")
    ));

    Ok(())
}

#[tokio::test]
async fn test_tap_interleaved_values_and_errors() -> anyhow::Result<()> {
    // Arrange
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.tap(move |_| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    // Act
    tx.try_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    unwrap_stream(&mut result, 500).await;

    tx.try_send(StreamItem::Error(FluxionError::stream_error("error 1")))?;
    unwrap_stream(&mut result, 500).await;

    tx.try_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    unwrap_stream(&mut result, 500).await;

    tx.try_send(StreamItem::Error(FluxionError::stream_error("error 2")))?;
    unwrap_stream(&mut result, 500).await;

    tx.try_send(StreamItem::Value(Sequenced::new(person_charlie())))?;
    unwrap_stream(&mut result, 500).await;

    // Assert
    assert_eq!(counter.load(Ordering::SeqCst), 3);

    Ok(())
}

#[tokio::test]
async fn test_tap_error_preserves_message() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.tap(|_| {});

    // Act
    let error_message = "specific error message to preserve";
    tx.try_send(StreamItem::Error(FluxionError::stream_error(error_message)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Error(e) if e.to_string().contains(error_message)
    ));

    Ok(())
}

#[tokio::test]
async fn test_tap_only_errors() -> anyhow::Result<()> {
    // Arrange
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.tap(move |_| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    // Act
    tx.try_send(StreamItem::Error(FluxionError::stream_error("error 1")))?;
    unwrap_stream(&mut result, 500).await;

    tx.try_send(StreamItem::Error(FluxionError::stream_error("error 2")))?;
    unwrap_stream(&mut result, 500).await;

    // Assert
    assert_eq!(counter.load(Ordering::SeqCst), 0);

    Ok(())
}
