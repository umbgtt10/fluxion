// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::StreamItem;
use fluxion_stream::prelude::*;
use fluxion_stream_time::prelude::*;
use fluxion_stream_time::timer::Timer;
use fluxion_stream_time::InstantTimestamped;
use fluxion_stream_time::TokioTimer;
use fluxion_stream_time::TokioTimestamped;
use fluxion_test_utils::{
    helpers::recv_timeout,
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
async fn test_sample_chained_with_map() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let pipeline = stream
        .map_ordered(|item| TokioTimestamped::new(item.value, item.timestamp))
        .sample(Duration::from_millis(100));

    let (result_tx, mut result_rx) = unbounded();

    spawn(async move {
        let mut stream = pipeline;
        while let Some(item) = stream.next().await {
            // map returns StreamItem<T>, we want to unwrap it
            if let StreamItem::Value(val) = item {
                let _ = result_tx.unbounded_send(val.value);
            }
        }
    });

    // Act
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    advance(Duration::from_millis(50)).await;

    tx.unbounded_send(TokioTimestamped::new(person_bob(), timer.now()))?;
    advance(Duration::from_millis(50)).await;

    // Assert
    assert_eq!(
        recv_timeout(&mut result_rx, 1000).await.unwrap(),
        person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_sample_chained_with_combine_with_previous() -> anyhow::Result<()> {
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
            InstantTimestamped::new(WithPrevious::new(previous_val, current_val), timestamp)
        })
        .sample(Duration::from_millis(100));

    let (result_tx, mut result_rx) = unbounded();

    spawn(async move {
        let mut stream = pipeline;
        while let Some(item) = stream.next().await {
            if let StreamItem::Value(val) = item {
                let wp = val.value;
                let _ = result_tx.unbounded_send((wp.previous, wp.current));
            }
        }
    });

    // Act & Assert
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;

    advance(Duration::from_millis(50)).await;
    tx.unbounded_send(TokioTimestamped::new(person_bob(), timer.now()))?;
    advance(Duration::from_millis(50)).await;
    assert_eq!(
        recv_timeout(&mut result_rx, 1000).await.unwrap(),
        (Some(person_alice()), person_bob())
    );

    advance(Duration::from_millis(50)).await;
    tx.unbounded_send(TokioTimestamped::new(person_charlie(), timer.now()))?;
    advance(Duration::from_millis(50)).await;

    assert_eq!(
        recv_timeout(&mut result_rx, 1000).await.unwrap(),
        (Some(person_bob()), person_charlie())
    );

    Ok(())
}
#[tokio::test]
async fn test_sample_chained_with_scan_ordered() -> anyhow::Result<()> {
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
        .sample(Duration::from_millis(100));

    let (result_tx, mut result_rx) = unbounded::<TokioTimestamped<u32>>();

    spawn(async move {
        let mut stream = pipeline;
        while let Some(item) = stream.next().await {
            if let StreamItem::Value(val) = item {
                let _ = result_tx.unbounded_send(val);
            }
        }
    });

    // Act and Assert
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    advance(Duration::from_millis(50)).await;
    tx.unbounded_send(TokioTimestamped::new(person_bob(), timer.now()))?;
    advance(Duration::from_millis(50)).await;
    assert_eq!(recv_timeout(&mut result_rx, 1000).await.unwrap().value, 55);

    advance(Duration::from_millis(50)).await;
    tx.unbounded_send(TokioTimestamped::new(person_charlie(), timer.now()))?;
    advance(Duration::from_millis(50)).await;
    assert_eq!(recv_timeout(&mut result_rx, 1000).await.unwrap().value, 90);

    Ok(())
}
