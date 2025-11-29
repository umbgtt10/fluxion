// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::StreamItem;
use fluxion_stream::FluxionStream;
use fluxion_stream_time::{ChronoStreamOps, ChronoTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_recv, recv_timeout},
    test_channel,
    test_data::{person_alice, person_bob},
    TestData,
};
use futures::StreamExt;
use tokio::time::{advance, pause};
use tokio::{spawn, sync::mpsc::unbounded_channel};

#[tokio::test]
async fn test_timeout_no_emission() -> anyhow::Result<()> {
    // Arrange
    pause();

    let (_tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let timeout_duration = std::time::Duration::from_millis(100);
    let timed_out = FluxionStream::new(stream).timeout(timeout_duration);

    let (result_tx, mut result_rx) = unbounded_channel();

    spawn(async move {
        let mut stream = timed_out;
        while let Some(item) = stream.next().await {
            result_tx.send(item).unwrap();
        }
    });

    // Act & Assert
    advance(std::time::Duration::from_millis(150)).await;
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
    pause();

    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let timeout_duration = std::time::Duration::from_millis(100);
    let timed_out = FluxionStream::new(stream).timeout(timeout_duration);

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
    tx.send(ChronoTimestamped::now(person_alice()))?;
    advance(std::time::Duration::from_millis(50)).await;
    assert_eq!(
        recv_timeout(&mut result_rx, 1000).await.unwrap(),
        person_alice()
    );

    tx.send(ChronoTimestamped::now(person_bob()))?;
    advance(std::time::Duration::from_millis(50)).await;
    assert_eq!(
        recv_timeout(&mut result_rx, 1000).await.unwrap(),
        person_bob()
    );

    advance(std::time::Duration::from_millis(100)).await;
    assert_no_recv(&mut result_rx, 100).await;

    Ok(())
}
