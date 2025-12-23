// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream_time::prelude::*;
use fluxion_stream_time::timer::Timer;
use fluxion_stream_time::TokioTimer;
use fluxion_stream_time::TokioTimestamped;
use fluxion_test_utils::{
    helpers::{assert_no_recv, recv_timeout},
    test_channel,
    test_data::{person_alice, person_bob, person_charlie},
    TestData,
};
use futures::channel::mpsc::unbounded;
use futures::StreamExt;
use std::time::Duration;
use tokio::spawn;
use tokio::time::{advance, pause};

#[tokio::test]
async fn test_sample_emits_latest_in_window() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let sampled = stream.sample(Duration::from_millis(100));
    let (result_tx, mut result_rx) = unbounded();

    spawn(async move {
        let mut stream = sampled;
        while let Some(item) = stream.next().await {
            result_tx.unbounded_send(item.unwrap().value).unwrap();
        }
    });

    // Act & Assert
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    advance(Duration::from_millis(50)).await;
    tx.unbounded_send(TokioTimestamped::new(person_bob(), timer.now()))?;
    assert_no_recv(&mut result_rx, 10).await;

    advance(Duration::from_millis(50)).await;
    assert_eq!(
        recv_timeout(&mut result_rx, 1000).await.unwrap(),
        person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_sample_no_emission_if_no_value() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let sampled = stream.sample(Duration::from_millis(100));

    let (result_tx, mut result_rx) = unbounded();

    spawn(async move {
        let mut stream = sampled;
        while let Some(item) = stream.next().await {
            let _ = result_tx.unbounded_send(item.unwrap().value);
        }
    });

    // Act & Assert
    advance(Duration::from_millis(100)).await;
    assert_no_recv(&mut result_rx, 100).await;

    advance(Duration::from_millis(50)).await;
    tx.unbounded_send(TokioTimestamped::new(person_charlie(), timer.now()))?;
    advance(Duration::from_millis(50)).await;
    assert_eq!(
        recv_timeout(&mut result_rx, 1000).await.unwrap(),
        person_charlie()
    );

    Ok(())
}

#[tokio::test]
async fn test_sample_multiple_periods_without_values() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let sampled = stream.sample(Duration::from_millis(100));
    let (result_tx, mut result_rx) = unbounded();

    spawn(async move {
        let mut stream = sampled;
        while let Some(item) = stream.next().await {
            let _ = result_tx.unbounded_send(item.unwrap().value);
        }
    });

    // Act & Assert - multiple sample periods without values
    advance(Duration::from_millis(100)).await;
    assert_no_recv(&mut result_rx, 10).await;

    advance(Duration::from_millis(100)).await;
    assert_no_recv(&mut result_rx, 10).await;

    advance(Duration::from_millis(100)).await;
    assert_no_recv(&mut result_rx, 10).await;

    // Now send a value mid-period
    advance(Duration::from_millis(50)).await;
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;

    // Wait for sample period to complete
    advance(Duration::from_millis(50)).await;
    assert_eq!(
        recv_timeout(&mut result_rx, 1000).await.unwrap(),
        person_alice()
    );

    Ok(())
}

#[tokio::test]
async fn test_sample_stream_ends_with_pending_value() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let sampled = stream.sample(Duration::from_millis(100));
    let (result_tx, mut result_rx) = unbounded();

    spawn(async move {
        let mut stream = sampled;
        while let Some(item) = stream.next().await {
            let _ = result_tx.unbounded_send(item.unwrap().value);
        }
    });

    // Act - send value mid-period then close stream
    advance(Duration::from_millis(50)).await;
    tx.unbounded_send(TokioTimestamped::new(person_bob(), timer.now()))?;

    // Drop sender before sample period completes - stream ends with pending value
    drop(tx);

    // Assert - stream ends, pending value is NOT emitted (sample only emits on timer)
    advance(Duration::from_millis(100)).await;
    assert_no_recv(&mut result_rx, 100).await;

    Ok(())
}
