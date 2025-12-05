// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::HasTimestamp;
use fluxion_stream::{CombineLatestExt, FluxionStream};
use fluxion_stream_time::{ChronoStreamOps, ChronoTimestamped};
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
    pause();

    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let debounce_duration = Duration::from_millis(500);

    // Chain map_ordered then debounce - transform the data before debouncing
    let mut processed = FluxionStream::new(stream)
        .map_ordered(|item: ChronoTimestamped<_>| {
            // Transform Alice to Bob
            let transformed = if item.value == person_alice() {
                person_bob()
            } else {
                item.value.clone()
            };
            ChronoTimestamped::new(transformed, item.timestamp)
        })
        .debounce(debounce_duration);

    // Act & Assert
    tx.send(ChronoTimestamped::now(person_alice()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(200)).await;
    tx.send(ChronoTimestamped::now(person_charlie()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(300)).await;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(200)).await;
    assert_eq!(
        unwrap_stream(&mut processed, 100).await.unwrap().value,
        person_charlie()
    );

    tx.send(ChronoTimestamped::now(person_alice()))?;
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
    pause();

    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let debounce_duration = Duration::from_millis(500);

    // Chain filter_ordered then debounce - keep only Alice and Charlie
    let mut processed = FluxionStream::new(stream)
        .filter_ordered(|data: &_| *data == person_alice() || *data == person_charlie())
        .debounce(debounce_duration);

    // Act & Assert
    tx.send(ChronoTimestamped::now(person_alice()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(100)).await;
    tx.send(ChronoTimestamped::now(person_bob()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(100)).await;
    tx.send(ChronoTimestamped::now(person_charlie()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(300)).await;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(200)).await;
    assert_eq!(
        unwrap_stream(&mut processed, 100).await.unwrap().value,
        person_charlie()
    );

    tx.send(ChronoTimestamped::now(person_alice()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(200)).await;
    tx.send(ChronoTimestamped::now(person_bob()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(300)).await;
    assert_eq!(
        unwrap_stream(&mut processed, 100).await.unwrap().value,
        person_alice()
    );

    tx.send(ChronoTimestamped::now(person_alice()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(500)).await;
    assert_eq!(
        unwrap_stream(&mut processed, 100).await.unwrap().value,
        person_alice()
    );

    Ok(())
}

#[tokio::test]
async fn test_debounce_then_delay() -> anyhow::Result<()> {
    // Arrange
    pause();

    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let debounce_duration = Duration::from_millis(300);
    let delay_duration = Duration::from_millis(200);

    // Chain debounce then delay
    let mut processed = FluxionStream::new(stream)
        .debounce(debounce_duration)
        .delay(delay_duration);

    // Act & Assert
    tx.send(ChronoTimestamped::now(person_alice()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(100)).await;
    tx.send(ChronoTimestamped::now(person_bob()))?;
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
    pause();

    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let delay_duration = Duration::from_millis(200);
    let debounce_duration = Duration::from_millis(300);

    // Chain delay then debounce
    let mut processed = FluxionStream::new(stream)
        .delay(delay_duration)
        .debounce(debounce_duration);

    // Act & Assert
    tx.send(ChronoTimestamped::now(person_alice()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(100)).await;
    tx.send(ChronoTimestamped::now(person_bob()))?;

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
    pause();

    let (tx1, stream1) = test_channel::<ChronoTimestamped<TestData>>();
    let (tx2, stream2) = test_channel::<ChronoTimestamped<TestData>>();
    let debounce_duration = Duration::from_millis(500);

    // Chain combine_latest then debounce
    let mut processed = stream1
        .combine_latest(vec![stream2], |_| true)
        .map_ordered(|state| {
            let ts = state.timestamp();
            ChronoTimestamped::new(state, ts)
        })
        .debounce(debounce_duration);

    // Act & Assert
    tx1.send(ChronoTimestamped::now(person_alice()))?;
    tx2.send(ChronoTimestamped::now(person_bob()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(200)).await;
    tx1.send(ChronoTimestamped::now(person_charlie()))?;
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
