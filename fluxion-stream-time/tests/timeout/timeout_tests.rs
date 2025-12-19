// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::StreamItem;
use fluxion_stream_time::timer::Timer;
use fluxion_stream_time::TimeoutExt;
use fluxion_stream_time::TokioTimer;
use fluxion_stream_time::TokioTimestamped;
use fluxion_test_utils::{
    helpers::{assert_no_recv, recv_timeout},
    test_channel,
    test_data::{person_alice, person_bob},
    TestData,
};
use futures::StreamExt;
use std::time::Duration;
use tokio::time::{advance, pause};
use tokio::{spawn, sync::mpsc::unbounded_channel};

#[tokio::test]
async fn test_timeout_no_emission() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (_tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let timed_out = stream.timeout(Duration::from_millis(100), timer.clone());
    let (result_tx, mut result_rx) = unbounded_channel();

    spawn(async move {
        let mut stream = timed_out;
        while let Some(item) = stream.next().await {
            result_tx.send(item).unwrap();
        }
    });

    // Act & Assert
    advance(Duration::from_millis(150)).await;
    assert_eq!(
        recv_timeout(&mut result_rx, 100)
            .await
            .unwrap()
            .err()
            .unwrap()
            .to_string(),
        "Timeout error: Timeout"
    );

    assert_no_recv(&mut result_rx, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_timeout_with_emissions() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let timed_out = stream.timeout(Duration::from_millis(100), timer.clone());
    let (result_tx, mut result_rx) = unbounded_channel();

    spawn(async move {
        let mut stream = timed_out;
        while let Some(item) = stream.next().await {
            if let StreamItem::Value(val) = item {
                result_tx.send(val.value).unwrap();
            }
        }
    });

    // Act & Assert
    tx.send(TokioTimestamped::new(person_alice(), timer.now()))?;
    advance(Duration::from_millis(50)).await;
    assert_eq!(
        recv_timeout(&mut result_rx, 1000).await.unwrap(),
        person_alice()
    );

    tx.send(TokioTimestamped::new(person_bob(), timer.now()))?;
    advance(Duration::from_millis(50)).await;
    assert_eq!(
        recv_timeout(&mut result_rx, 1000).await.unwrap(),
        person_bob()
    );

    advance(Duration::from_millis(100)).await;
    assert_no_recv(&mut result_rx, 100).await;

    Ok(())
}
