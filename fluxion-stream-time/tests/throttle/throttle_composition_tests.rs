// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::FluxionStream;
use fluxion_stream_time::{ChronoStreamOps, ChronoTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_recv, recv_timeout},
    person::Person,
    test_channel,
    test_data::{person_alice, person_bob, person_charlie, person_diane},
    TestData,
};
use futures::StreamExt;
use std::time::Duration;
use tokio::time::{advance, pause};
use tokio::{spawn, sync::mpsc::unbounded_channel};

#[tokio::test]
async fn test_throttle_chained_with_map() -> anyhow::Result<()> {
    // Arrange
    pause();

    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let throttle_duration = Duration::from_millis(100);

    // Map then Throttle
    let throttled = FluxionStream::new(stream)
        .map_ordered(|x| {
            let val = if let TestData::Person(p) = x.value {
                TestData::Person(Person::new(p.name, p.age * 2))
            } else {
                x.value
            };
            ChronoTimestamped::new(val, x.timestamp)
        })
        .throttle(throttle_duration);

    let (result_tx, mut result_rx) = unbounded_channel();

    spawn(async move {
        let mut stream = throttled;
        while let Some(item) = stream.next().await {
            result_tx.send(item.unwrap().value).unwrap();
        }
    });

    // Act & Assert
    tx.send(ChronoTimestamped::now(person_alice()))?;
    assert_eq!(
        recv_timeout(&mut result_rx, 1000).await.unwrap(),
        TestData::Person(Person::new("Alice".to_string(), 50))
    );

    tx.send(ChronoTimestamped::now(person_bob()))?;
    assert_no_recv(&mut result_rx, 100).await;

    tx.send(ChronoTimestamped::now(person_charlie()))?;
    assert_eq!(
        recv_timeout(&mut result_rx, 1000).await.unwrap(),
        TestData::Person(Person::new("Charlie".to_string(), 70))
    );

    Ok(())
}

#[tokio::test]
async fn test_throttle_chained_with_throttle() -> anyhow::Result<()> {
    // Arrange
    pause();

    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();

    let throttled = FluxionStream::new(stream)
        .throttle(Duration::from_millis(100))
        .throttle(Duration::from_millis(200));

    let (result_tx, mut result_rx) = unbounded_channel();

    spawn(async move {
        let mut stream = throttled.map(|x| x.map(|v| v.value));
        while let Some(item) = stream.next().await {
            result_tx.send(item.unwrap()).unwrap();
        }
    });

    // Act & Assert
    tx.send(ChronoTimestamped::now(person_alice()))?;
    assert_eq!(
        recv_timeout(&mut result_rx, 1000).await.unwrap(),
        person_alice()
    );

    advance(Duration::from_millis(49)).await;
    tx.send(ChronoTimestamped::now(person_bob()))?;
    assert_no_recv(&mut result_rx, 50).await;

    advance(Duration::from_millis(10)).await;
    tx.send(ChronoTimestamped::now(person_charlie()))?;
    assert_no_recv(&mut result_rx, 50).await;

    advance(Duration::from_millis(50)).await;
    tx.send(ChronoTimestamped::now(person_diane()))?;
    assert_eq!(
        recv_timeout(&mut result_rx, 1000).await.unwrap(),
        person_diane()
    );

    Ok(())
}

#[tokio::test]
async fn test_throttle_chained_with_take_while_with() -> anyhow::Result<()> {
    // Arrange
    pause();

    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<ChronoTimestamped<bool>>();

    let throttle_duration = Duration::from_millis(100);

    let throttled = FluxionStream::new(stream)
        .take_while_with(filter_stream, |&condition| condition)
        .throttle(throttle_duration);

    let (result_tx, mut result_rx) = unbounded_channel();

    spawn(async move {
        let mut stream = throttled.map(|item| item.map(|x| x.value));
        while let Some(item) = stream.next().await {
            result_tx.send(item.unwrap()).unwrap();
        }
    });

    // Act & Assert
    filter_tx.send(ChronoTimestamped::now(true))?;
    tx.send(ChronoTimestamped::now(person_alice()))?;
    assert_eq!(
        recv_timeout(&mut result_rx, 1000).await.unwrap(),
        person_alice()
    );

    tx.send(ChronoTimestamped::now(person_bob()))?;
    assert_no_recv(&mut result_rx, 50).await;

    advance(Duration::from_millis(100)).await;
    tx.send(ChronoTimestamped::now(person_charlie()))?;
    assert_eq!(
        recv_timeout(&mut result_rx, 1000).await.unwrap(),
        person_charlie()
    );

    filter_tx.send(ChronoTimestamped::now(false))?;
    advance(Duration::from_millis(100)).await;
    tx.send(ChronoTimestamped::now(person_diane()))?;
    assert_no_recv(&mut result_rx, 100).await;

    Ok(())
}
