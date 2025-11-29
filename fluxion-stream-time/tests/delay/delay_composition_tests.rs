// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::{FluxionStream, MergedStream};
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
async fn test_delay_chaining_with_map_ordered() -> anyhow::Result<()> {
    // Arrange
    pause();

    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let delay_duration = Duration::from_secs(1);

    // Chain map_ordered then delay - transform the data before delaying
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
        .delay(delay_duration);

    // Act & Assert
    tx.send(ChronoTimestamped::now(person_alice()))?;
    advance(Duration::from_millis(100)).await;
    assert_no_element_emitted(&mut processed, 100).await;

    advance(Duration::from_millis(900)).await;
    assert_eq!(
        unwrap_stream(&mut processed, 100).await.unwrap().value,
        person_bob()
    );

    tx.send(ChronoTimestamped::now(person_charlie()))?;
    advance(Duration::from_millis(100)).await;
    assert_no_element_emitted(&mut processed, 100).await;

    advance(Duration::from_millis(900)).await;
    assert_eq!(
        unwrap_stream(&mut processed, 100).await.unwrap().value,
        person_charlie()
    );

    Ok(())
}

#[tokio::test]
async fn test_delay_chaining_with_filter_ordered() -> anyhow::Result<()> {
    // Arrange
    pause();

    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let delay_duration = Duration::from_secs(1);

    // Chain filter_ordered then delay - keep only Alice and Charlie
    let mut processed = FluxionStream::new(stream)
        .filter_ordered(|data: &_| *data == person_alice() || *data == person_charlie())
        .delay(delay_duration);

    // Act & Assert
    tx.send(ChronoTimestamped::now(person_alice()))?;
    advance(Duration::from_millis(100)).await;
    assert_no_element_emitted(&mut processed, 100).await;

    advance(Duration::from_millis(900)).await;
    assert_eq!(
        unwrap_stream(&mut processed, 100).await.unwrap().value,
        person_alice()
    );

    tx.send(ChronoTimestamped::now(person_bob()))?;
    tx.send(ChronoTimestamped::now(person_charlie()))?;

    advance(Duration::from_millis(100)).await;
    assert_no_element_emitted(&mut processed, 100).await;

    advance(Duration::from_millis(1000)).await;
    assert_eq!(
        unwrap_stream(&mut processed, 100).await.unwrap().value,
        person_charlie()
    );

    Ok(())
}

#[tokio::test]
async fn test_merge_with_then_delay() -> anyhow::Result<()> {
    // Arrange
    pause();

    let (tx1, stream1) = test_channel::<ChronoTimestamped<TestData>>();
    let (tx2, stream2) = test_channel::<ChronoTimestamped<TestData>>();
    let delay_duration = Duration::from_millis(200);

    // Merge streams with state (count emissions)
    // We use MergedStream to merge two streams and then apply delay
    let mut processed = MergedStream::seed::<ChronoTimestamped<TestData>>(0)
        .merge_with(stream1, |value, state| {
            *state += 1;
            value // Pass through
        })
        .merge_with(stream2, |value, state| {
            *state += 1;
            value // Pass through
        })
        .into_fluxion_stream()
        .delay(delay_duration);

    // Act & Assert
    tx1.send(ChronoTimestamped::now(person_alice()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(100)).await;
    assert_no_element_emitted(&mut processed, 0).await;

    tx2.send(ChronoTimestamped::now(person_bob()))?;
    advance(Duration::from_millis(100)).await;
    assert_eq!(
        unwrap_stream(&mut processed, 100).await.unwrap().value,
        person_alice()
    );

    assert_no_element_emitted(&mut processed, 0).await;
    advance(Duration::from_millis(100)).await;
    assert_eq!(
        unwrap_stream(&mut processed, 100).await.unwrap().value,
        person_bob()
    );

    Ok(())
}
