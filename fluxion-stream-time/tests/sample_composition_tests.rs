// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::StreamItem;
use fluxion_stream::{FluxionStream, WithPrevious};
use fluxion_stream_time::{ChronoStreamOps, ChronoTimestamped};
use fluxion_test_utils::{
    helpers::recv_timeout,
    test_channel,
    test_data::{person_alice, person_bob, person_charlie},
    TestData,
};
use futures::StreamExt;
use tokio::time::{advance, pause};
use tokio::{spawn, sync::mpsc::unbounded_channel};

#[tokio::test]
async fn test_sample_chained_with_map() -> anyhow::Result<()> {
    // Arrange
    pause();

    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let sample_duration = std::time::Duration::from_millis(100);

    let pipeline = FluxionStream::new(stream)
        .map_ordered(|item| ChronoTimestamped::new(item.value, item.timestamp))
        .sample(sample_duration);

    let (result_tx, mut result_rx) = unbounded_channel();

    spawn(async move {
        let mut stream = pipeline;
        while let Some(item) = stream.next().await {
            // map returns StreamItem<T>, we want to unwrap it
            if let StreamItem::Value(val) = item {
                result_tx.send(val.value).unwrap();
            }
        }
    });

    // Act
    tx.send(ChronoTimestamped::now(person_alice()))?;
    advance(std::time::Duration::from_millis(50)).await;

    tx.send(ChronoTimestamped::now(person_bob()))?;
    advance(std::time::Duration::from_millis(50)).await;

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
    pause();

    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let sample_duration = std::time::Duration::from_millis(100);

    // Chain: combine_with_previous -> map -> sample
    let pipeline = FluxionStream::new(stream)
        .combine_with_previous()
        .map_ordered(|wp| {
            let timestamp = wp.current.timestamp;
            let current_val = wp.current.value;
            let previous_val = wp.previous.map(|p| p.value);
            ChronoTimestamped::new(WithPrevious::new(previous_val, current_val), timestamp)
        })
        .sample(sample_duration);

    let (result_tx, mut result_rx) = unbounded_channel();

    spawn(async move {
        let mut stream = pipeline;
        while let Some(item) = stream.next().await {
            if let StreamItem::Value(val) = item {
                let wp = val.value;
                result_tx.send((wp.previous, wp.current)).unwrap();
            }
        }
    });

    // Act & Assert
    tx.send(ChronoTimestamped::now(person_alice()))?;

    advance(std::time::Duration::from_millis(50)).await;
    tx.send(ChronoTimestamped::now(person_bob()))?;
    advance(std::time::Duration::from_millis(50)).await;
    assert_eq!(
        recv_timeout(&mut result_rx, 1000).await.unwrap(),
        (Some(person_alice()), person_bob())
    );

    advance(std::time::Duration::from_millis(50)).await;
    tx.send(ChronoTimestamped::now(person_charlie()))?;
    advance(std::time::Duration::from_millis(50)).await;

    assert_eq!(
        recv_timeout(&mut result_rx, 1000).await.unwrap(),
        (Some(person_bob()), person_charlie())
    );

    Ok(())
}
#[tokio::test]
async fn test_sample_chained_with_scan_ordered() -> anyhow::Result<()> {
    // Arrange
    pause();

    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let sample_duration = std::time::Duration::from_millis(100);

    let pipeline = FluxionStream::new(stream)
        .scan_ordered(0, |acc, item| {
            let val = match item {
                TestData::Person(p) => p.age,
                _ => 0,
            };
            *acc += val;
            *acc
        })
        .sample(sample_duration);

    let (result_tx, mut result_rx) = unbounded_channel();

    spawn(async move {
        let mut stream = pipeline;
        while let Some(item) = stream.next().await {
            if let StreamItem::Value(val) = item {
                result_tx.send(val.value).unwrap();
            }
        }
    });

    // Act and Assert
    tx.send(ChronoTimestamped::now(person_alice()))?;
    advance(std::time::Duration::from_millis(50)).await;
    tx.send(ChronoTimestamped::now(person_bob()))?;
    advance(std::time::Duration::from_millis(50)).await;
    assert_eq!(recv_timeout(&mut result_rx, 1000).await.unwrap(), 55);

    advance(std::time::Duration::from_millis(50)).await;
    tx.send(ChronoTimestamped::now(person_charlie()))?;
    advance(std::time::Duration::from_millis(50)).await;
    assert_eq!(recv_timeout(&mut result_rx, 1000).await.unwrap(), 90);

    Ok(())
}
