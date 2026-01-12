// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::HasTimestamp;
use fluxion_runtime::impls::tokio::TokioTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream::prelude::*;
use fluxion_stream_time::{DebounceExt, DelayExt, TokioTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel,
    test_data::{person_alice, person_bob, person_charlie},
    TestData,
};
use std::time::Duration;
use tokio::time::{advance, pause};

#[tokio::test]
async fn test_debounce_chaining_with_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();

    // Chain map_ordered then debounce - transform the data before debouncing
    let mut processed = stream
        .map_ordered(|item: TokioTimestamped<_>| {
            // Transform Alice to Bob
            let transformed = if item.value == person_alice() {
                person_bob()
            } else {
                item.value.clone()
            };
            TokioTimestamped::new(transformed, item.timestamp)
        })
        .debounce(Duration::from_millis(500));

    // Act & Assert
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(200)).await;
    tx.unbounded_send(TokioTimestamped::new(person_charlie(), timer.now()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(300)).await;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(200)).await;
    assert_eq!(
        unwrap_stream(&mut processed, 100).await.unwrap().value,
        person_charlie()
    );

    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(500)).await;
    assert_eq!(
        unwrap_stream(&mut processed, 100).await.unwrap().value,
        person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_debounce_chaining_with_filter_ordered() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();

    // Chain filter_ordered then debounce - keep only Alice and Charlie
    let mut processed = stream
        .filter_ordered(|data: &_| *data == person_alice() || *data == person_charlie())
        .debounce(Duration::from_millis(500));

    // Act & Assert
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(100)).await;
    tx.unbounded_send(TokioTimestamped::new(person_bob(), timer.now()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(100)).await;
    tx.unbounded_send(TokioTimestamped::new(person_charlie(), timer.now()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(300)).await;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(200)).await;
    assert_eq!(
        unwrap_stream(&mut processed, 100).await.unwrap().value,
        person_charlie()
    );

    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(200)).await;
    tx.unbounded_send(TokioTimestamped::new(person_bob(), timer.now()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(300)).await;
    assert_eq!(
        unwrap_stream(&mut processed, 100).await.unwrap().value,
        person_alice()
    );

    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(500)).await;
    assert_eq!(
        unwrap_stream(&mut processed, 100).await.unwrap().value,
        person_alice()
    );

    Ok(())
}
#[tokio::test]
async fn test_debounce_before_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();

    // Chain debounce then map_ordered - debounce first, transform after
    let mut processed =
        stream
            .debounce(Duration::from_millis(500))
            .map_ordered(|item: TokioTimestamped<_>| {
                // Transform Alice to Bob
                let transformed = if item.value == person_alice() {
                    person_bob()
                } else {
                    item.value.clone()
                };
                TokioTimestamped::new(transformed, item.timestamp)
            });

    // Act & Assert
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(200)).await;
    tx.unbounded_send(TokioTimestamped::new(person_charlie(), timer.now()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(300)).await;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(200)).await;
    assert_eq!(
        unwrap_stream(&mut processed, 100).await.unwrap().value,
        person_charlie()
    );

    Ok(())
}
#[tokio::test]
async fn test_debounce_then_delay() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut processed = stream
        .debounce(Duration::from_millis(300))
        .delay(Duration::from_millis(200));

    // Act & Assert
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(100)).await;
    tx.unbounded_send(TokioTimestamped::new(person_bob(), timer.now()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(200)).await;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(100)).await;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(200)).await;
    assert_eq!(
        unwrap_stream(&mut processed, 100).await.unwrap().value,
        person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_delay_then_debounce() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut processed = stream
        .delay(Duration::from_millis(200))
        .debounce(Duration::from_millis(300));

    // Act & Assert
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(100)).await;
    tx.unbounded_send(TokioTimestamped::new(person_bob(), timer.now()))?;

    advance(Duration::from_millis(200)).await;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(100)).await;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(300)).await;
    assert_eq!(
        unwrap_stream(&mut processed, 100).await.unwrap().value,
        person_alice()
    );

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_then_debounce() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx1, stream1) = test_channel::<TokioTimestamped<TestData>>();
    let (tx2, stream2) = test_channel::<TokioTimestamped<TestData>>();
    let mut processed = stream1
        .combine_latest(vec![stream2], |_| true)
        .map_ordered(|state| {
            let ts = state.timestamp();
            TokioTimestamped::new(state, ts)
        })
        .debounce(Duration::from_millis(500));

    // Act & Assert
    tx1.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    tx2.unbounded_send(TokioTimestamped::new(person_bob(), timer.now()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(200)).await;
    tx1.unbounded_send(TokioTimestamped::new(person_charlie(), timer.now()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(300)).await;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(200)).await;

    let item = unwrap_stream(&mut processed, 100).await.unwrap();
    let values = item.value.values();
    // stream1 is index 0, stream2 is index 1
    assert_eq!(values[0], person_charlie());
    assert_eq!(values[1], person_bob());

    Ok(())
}
