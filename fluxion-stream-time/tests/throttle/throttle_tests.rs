// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream_time::timer::Timer;
use fluxion_stream_time::ThrottleExt;
use fluxion_stream_time::TokioTimer;
use fluxion_stream_time::TokioTimestamped;
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
use tokio::time::{advance, pause};
use tokio::{spawn, sync::mpsc::unbounded_channel};

#[tokio::test]
async fn test_throttle_with_instant_timestamped() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let throttled = stream.throttle(Duration::from_secs(1), timer.clone());
    let (result_tx, mut result_rx) = unbounded_channel();

    spawn(async move {
        let mut stream = throttled;
        while let Some(item) = stream.next().await {
            result_tx.send(item.unwrap().value).unwrap();
        }
    });

    // Act & Assert
    tx.send(TokioTimestamped::new(person_alice(), timer.now()))?;
    assert_eq!(
        recv_timeout(&mut result_rx, 1000).await.unwrap(),
        person_alice()
    );

    tx.send(TokioTimestamped::new(person_bob(), timer.now()))?;
    advance(Duration::from_millis(100)).await;
    assert_no_recv(&mut result_rx, 100).await;

    advance(Duration::from_millis(900)).await;
    tx.send(TokioTimestamped::new(person_charlie(), timer.now()))?;

    assert_eq!(
        recv_timeout(&mut result_rx, 1000).await.unwrap(),
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
    let mut throttled = stream.throttle(Duration::from_millis(100), timer.clone());

    // Act & Assert
    tx.send(TokioTimestamped::new(
        TestData::Person(Person::new("Alice".to_string(), 0)),
        timer.now(),
    ))?;
    assert_eq!(
        unwrap_stream(&mut throttled, 100).await.unwrap().value,
        TestData::Person(Person::new("Alice".to_string(), 0))
    );

    // Send intermediate values during throttle period - all should be dropped
    for i in 1..10 {
        tx.send(TokioTimestamped::new(
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
    tx.send(TokioTimestamped::new(
        TestData::Person(Person::new("Alice".to_string(), 10)),
        timer.now(),
    ))?;
    assert_eq!(
        unwrap_stream(&mut throttled, 100).await.unwrap().value,
        TestData::Person(Person::new("Alice".to_string(), 10))
    );

    Ok(())
}
