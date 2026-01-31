// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_runtime::impls::tokio::TokioTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream_time::{SampleExt, TokioTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, test_channel, unwrap_stream},
    test_data::{person_alice, person_bob, person_charlie, TestData},
};
use futures::StreamExt;
use std::time::Duration;
use tokio::time::{advance, pause};

#[tokio::test]
async fn test_sample_returns_pending_while_waiting() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut sampled = stream.sample(Duration::from_millis(500));

    // Act
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;

    // Assert
    assert_no_element_emitted(&mut sampled, 0).await;

    // Act
    advance(Duration::from_millis(500)).await;

    // Assert
    assert_eq!(
        unwrap_stream(&mut sampled, 100).await.unwrap().value,
        person_alice()
    );

    Ok(())
}

#[tokio::test]
async fn test_sample_pending_without_values() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut sampled = stream.sample(Duration::from_millis(500));

    // Act
    advance(Duration::from_millis(500)).await;

    // Assert
    assert_no_element_emitted(&mut sampled, 0).await;

    // Act
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    advance(Duration::from_millis(500)).await;

    // Assert
    assert_eq!(
        unwrap_stream(&mut sampled, 100).await.unwrap().value,
        person_alice()
    );

    Ok(())
}

#[tokio::test]
async fn test_sample_pending_with_value_updates() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut sampled = stream.sample(Duration::from_millis(500));

    // Act
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;

    // Assert
    assert_no_element_emitted(&mut sampled, 0).await;

    // Act
    advance(Duration::from_millis(250)).await;
    tx.unbounded_send(TokioTimestamped::new(person_bob(), timer.now()))?;

    // Assert
    assert_no_element_emitted(&mut sampled, 0).await;

    // Act
    advance(Duration::from_millis(200)).await;
    tx.unbounded_send(TokioTimestamped::new(person_charlie(), timer.now()))?;
    advance(Duration::from_millis(50)).await;

    // Assert
    assert_eq!(
        unwrap_stream(&mut sampled, 100).await.unwrap().value,
        person_charlie()
    );

    Ok(())
}

#[tokio::test]
async fn test_sample_pending_then_stream_ends() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut sampled = stream.sample(Duration::from_millis(500));

    // Act
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;

    // Assert
    assert_no_element_emitted(&mut sampled, 0).await;

    // Act
    drop(tx);

    // Assert
    assert!(sampled.next().await.is_none());

    Ok(())
}

#[tokio::test]
async fn test_sample_multiple_periods_all_pending() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut sampled = stream.sample(Duration::from_millis(500));

    // Act
    for _ in 0..3 {
        advance(Duration::from_millis(500)).await;

        // Assert
        assert_no_element_emitted(&mut sampled, 0).await;
    }

    // Act
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    advance(Duration::from_millis(500)).await;

    // Assert
    assert_eq!(
        unwrap_stream(&mut sampled, 100).await.unwrap().value,
        person_alice()
    );

    Ok(())
}
