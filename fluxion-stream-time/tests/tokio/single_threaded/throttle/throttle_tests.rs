// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use async_channel::unbounded;
use fluxion_runtime::impls::tokio::TokioTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream_time::{ThrottleExt, TokioTimestamped};
use fluxion_test_utils::assert_no_element_emitted;
use fluxion_test_utils::unwrap_stream;
use fluxion_test_utils::{
    helpers::{assert_no_recv, recv_timeout},
    person::Person,
    test_channel,
    test_data::{person_alice, person_bob, person_charlie},
    TestData,
};
use futures::StreamExt;
use std::time::Duration;
use tokio::spawn;
use tokio::time::{advance, pause};

#[tokio::test]
async fn test_throttle_with_instant_timestamped() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let throttled = stream.throttle(Duration::from_secs(1));
    let (result_tx, result_rx) = unbounded();

    spawn(async move {
        let mut stream = throttled;
        while let Some(item) = stream.next().await {
            result_tx.try_send(item.unwrap().value).unwrap();
        }
    });

    // Act & Assert
    tx.try_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    assert_eq!(
        recv_timeout(&result_rx, 1000).await.unwrap(),
        person_alice()
    );

    tx.try_send(TokioTimestamped::new(person_bob(), timer.now()))?;
    advance(Duration::from_millis(100)).await;
    assert_no_recv(&result_rx, 100).await;

    advance(Duration::from_millis(900)).await;
    tx.try_send(TokioTimestamped::new(person_charlie(), timer.now()))?;

    assert_eq!(
        recv_timeout(&result_rx, 1000).await.unwrap(),
        person_charlie()
    );

    Ok(())
}

#[tokio::test]
async fn test_throttle_drops_intermediate_values() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut throttled = stream.throttle(Duration::from_millis(100));

    // Act & Assert
    tx.try_send(TokioTimestamped::new(
        TestData::Person(Person::new("Alice".to_string(), 0)),
        timer.now(),
    ))?;
    assert_eq!(
        unwrap_stream(&mut throttled, 100).await.unwrap().value,
        TestData::Person(Person::new("Alice".to_string(), 0))
    );

    // Send intermediate values during throttle period - all should be dropped
    for i in 1..10 {
        tx.try_send(TokioTimestamped::new(
            TestData::Person(Person::new("Alice".to_string(), i)),
            timer.now(),
        ))?;
        // Verify nothing is emitted (still throttling)
        assert_no_element_emitted(&mut throttled, 0).await;
    }

    // Advance 99ms - still throttling
    advance(Duration::from_millis(99)).await;
    assert_no_element_emitted(&mut throttled, 0).await;

    // Advance final 1ms - throttle window complete
    advance(Duration::from_millis(1)).await;

    // Send next value - should be emitted since throttle expired
    tx.try_send(TokioTimestamped::new(
        TestData::Person(Person::new("Alice".to_string(), 10)),
        timer.now(),
    ))?;
    assert_eq!(
        unwrap_stream(&mut throttled, 100).await.unwrap().value,
        TestData::Person(Person::new("Alice".to_string(), 10))
    );

    Ok(())
}

#[tokio::test]
async fn test_throttle_zero_duration() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut throttled = stream.throttle(Duration::from_millis(0));

    // Act & Assert - zero duration means all values pass through (no throttling)
    tx.try_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    // Advance to allow zero-duration sleep to complete and unwrap_stream timeout to work
    advance(Duration::from_millis(100)).await;
    assert_eq!(
        unwrap_stream(&mut throttled, 100).await.unwrap().value,
        person_alice()
    );

    tx.try_send(TokioTimestamped::new(person_bob(), timer.now()))?;
    advance(Duration::from_millis(100)).await;
    assert_eq!(
        unwrap_stream(&mut throttled, 100).await.unwrap().value,
        person_bob()
    );

    tx.try_send(TokioTimestamped::new(person_charlie(), timer.now()))?;
    advance(Duration::from_millis(100)).await;
    assert_eq!(
        unwrap_stream(&mut throttled, 100).await.unwrap().value,
        person_charlie()
    );

    Ok(())
}
