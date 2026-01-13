// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use async_channel::unbounded;
use fluxion_core::StreamItem;
use fluxion_runtime::impls::tokio::TokioTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream::prelude::*;
use fluxion_stream_time::{ThrottleExt, TokioTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_recv, recv_timeout},
    person::Person,
    test_channel,
    test_data::{person_alice, person_bob, person_charlie, person_diane},
    TestData,
};
use futures::StreamExt;
use std::time::Duration;
use tokio::spawn;
use tokio::time::{advance, pause};

#[tokio::test]
async fn test_throttle_chained_with_map() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let throttled = stream
        .map_ordered(|x| {
            let val = if let TestData::Person(p) = x.value {
                TestData::Person(Person::new(p.name, p.age * 2))
            } else {
                x.value
            };
            TokioTimestamped::new(val, x.timestamp)
        })
        .throttle(Duration::from_millis(100));

    let (result_tx, result_rx) = unbounded();

    spawn(async move {
        let mut stream = throttled;
        while let Some(item) = stream.next().await {
            let _ = result_tx.try_send(item.unwrap().value);
        }
    });

    // Act & Assert
    tx.try_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    assert_eq!(
        recv_timeout(&result_rx, 1000).await.unwrap(),
        TestData::Person(Person::new("Alice".to_string(), 50))
    );

    tx.try_send(TokioTimestamped::new(person_bob(), timer.now()))?;
    assert_no_recv(&result_rx, 100).await;

    tx.try_send(TokioTimestamped::new(person_charlie(), timer.now()))?;
    assert_eq!(
        recv_timeout(&result_rx, 1000).await.unwrap(),
        TestData::Person(Person::new("Charlie".to_string(), 70))
    );

    Ok(())
}

#[tokio::test]
async fn test_throttle_chained_with_throttle() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let throttled = stream
        .throttle(Duration::from_millis(100))
        .throttle(Duration::from_millis(200));

    let (result_tx, result_rx) = unbounded();

    spawn(async move {
        let mut stream = throttled.map(|x| x.map(|v| v.value));
        while let Some(item) = stream.next().await {
            let _ = result_tx.try_send(item.unwrap());
        }
    });

    // Act & Assert
    tx.try_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    assert_eq!(
        recv_timeout(&result_rx, 1000).await.unwrap(),
        person_alice()
    );

    advance(Duration::from_millis(49)).await;
    tx.try_send(TokioTimestamped::new(person_bob(), timer.now()))?;
    assert_no_recv(&result_rx, 50).await;

    advance(Duration::from_millis(10)).await;
    tx.try_send(TokioTimestamped::new(person_charlie(), timer.now()))?;
    assert_no_recv(&result_rx, 50).await;

    advance(Duration::from_millis(50)).await;
    tx.try_send(TokioTimestamped::new(person_diane(), timer.now()))?;
    assert_eq!(
        recv_timeout(&result_rx, 1000).await.unwrap(),
        person_diane()
    );

    Ok(())
}

#[tokio::test]
async fn test_throttle_chained_with_take_while_with() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<TokioTimestamped<bool>>();

    let throttled = stream
        .take_while_with(filter_stream, |&condition| condition)
        .throttle(Duration::from_millis(100));

    let (result_tx, result_rx) = unbounded();

    spawn(async move {
        let mut stream = throttled.map(|item| item.map(|x| x.value));
        while let Some(item) = stream.next().await {
            let _ = result_tx.try_send(item.unwrap());
        }
    });

    // Act & Assert
    filter_tx.try_send(TokioTimestamped::new(true, timer.now()))?;
    tx.try_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    assert_eq!(
        recv_timeout(&result_rx, 1000).await.unwrap(),
        person_alice()
    );

    tx.try_send(TokioTimestamped::new(person_bob(), timer.now()))?;
    assert_no_recv(&result_rx, 50).await;

    advance(Duration::from_millis(100)).await;
    tx.try_send(TokioTimestamped::new(person_charlie(), timer.now()))?;
    assert_eq!(
        recv_timeout(&result_rx, 1000).await.unwrap(),
        person_charlie()
    );

    filter_tx.try_send(TokioTimestamped::new(false, timer.now()))?;
    advance(Duration::from_millis(100)).await;

    tx.try_send(TokioTimestamped::new(person_diane(), timer.now()))?;
    assert_no_recv(&result_rx, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_throttle_before_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let throttled = stream
        .throttle(Duration::from_millis(100))
        .map_ordered(|x| {
            let val = if let TestData::Person(p) = x.value {
                TestData::Person(Person::new(p.name, p.age * 2))
            } else {
                x.value
            };
            TokioTimestamped::new(val, x.timestamp)
        });

    let (result_tx, result_rx) = unbounded();

    spawn(async move {
        let mut stream = throttled;
        while let Some(item) = stream.next().await {
            if let StreamItem::Value(val) = item {
                let _ = result_tx.try_send(val.value);
            }
        }
    });

    // Act & Assert
    tx.try_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    advance(Duration::from_millis(1)).await;
    let received = recv_timeout(&result_rx, 100).await.unwrap();
    assert_eq!(
        received,
        TestData::Person(Person::new(String::from("Alice"), 50))
    );

    tx.try_send(TokioTimestamped::new(person_bob(), timer.now()))?;
    advance(Duration::from_millis(1)).await;
    assert_no_recv(&result_rx, 100).await;

    Ok(())
}
