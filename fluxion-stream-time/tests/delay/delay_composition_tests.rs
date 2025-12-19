// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::IntoStream;
use fluxion_stream::prelude::*;
use fluxion_stream::MergedStream;
use fluxion_stream_time::timer::Timer;
use fluxion_stream_time::DelayExt;
use fluxion_stream_time::TokioTimer;
use fluxion_stream_time::TokioTimestamped;
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
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
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
        .delay(Duration::from_secs(1), timer.clone());

    // Act & Assert
    tx.send(TokioTimestamped::new(person_alice(), timer.now()))?;
    advance(Duration::from_millis(100)).await;
    assert_no_element_emitted(&mut processed, 100).await;

    advance(Duration::from_millis(900)).await;
    assert_eq!(
        unwrap_stream(&mut processed, 100).await.unwrap().value,
        person_bob()
    );

    tx.send(TokioTimestamped::new(person_charlie(), timer.now()))?;
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
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut processed = stream
        .filter_ordered(|data: &_| *data == person_alice() || *data == person_charlie())
        .delay(Duration::from_secs(1), timer.clone());

    // Act & Assert
    tx.send(TokioTimestamped::new(person_alice(), timer.now()))?;
    advance(Duration::from_millis(100)).await;
    assert_no_element_emitted(&mut processed, 100).await;

    advance(Duration::from_millis(900)).await;
    assert_eq!(
        unwrap_stream(&mut processed, 100).await.unwrap().value,
        person_alice()
    );

    tx.send(TokioTimestamped::new(person_bob(), timer.now()))?;
    tx.send(TokioTimestamped::new(person_charlie(), timer.now()))?;

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
    let timer = TokioTimer;
    pause();

    let (tx1, stream1) = test_channel::<TokioTimestamped<TestData>>();
    let (tx2, stream2) = test_channel::<TokioTimestamped<TestData>>();
    let mut processed = MergedStream::seed::<TokioTimestamped<TestData>>(0)
        .merge_with(stream1, |value, state| {
            *state += 1;
            value // Pass through
        })
        .merge_with(stream2, |value, state| {
            *state += 1;
            value // Pass through
        })
        .into_stream()
        .delay(Duration::from_millis(200), timer.clone());

    // Act & Assert
    tx1.send(TokioTimestamped::new(person_alice(), timer.now()))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(100)).await;
    assert_no_element_emitted(&mut processed, 0).await;

    tx2.send(TokioTimestamped::new(person_bob(), timer.now()))?;
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
