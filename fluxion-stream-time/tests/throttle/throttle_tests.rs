// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::FluxionStream;
use fluxion_stream_time::{ChronoStreamOps, ChronoTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_recv, recv_timeout},
    person::Person,
    test_channel,
    test_data::{person_alice, person_bob, person_charlie},
    TestData,
};
use futures::StreamExt;
use std::time::Duration;
use tokio::time::{advance, pause};
use tokio::{spawn, sync::mpsc::unbounded_channel};

#[tokio::test]
async fn test_throttle_with_chrono_timestamped() -> anyhow::Result<()> {
    // Arrange
    pause();

    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let throttle_duration = Duration::from_secs(1);
    let throttled = FluxionStream::new(stream).throttle(throttle_duration);

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
        person_alice()
    );

    tx.send(ChronoTimestamped::now(person_bob()))?;
    advance(Duration::from_millis(100)).await;
    assert_no_recv(&mut result_rx, 100).await;

    advance(Duration::from_millis(900)).await;
    tx.send(ChronoTimestamped::now(person_charlie()))?;

    assert_eq!(
        recv_timeout(&mut result_rx, 1000).await.unwrap(),
        person_charlie()
    );

    Ok(())
}

#[tokio::test]
async fn test_throttle_drops_intermediate_values() -> anyhow::Result<()> {
    // Arrange
    pause();

    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let throttle_duration = Duration::from_millis(100);
    let throttled = FluxionStream::new(stream).throttle(throttle_duration);

    let (result_tx, mut result_rx) = unbounded_channel();

    // Spawn a task to drive the stream
    spawn(async move {
        let mut stream = throttled;
        while let Some(item) = stream.next().await {
            result_tx.send(item.unwrap().value).unwrap();
        }
    });

    // Act & Assert
    tx.send(ChronoTimestamped::now(TestData::Person(Person::new(
        "Alice".to_string(),
        0,
    ))))?;
    for i in 1..10 {
        tx.send(ChronoTimestamped::now(TestData::Person(Person::new(
            "Alice".to_string(),
            i,
        ))))?;
    }

    // Allow processing of initial items
    tokio::task::yield_now().await;

    // Advance time past window
    advance(Duration::from_millis(100)).await;

    // Send 10 - emitted
    tx.send(ChronoTimestamped::now(TestData::Person(Person::new(
        "Alice".to_string(),
        10,
    ))))?;

    // Assert
    let mut results = Vec::new();
    // We expect 2 items: 0 and 10.

    // Give time for processing
    tokio::task::yield_now().await;

    if let Some(item) = recv_timeout(&mut result_rx, 1000).await {
        results.push(item);
    }
    if let Some(item) = recv_timeout(&mut result_rx, 1000).await {
        results.push(item);
    }

    assert_eq!(
        results,
        vec![
            TestData::Person(Person::new("Alice".to_string(), 0)),
            TestData::Person(Person::new("Alice".to_string(), 10))
        ]
    );

    Ok(())
}
