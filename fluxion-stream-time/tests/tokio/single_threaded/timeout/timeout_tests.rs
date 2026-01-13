// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use async_channel::unbounded;
use fluxion_core::StreamItem;
use fluxion_runtime::impls::tokio::TokioTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream_time::{TimeoutExt, TokioTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_recv, recv_timeout},
    test_channel,
    test_data::{person_alice, person_bob},
    TestData,
};
use futures::StreamExt;
use std::time::Duration;
use tokio::spawn;
use tokio::time::{advance, pause};

#[tokio::test]
async fn test_timeout_no_emission() -> anyhow::Result<()> {
    // Arrange
    pause();

    let (_tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let timed_out = stream.timeout(Duration::from_millis(100));
    let (result_tx, result_rx) = unbounded();

    spawn(async move {
        let mut stream = timed_out;
        while let Some(item) = stream.next().await {
            result_tx.try_send(item).unwrap();
        }
    });

    // Act & Assert
    advance(Duration::from_millis(150)).await;
    assert_eq!(
        recv_timeout(&result_rx, 100)
            .await
            .unwrap()
            .err()
            .unwrap()
            .to_string(),
        "Timeout error: Timeout"
    );

    assert_no_recv(&result_rx, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_timeout_with_emissions() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let timed_out = stream.timeout(Duration::from_millis(100));
    let (result_tx, result_rx) = unbounded();

    spawn(async move {
        let mut stream = timed_out;
        while let Some(item) = stream.next().await {
            if let StreamItem::Value(val) = item {
                let _ = result_tx.try_send(val.value);
            }
        }
    });

    // Act & Assert
    tx.try_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    advance(Duration::from_millis(50)).await;
    assert_eq!(
        recv_timeout(&result_rx, 1000).await.unwrap(),
        person_alice()
    );

    tx.try_send(TokioTimestamped::new(person_bob(), timer.now()))?;
    advance(Duration::from_millis(50)).await;
    assert_eq!(recv_timeout(&result_rx, 1000).await.unwrap(), person_bob());

    advance(Duration::from_millis(100)).await;
    assert_no_recv(&result_rx, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_timeout_zero_duration() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let timed_out = stream.timeout(Duration::from_millis(0));
    let (result_tx, result_rx) = unbounded();

    spawn(async move {
        let mut stream = timed_out;
        while let Some(item) = stream.next().await {
            result_tx.try_send(item).unwrap();
        }
    });

    // Advance time to let zero-duration timeout fire and recv_timeout work
    advance(Duration::from_millis(100)).await;

    // Act & Assert - zero duration timeout should fire immediately
    assert_eq!(
        recv_timeout(&result_rx, 100)
            .await
            .unwrap()
            .err()
            .unwrap()
            .to_string(),
        "Timeout error: Timeout"
    );

    // Even if we send a value, stream is already terminated
    // The spawned task has exited and dropped result_tx, so this send will fail
    let _ = tx.try_send(TokioTimestamped::new(person_alice(), timer.now()));
    advance(Duration::from_millis(100)).await;
    assert_no_recv(&result_rx, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_timeout_timer_reset_on_each_emission() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let timed_out = stream.timeout(Duration::from_millis(100));
    let (result_tx, result_rx) = unbounded();

    spawn(async move {
        let mut stream = timed_out;
        while let Some(item) = stream.next().await {
            if let StreamItem::Value(val) = item {
                let _ = result_tx.try_send(val.value);
            }
        }
    });

    // Act & Assert - verify timer resets on each value
    for i in 0..5 {
        advance(Duration::from_millis(80)).await;
        tx.try_send(TokioTimestamped::new(
            if i % 2 == 0 {
                person_alice()
            } else {
                person_bob()
            },
            timer.now(),
        ))?;

        let received = recv_timeout(&result_rx, 100).await.unwrap();
        assert_eq!(
            received,
            if i % 2 == 0 {
                person_alice()
            } else {
                person_bob()
            }
        );
    }

    // Now stop sending - should timeout after 100ms
    advance(Duration::from_millis(100)).await;
    assert_no_recv(&result_rx, 100).await;

    Ok(())
}
