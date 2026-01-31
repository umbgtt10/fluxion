// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_runtime::impls::tokio::TokioTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream_time::{ThrottleExt, TokioTimestamped};
use fluxion_test_utils::helpers::{assert_no_element_emitted, test_channel, unwrap_stream};
use fluxion_test_utils::test_data::TestData;
use fluxion_test_utils::{
    helpers::{assert_no_recv, recv_timeout},
    person::Person,
    test_data::{person_alice, person_bob, person_charlie},
};
use futures::channel::mpsc::unbounded;
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
    let (result_tx, mut result_rx) = unbounded();

    spawn(async move {
        let mut stream = throttled;
        while let Some(item) = stream.next().await {
            result_tx.unbounded_send(item.unwrap().value).unwrap();
        }
    });

    // Act
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;

    // Assert
    assert_eq!(
        recv_timeout(&mut result_rx, 1000).await.unwrap(),
        person_alice()
    );

    // Act
    tx.unbounded_send(TokioTimestamped::new(person_bob(), timer.now()))?;
    advance(Duration::from_millis(100)).await;

    // Assert
    assert_no_recv(&mut result_rx, 100).await;

    // Act
    advance(Duration::from_millis(900)).await;
    tx.unbounded_send(TokioTimestamped::new(person_charlie(), timer.now()))?;

    // Assert
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
    let mut throttled = stream.throttle(Duration::from_millis(100));

    // Act
    tx.unbounded_send(TokioTimestamped::new(
        TestData::Person(Person::new("Alice".to_string(), 0)),
        timer.now(),
    ))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut throttled, 100).await.unwrap().value,
        TestData::Person(Person::new("Alice".to_string(), 0))
    );

    // Act
    for i in 1..10 {
        tx.unbounded_send(TokioTimestamped::new(
            TestData::Person(Person::new("Alice".to_string(), i)),
            timer.now(),
        ))?;

        // Assert
        assert_no_element_emitted(&mut throttled, 0).await;
    }

    // Act
    advance(Duration::from_millis(99)).await;

    // Assert
    assert_no_element_emitted(&mut throttled, 0).await;

    // Act
    advance(Duration::from_millis(1)).await;
    tx.unbounded_send(TokioTimestamped::new(
        TestData::Person(Person::new("Alice".to_string(), 10)),
        timer.now(),
    ))?;

    // Assert
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

    // Act
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    advance(Duration::from_millis(100)).await;

    // Assert
    assert_eq!(
        unwrap_stream(&mut throttled, 100).await.unwrap().value,
        person_alice()
    );

    // Act
    tx.unbounded_send(TokioTimestamped::new(person_bob(), timer.now()))?;
    advance(Duration::from_millis(100)).await;

    // Assert
    assert_eq!(
        unwrap_stream(&mut throttled, 100).await.unwrap().value,
        person_bob()
    );

    // Act
    tx.unbounded_send(TokioTimestamped::new(person_charlie(), timer.now()))?;
    advance(Duration::from_millis(100)).await;

    // Assert
    assert_eq!(
        unwrap_stream(&mut throttled, 100).await.unwrap().value,
        person_charlie()
    );

    Ok(())
}
