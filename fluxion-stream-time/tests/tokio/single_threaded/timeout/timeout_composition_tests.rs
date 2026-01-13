// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use async_channel::unbounded;
use fluxion_core::StreamItem;
use fluxion_runtime::impls::tokio::TokioTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream::prelude::*;
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
async fn test_timeout_chained_with_map() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let pipeline = stream
        .map_ordered(|item| TokioTimestamped::new(item.value, item.timestamp))
        .timeout(Duration::from_millis(100));
    let (result_tx, result_rx) = unbounded();

    spawn(async move {
        let mut stream = pipeline;
        while let Some(item) = stream.next().await {
            if let StreamItem::Value(val) = item {
                result_tx.try_send(val.value).unwrap();
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

    advance(Duration::from_millis(100)).await;
    assert_no_recv(&result_rx, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_timeout_chained_with_combine_with_previous() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let pipeline = stream
        .combine_with_previous()
        .map_ordered(|wp| {
            let timestamp = wp.current.timestamp;
            let current_val = wp.current.value;
            let previous_val = wp.previous.map(|p| p.value);
            TokioTimestamped::new(WithPrevious::new(previous_val, current_val), timestamp)
        })
        .timeout(Duration::from_millis(100));

    let (result_tx, result_rx) = unbounded();

    spawn(async move {
        let mut stream = pipeline;
        while let Some(item) = stream.next().await {
            if let StreamItem::Value(val) = item {
                let wp = val.value;
                let _ = result_tx.try_send((wp.previous, wp.current));
            }
        }
    });

    // Act & Assert
    tx.try_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    advance(Duration::from_millis(50)).await;
    assert_eq!(
        recv_timeout(&result_rx, 1000).await.unwrap(),
        (None, person_alice())
    );

    tx.try_send(TokioTimestamped::new(person_bob(), timer.now()))?;
    advance(Duration::from_millis(50)).await;
    assert_eq!(
        recv_timeout(&result_rx, 1000).await.unwrap(),
        (Some(person_alice()), person_bob())
    );

    advance(Duration::from_millis(100)).await;
    assert_no_recv(&result_rx, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_timeout_chained_with_scan_ordered() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let pipeline = stream
        .scan_ordered(0u32, |acc, item| {
            let val = match item {
                TestData::Person(p) => p.age,
                _ => 0,
            };
            *acc += val;
            *acc
        })
        .timeout(Duration::from_millis(100));

    let (result_tx, result_rx) = unbounded::<TokioTimestamped<u32>>();

    spawn(async move {
        let mut stream = pipeline;
        while let Some(item) = stream.next().await {
            if let StreamItem::Value(val) = item {
                let _ = result_tx.try_send(val);
            }
        }
    });

    // Act & Assert
    tx.try_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    advance(Duration::from_millis(50)).await;
    assert_eq!(recv_timeout(&result_rx, 1000).await.unwrap().value, 25);

    tx.try_send(TokioTimestamped::new(person_bob(), timer.now()))?;
    advance(Duration::from_millis(50)).await;
    assert_eq!(recv_timeout(&result_rx, 1000).await.unwrap().value, 55);

    advance(Duration::from_millis(100)).await;
    assert_no_recv(&result_rx, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_timeout_before_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let pipeline = stream
        .timeout(Duration::from_millis(100))
        .map_ordered(|item| TokioTimestamped::new(item.value, item.timestamp));
    let (result_tx, result_rx) = unbounded();

    spawn(async move {
        let mut stream = pipeline;
        while let Some(item) = stream.next().await {
            if let StreamItem::Value(val) = item {
                result_tx.try_send(val.value).unwrap();
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

    Ok(())
}
