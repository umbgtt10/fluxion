// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_runtime::impls::tokio::TokioTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream_time::{TimeoutExt, TokioTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, test_channel, unwrap_stream},
    test_data::{person_alice, person_bob, TestData},
};
use futures::StreamExt;
use std::time::Duration;
use tokio::time::{advance, pause};

#[tokio::test]
async fn test_timeout_returns_pending_while_waiting() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut timeout_stream = stream.timeout(Duration::from_millis(500));

    // Act
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut timeout_stream, 100).await.unwrap().value,
        person_alice()
    );
    assert_no_element_emitted(&mut timeout_stream, 0).await;

    // Act
    advance(Duration::from_millis(300)).await;
    tx.unbounded_send(TokioTimestamped::new(person_bob(), timer.now()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut timeout_stream, 100).await.unwrap().value,
        person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_timeout_pending_without_values() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut timeout_stream = stream.timeout(Duration::from_millis(500));

    // Assert
    assert_no_element_emitted(&mut timeout_stream, 0).await;

    // Act
    let timer = TokioTimer;
    pause();
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut timeout_stream, 100).await.unwrap().value,
        person_alice()
    );

    Ok(())
}

#[tokio::test]
async fn test_timeout_pending_then_timer_expires() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut timeout_stream = stream.timeout(Duration::from_millis(500));

    // Act
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut timeout_stream, 100).await.unwrap().value,
        person_alice()
    );

    // Act
    advance(Duration::from_millis(300)).await;

    // Assert
    assert_no_element_emitted(&mut timeout_stream, 0).await;

    // Act
    advance(Duration::from_millis(200)).await;

    // Assert
    let result = unwrap_stream(&mut timeout_stream, 100).await;
    assert!(result.is_error());

    Ok(())
}

#[tokio::test]
async fn test_timeout_pending_with_stream_end() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut timeout_stream = stream.timeout(Duration::from_millis(500));

    // Act
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut timeout_stream, 100).await.unwrap().value,
        person_alice()
    );
    assert_no_element_emitted(&mut timeout_stream, 0).await;

    // Act
    drop(tx);

    // Assert
    assert!(timeout_stream.next().await.is_none());

    Ok(())
}
