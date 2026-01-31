// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_runtime::impls::tokio::TokioTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream_time::{DelayExt, TokioTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, test_channel, unwrap_stream},
    test_data::{person_alice, person_bob, TestData},
};
use futures::StreamExt;
use std::time::Duration;
use tokio::time::{advance, pause};

#[tokio::test]
async fn test_delay_returns_pending_while_waiting() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut delayed = stream.delay(Duration::from_millis(500));

    // Act
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;

    // Assert
    assert_no_element_emitted(&mut delayed, 0).await;

    // Act
    advance(Duration::from_millis(500)).await;

    // Assert
    assert_eq!(
        unwrap_stream(&mut delayed, 100).await.unwrap().value,
        person_alice()
    );

    Ok(())
}

#[tokio::test]
async fn test_delay_pending_without_values() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut delayed = stream.delay(Duration::from_millis(500));

    // Assert
    assert_no_element_emitted(&mut delayed, 0).await;

    // Act
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;

    // Assert
    assert_no_element_emitted(&mut delayed, 0).await;

    // Act
    advance(Duration::from_millis(500)).await;

    // Assert
    assert_eq!(
        unwrap_stream(&mut delayed, 100).await.unwrap().value,
        person_alice()
    );

    Ok(())
}

#[tokio::test]
async fn test_delay_pending_with_multiple_in_flight() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut delayed = stream.delay(Duration::from_millis(500));

    // Act
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    advance(Duration::from_millis(100)).await;
    tx.unbounded_send(TokioTimestamped::new(person_bob(), timer.now()))?;

    // Assert
    assert_no_element_emitted(&mut delayed, 0).await;

    // Act
    advance(Duration::from_millis(400)).await;

    // Assert
    assert_eq!(
        unwrap_stream(&mut delayed, 100).await.unwrap().value,
        person_alice()
    );

    // Act
    advance(Duration::from_millis(100)).await;

    // Assert
    assert_eq!(
        unwrap_stream(&mut delayed, 100).await.unwrap().value,
        person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_delay_pending_until_upstream_ends() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut delayed = stream.delay(Duration::from_millis(500));

    // Act
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    drop(tx);

    // Assert
    assert_no_element_emitted(&mut delayed, 0).await;

    // Act
    advance(Duration::from_millis(500)).await;

    // Assert
    assert_eq!(
        unwrap_stream(&mut delayed, 100).await.unwrap().value,
        person_alice()
    );

    // Assert
    assert!(delayed.next().await.is_none());

    Ok(())
}
