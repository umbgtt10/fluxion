// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_runtime::impls::tokio::TokioTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream_time::{DebounceExt, TokioTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, test_channel_with_errors, unwrap_stream},
    test_data::{person_alice, person_bob, TestData},
};
use std::time::Duration;
use tokio::time::{advance, pause};

#[tokio::test]
async fn test_debounce_errors_pass_through() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel_with_errors::<TokioTimestamped<TestData>>();
    let mut debounced = stream.debounce(Duration::from_millis(500));

    // Act
    tx.unbounded_send(StreamItem::Value(TokioTimestamped::new(
        person_alice(),
        timer.now(),
    )))?;

    // Assert
    assert_no_element_emitted(&mut debounced, 0).await;

    // Act
    advance(Duration::from_millis(300)).await;

    // Assert
    assert_no_element_emitted(&mut debounced, 0).await;

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("test error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut debounced, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    advance(Duration::from_millis(300)).await;

    // Assert
    assert_no_element_emitted(&mut debounced, 0).await;

    // Act
    tx.unbounded_send(StreamItem::Value(TokioTimestamped::new(
        person_bob(),
        timer.now(),
    )))?;

    // Assert
    assert_no_element_emitted(&mut debounced, 0).await;

    // Act
    advance(Duration::from_millis(300)).await;

    // Assert
    assert_no_element_emitted(&mut debounced, 0).await;

    // Act
    advance(Duration::from_millis(200)).await;

    // Assert
    assert_eq!(
        unwrap_stream(&mut debounced, 100).await.unwrap().value,
        person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_debounce_error_discards_pending() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel_with_errors::<TokioTimestamped<TestData>>();
    let mut debounced = stream.debounce(Duration::from_millis(500));

    // Act
    tx.unbounded_send(StreamItem::Value(TokioTimestamped::new(
        person_alice(),
        timer.now(),
    )))?;
    advance(Duration::from_millis(200)).await;
    tx.unbounded_send(StreamItem::Value(TokioTimestamped::new(
        person_bob(),
        timer.now(),
    )))?;
    advance(Duration::from_millis(200)).await;
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("test error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut debounced, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    advance(Duration::from_millis(500)).await;

    // Assert
    assert_no_element_emitted(&mut debounced, 100).await;

    Ok(())
}
