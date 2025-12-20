// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream_time::prelude::*;
use fluxion_stream_time::timer::Timer;
use fluxion_stream_time::{TokioTimer, TokioTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_recv, recv_timeout},
    test_channel_with_errors,
    test_data::{person_alice, person_bob},
    TestData,
};
use futures::channel::mpsc::unbounded;
use futures::StreamExt;
use std::time::Duration;
use tokio::spawn;
use tokio::time::{advance, pause};

#[tokio::test]
async fn test_throttle_propagates_errors_immediately() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel_with_errors::<TokioTimestamped<TestData>>();
    let throttled = stream.throttle(Duration::from_secs(1));
    let (result_tx, mut result_rx) = unbounded();

    spawn(async move {
        let mut stream = throttled;
        while let Some(item) = stream.next().await {
            let _ = result_tx.unbounded_send(item);
        }
    });

    // Act & Assert
    let error = FluxionError::stream_error("test error");
    tx.unbounded_send(StreamItem::Error(error.clone()))?;
    assert_eq!(
        recv_timeout(&mut result_rx, 1000)
            .await
            .unwrap()
            .err()
            .expect("Expected Error")
            .to_string(),
        error.to_string()
    );

    tx.unbounded_send(StreamItem::Value(TokioTimestamped::new(
        person_alice(),
        timer.now(),
    )))?;
    assert_eq!(
        recv_timeout(&mut result_rx, 1000)
            .await
            .unwrap()
            .ok()
            .expect("Expected Value")
            .value,
        person_alice()
    );

    Ok(())
}

#[tokio::test]
async fn test_throttle_propagates_errors_during_throttle() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel_with_errors::<TokioTimestamped<TestData>>();
    let throttled = stream.throttle(Duration::from_secs(1));
    let (result_tx, mut result_rx) = unbounded();

    spawn(async move {
        let mut stream = throttled;
        while let Some(item) = stream.next().await {
            let _ = result_tx.unbounded_send(item);
        }
    });

    // Act & Assert
    tx.unbounded_send(StreamItem::Value(TokioTimestamped::new(
        person_alice(),
        timer.now(),
    )))?;
    let _ = recv_timeout(&mut result_rx, 1000).await; // Consume Alice

    let error = FluxionError::stream_error("error during throttle");
    tx.unbounded_send(StreamItem::Error(error.clone()))?;
    assert_eq!(
        recv_timeout(&mut result_rx, 1000)
            .await
            .unwrap()
            .err()
            .expect("Expected Error")
            .to_string(),
        error.to_string()
    );

    tx.unbounded_send(StreamItem::Value(TokioTimestamped::new(
        person_bob(),
        timer.now(),
    )))?;
    advance(Duration::from_millis(100)).await;
    assert_no_recv(&mut result_rx, 100).await;

    Ok(())
}
