// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::FluxionStream;
use fluxion_stream_time::{ChronoStreamOps, ChronoTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_recv, recv_timeout},
    test_channel,
};
use futures::StreamExt;
use tokio::time::{advance, pause};
use tokio::{spawn, sync::mpsc::unbounded_channel};

#[tokio::test]
async fn test_throttle_chained_with_map() -> anyhow::Result<()> {
    // Arrange
    pause();
    let (tx, stream) = test_channel::<ChronoTimestamped<i32>>();
    let throttle_duration = std::time::Duration::from_millis(100);

    // Throttle then Map
    let throttled = FluxionStream::new(stream).throttle(throttle_duration);

    let (result_tx, mut result_rx) = unbounded_channel();

    spawn(async move {
        let mut stream = throttled.map(|item| item.map(|x| x.value * 2));
        while let Some(item) = stream.next().await {
            result_tx.send(item.unwrap()).unwrap();
        }
    });

    // Act & Assert
    tx.send(ChronoTimestamped::now(10))?;
    assert_eq!(recv_timeout(&mut result_rx, 1000).await.unwrap(), 20);

    tx.send(ChronoTimestamped::now(20))?;
    assert_no_recv(&mut result_rx, 100).await;

    tx.send(ChronoTimestamped::now(30))?;
    assert_eq!(recv_timeout(&mut result_rx, 1000).await.unwrap(), 60);

    Ok(())
}

#[tokio::test]
async fn test_throttle_chained_with_throttle() -> anyhow::Result<()> {
    // Arrange
    pause();

    let (tx, stream) = test_channel::<ChronoTimestamped<i32>>();

    let throttled = FluxionStream::new(stream)
        .throttle(std::time::Duration::from_millis(100))
        .throttle(std::time::Duration::from_millis(200));

    let (result_tx, mut result_rx) = unbounded_channel();

    spawn(async move {
        let mut stream = throttled.map(|x| x.map(|v| v.value));
        while let Some(item) = stream.next().await {
            result_tx.send(item.unwrap()).unwrap();
        }
    });

    // Act & Assert
    tx.send(ChronoTimestamped::now(1))?;
    assert_eq!(recv_timeout(&mut result_rx, 1000).await.unwrap(), 1);

    advance(std::time::Duration::from_millis(49)).await;
    tx.send(ChronoTimestamped::now(2))?;
    assert_no_recv(&mut result_rx, 50).await;

    advance(std::time::Duration::from_millis(10)).await;
    tx.send(ChronoTimestamped::now(3))?;
    assert_no_recv(&mut result_rx, 50).await;

    advance(std::time::Duration::from_millis(50)).await;
    tx.send(ChronoTimestamped::now(4))?;
    assert_eq!(recv_timeout(&mut result_rx, 1000).await.unwrap(), 4);

    Ok(())
}
