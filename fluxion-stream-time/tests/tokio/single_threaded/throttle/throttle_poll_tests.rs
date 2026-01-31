// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_runtime::impls::tokio::TokioTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream_time::{ThrottleExt, TokioTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, test_channel, unwrap_stream},
    test_data::{person_alice, person_bob, person_charlie, TestData},
};
use std::time::Duration;
use tokio::time::{advance, pause};

#[tokio::test]
async fn test_throttle_returns_pending_while_throttling() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut throttled = stream.throttle(Duration::from_millis(500));

    // Act
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut throttled, 100).await.unwrap().value,
        person_alice()
    );

    // Act
    tx.unbounded_send(TokioTimestamped::new(person_bob(), timer.now()))?;

    // Assert
    assert_no_element_emitted(&mut throttled, 0).await;

    // Act
    advance(Duration::from_millis(500)).await;
    tx.unbounded_send(TokioTimestamped::new(person_charlie(), timer.now()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut throttled, 100).await.unwrap().value,
        person_charlie()
    );

    Ok(())
}

#[tokio::test]
async fn test_throttle_pending_without_values() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut throttled = stream.throttle(Duration::from_millis(500));

    // Assert
    assert_no_element_emitted(&mut throttled, 0).await;

    // Assert
    assert_no_element_emitted(&mut throttled, 0).await;

    // Act
    let timer = TokioTimer;
    pause();
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut throttled, 100).await.unwrap().value,
        person_alice()
    );

    Ok(())
}

#[tokio::test]
async fn test_throttle_pending_during_throttle_period() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut throttled = stream.throttle(Duration::from_millis(500));

    // Act
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut throttled, 100).await.unwrap().value,
        person_alice()
    );

    // Act
    advance(Duration::from_millis(250)).await;
    tx.unbounded_send(TokioTimestamped::new(person_bob(), timer.now()))?;
    tx.unbounded_send(TokioTimestamped::new(person_charlie(), timer.now()))?;

    // Assert
    assert_no_element_emitted(&mut throttled, 0).await;

    // Act
    advance(Duration::from_millis(250)).await;

    // Assert
    assert_no_element_emitted(&mut throttled, 0).await;

    Ok(())
}

#[tokio::test]
async fn test_throttle_pending_then_timer_expires() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut throttled = stream.throttle(Duration::from_millis(500));

    // Act
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut throttled, 100).await.unwrap().value,
        person_alice()
    );

    // Act
    tx.unbounded_send(TokioTimestamped::new(person_bob(), timer.now()))?;
    advance(Duration::from_millis(499)).await;

    // Assert
    assert_no_element_emitted(&mut throttled, 0).await;

    // Act
    advance(Duration::from_millis(1)).await;
    tx.unbounded_send(TokioTimestamped::new(person_charlie(), timer.now()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut throttled, 100).await.unwrap().value,
        person_charlie()
    );

    Ok(())
}
