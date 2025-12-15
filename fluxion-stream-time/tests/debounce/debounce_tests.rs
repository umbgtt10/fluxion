// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream_time::prelude::*;
use fluxion_stream_time::ChronoTimestamped;
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel,
    test_data::{person_alice, person_bob, person_charlie},
    TestData,
};
use std::time::Duration;
use tokio::time::{advance, pause};

#[tokio::test]
async fn test_debounce_emits_after_quiet_period() -> anyhow::Result<()> {
    // Arrange
    pause();

    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let debounce_duration = Duration::from_millis(500);
    let mut debounced = stream.debounce(debounce_duration);

    // Act & Assert
    tx.send(ChronoTimestamped::now(person_alice()))?;
    assert_no_element_emitted(&mut debounced, 0).await;

    advance(Duration::from_millis(100)).await;
    assert_no_element_emitted(&mut debounced, 0).await;

    advance(Duration::from_millis(300)).await;
    assert_no_element_emitted(&mut debounced, 0).await;

    advance(Duration::from_millis(100)).await;
    assert_eq!(
        unwrap_stream(&mut debounced, 100).await.unwrap().value,
        person_alice()
    );

    Ok(())
}

#[tokio::test]
async fn test_debounce_resets_on_new_value() -> anyhow::Result<()> {
    // Arrange
    pause();

    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let debounce_duration = Duration::from_millis(500);
    let mut debounced = stream.debounce(debounce_duration);

    // Act & Assert
    tx.send(ChronoTimestamped::now(person_alice()))?;
    advance(Duration::from_millis(300)).await;
    assert_no_element_emitted(&mut debounced, 0).await;

    tx.send(ChronoTimestamped::now(person_bob()))?;
    assert_no_element_emitted(&mut debounced, 0).await;

    advance(Duration::from_millis(300)).await;
    assert_no_element_emitted(&mut debounced, 0).await;

    advance(Duration::from_millis(200)).await;
    assert_eq!(
        unwrap_stream(&mut debounced, 100).await.unwrap().value,
        person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_debounce_multiple_resets() -> anyhow::Result<()> {
    // Arrange
    pause();

    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let debounce_duration = Duration::from_millis(500);
    let mut debounced = stream.debounce(debounce_duration);

    // Act & Assert
    tx.send(ChronoTimestamped::now(person_alice()))?;
    advance(Duration::from_millis(100)).await;

    tx.send(ChronoTimestamped::now(person_bob()))?;
    advance(Duration::from_millis(100)).await;

    tx.send(ChronoTimestamped::now(person_charlie()))?;
    advance(Duration::from_millis(100)).await;
    assert_no_element_emitted(&mut debounced, 0).await;

    advance(Duration::from_millis(400)).await;
    assert_eq!(
        unwrap_stream(&mut debounced, 100).await.unwrap().value,
        person_charlie()
    );

    Ok(())
}

#[tokio::test]
async fn test_debounce_emits_pending_on_stream_end() -> anyhow::Result<()> {
    // Arrange
    pause();

    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let debounce_duration = Duration::from_millis(500);
    let mut debounced = stream.debounce(debounce_duration);

    // Act & Assert
    tx.send(ChronoTimestamped::now(person_alice()))?;
    advance(Duration::from_millis(200)).await;
    assert_no_element_emitted(&mut debounced, 0).await;

    drop(tx);
    assert_eq!(
        unwrap_stream(&mut debounced, 100).await.unwrap().value,
        person_alice()
    );

    Ok(())
}
