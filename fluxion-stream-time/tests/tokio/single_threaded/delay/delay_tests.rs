// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_runtime::impls::tokio::TokioTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream_time::{DelayExt, TokioTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    person::Person,
    test_channel,
    test_data::{person_alice, person_bob},
    unwrap_value, TestData,
};
use futures::StreamExt;
use std::time::Duration;
use tokio::spawn;
use tokio::time::{advance, pause};

#[tokio::test]
async fn test_delay_with_instant_timestamped() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut delayed = stream.delay(Duration::from_secs(1));

    // Act & Assert
    tx.try_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    advance(Duration::from_millis(100)).await;
    assert_no_element_emitted(&mut delayed, 100).await;

    advance(Duration::from_millis(900)).await;
    assert_eq!(
        unwrap_stream(&mut delayed, 100).await.unwrap().value,
        person_alice()
    );

    tx.try_send(TokioTimestamped::new(person_bob(), timer.now()))?;
    advance(Duration::from_millis(100)).await;
    assert_no_element_emitted(&mut delayed, 100).await;

    advance(Duration::from_millis(900)).await;
    assert_eq!(
        unwrap_stream(&mut delayed, 100).await.unwrap().value,
        person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_delay_preserves_order() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut delayed = stream.delay(Duration::from_millis(10));

    let count = 100;
    let (result_tx, result_rx) = test_channel::<TokioTimestamped<TestData>>();

    // Spawn a task to drive the stream
    spawn(async move {
        while let Some(item) = delayed.next().await {
            result_tx.try_send(item.unwrap()).unwrap();
        }
    });

    // Act
    for i in 0..count {
        tx.try_send(TokioTimestamped::new(
            TestData::Person(Person::new(format!("Person_{}", i), i as u32)),
            timer.now(),
        ))?;
    }
    // Drop tx to signal end of stream
    drop(tx);

    // Advance time to ensure all delays expire
    advance(Duration::from_millis(100)).await;

    // Assert
    let mut results = Vec::with_capacity(count as usize);
    let mut result_rx = Box::pin(result_rx);
    while let Some(item) = result_rx.next().await {
        results.push(unwrap_value(Some(item)).value);
    }

    assert_eq!(results.len(), count as usize);

    for i in 0..count {
        let expected = TestData::Person(Person::new(format!("Person_{}", i), i as u32));
        assert_eq!(
            results[i as usize], expected,
            "Order mismatch at index {}",
            i
        );
    }

    Ok(())
}
