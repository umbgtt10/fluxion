// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use chrono::Duration;
use fluxion_stream::FluxionStream;
use fluxion_stream_time::{ChronoStreamOps, ChronoTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel,
    test_data::{person_alice, person_bob, person_charlie},
    TestData,
};
use tokio::{
    task::yield_now,
    time::{advance, pause},
};

#[tokio::test]
async fn test_debounce_emits_after_quiet_period() -> anyhow::Result<()> {
    pause(); // Mock time for instant test execution

    // Arrange
    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let debounce_duration = Duration::milliseconds(500);
    let mut debounced = FluxionStream::new(stream).debounce(debounce_duration);

    // Act - Send first value
    tx.send(ChronoTimestamped::now(person_alice()))?;

    // Yield to let debounce stream poll and create the timer
    yield_now().await;

    // Assert - Should NOT arrive immediately
    assert_no_element_emitted(&mut debounced, 0).await;

    // Advance 100ms
    advance(std::time::Duration::from_millis(100)).await;
    yield_now().await; // Allow tasks to process
    assert_no_element_emitted(&mut debounced, 0).await;

    // Assert - Should NOT arrive after 400ms total (still within debounce window)
    advance(std::time::Duration::from_millis(300)).await;
    yield_now().await;
    assert_no_element_emitted(&mut debounced, 0).await;

    // Assert - Should arrive after full 500ms of quiet period
    advance(std::time::Duration::from_millis(100)).await;
    yield_now().await;
    let result = unwrap_stream(&mut debounced, 100).await.unwrap();
    assert_eq!(result.value, person_alice());

    Ok(())
}

#[tokio::test]
async fn test_debounce_resets_on_new_value() -> anyhow::Result<()> {
    tokio::time::pause(); // Mock time for instant test execution

    // Arrange
    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let debounce_duration = Duration::milliseconds(500);
    let mut debounced = FluxionStream::new(stream).debounce(debounce_duration);

    // Act - Send Alice
    tx.send(ChronoTimestamped::now(person_alice()))?;

    // Yield to let debounce stream poll and create the timer
    yield_now().await;

    // Advance 300ms (not enough to emit)
    advance(std::time::Duration::from_millis(300)).await;
    yield_now().await;
    assert_no_element_emitted(&mut debounced, 0).await;

    // Act - Send Bob (resets the timer, Alice is discarded)
    tx.send(ChronoTimestamped::now(person_bob()))?;
    yield_now().await; // Let debounce poll and reset timer

    // Advance another 300ms (600ms total from Alice, but only 300ms from Bob)
    advance(std::time::Duration::from_millis(300)).await;
    yield_now().await;
    assert_no_element_emitted(&mut debounced, 0).await;

    // Advance final 200ms (500ms from Bob)
    advance(std::time::Duration::from_millis(200)).await;
    yield_now().await;

    // Assert - Should emit Bob, not Alice
    assert_eq!(
        unwrap_stream(&mut debounced, 100).await.unwrap().value,
        person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_debounce_multiple_resets() -> anyhow::Result<()> {
    pause(); // Mock time for instant test execution

    // Arrange
    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let debounce_duration = Duration::milliseconds(500);
    let mut debounced = FluxionStream::new(stream).debounce(debounce_duration);

    // Act - Rapid-fire three values
    tx.send(ChronoTimestamped::now(person_alice()))?;

    // Yield to let debounce poll
    yield_now().await;

    advance(std::time::Duration::from_millis(100)).await;
    yield_now().await;
    tx.send(ChronoTimestamped::now(person_bob()))?;
    yield_now().await; // Let debounce poll and reset

    advance(std::time::Duration::from_millis(100)).await;
    yield_now().await;
    tx.send(ChronoTimestamped::now(person_charlie()))?;
    yield_now().await; // Let debounce poll and reset

    // Assert - Nothing should have been emitted yet
    advance(std::time::Duration::from_millis(100)).await;
    yield_now().await;
    assert_no_element_emitted(&mut debounced, 0).await;

    // Advance final 400ms (500ms from Charlie)
    advance(std::time::Duration::from_millis(400)).await;
    yield_now().await;

    // Assert - Should emit only Charlie (last value)
    let result = unwrap_stream(&mut debounced, 100).await.unwrap();
    assert_eq!(result.value, person_charlie());

    Ok(())
}

#[tokio::test]
async fn test_debounce_emits_pending_on_stream_end() -> anyhow::Result<()> {
    pause(); // Mock time for instant test execution

    // Arrange
    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let debounce_duration = Duration::milliseconds(500);
    let mut debounced = FluxionStream::new(stream).debounce(debounce_duration);

    // Act - Send value
    tx.send(ChronoTimestamped::now(person_alice()))?;

    // Yield to let debounce stream poll and create the timer
    yield_now().await;

    // Advance only 200ms (not enough to emit normally)
    advance(std::time::Duration::from_millis(200)).await;
    yield_now().await;
    assert_no_element_emitted(&mut debounced, 0).await;

    // Act - Drop sender to end stream
    drop(tx);

    // Assert - Should emit pending value immediately when stream ends
    yield_now().await;
    let result = unwrap_stream(&mut debounced, 100).await.unwrap();
    assert_eq!(result.value, person_alice());

    Ok(())
}
