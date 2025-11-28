// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use chrono::Duration;
use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::FluxionStream;
use fluxion_stream_time::{ChronoStreamOps, ChronoTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel_with_errors,
    test_data::{person_alice, person_bob},
    TestData,
};
use tokio::{
    task::yield_now,
    time::{advance, pause},
};

#[tokio::test]
async fn test_debounce_errors_pass_through() -> anyhow::Result<()> {
    pause(); // Mock time for instant test execution

    // Arrange
    let (tx, stream) = test_channel_with_errors::<ChronoTimestamped<TestData>>();
    let debounce_duration = Duration::milliseconds(500);
    let mut debounced = FluxionStream::new(stream).debounce(debounce_duration);

    // Act - Send value
    tx.send(StreamItem::Value(ChronoTimestamped::now(person_alice())))?;
    yield_now().await; // Allow debounce to poll
    assert_no_element_emitted(&mut debounced, 0).await; // Poll the stream to let debounce see the value

    // Assert - Should NOT arrive immediately (advance 300ms, not enough)
    advance(std::time::Duration::from_millis(300)).await;
    yield_now().await; // Allow tasks to process
    assert_no_element_emitted(&mut debounced, 0).await;

    // Act - Send error - should pass through immediately and discard pending value
    tx.send(StreamItem::Error(FluxionError::stream_error("test error")))?;

    // Assert - Error should arrive immediately (no time advance needed)
    yield_now().await; // Allow tasks to process
    let error_result = unwrap_stream(&mut debounced, 100).await;
    assert!(matches!(error_result, StreamItem::Error(_)));

    // Assert - The pending Alice value should have been discarded
    // Advance past where Alice would have emitted
    advance(std::time::Duration::from_millis(300)).await;
    yield_now().await;
    assert_no_element_emitted(&mut debounced, 0).await;

    // Act - Send another value after error
    tx.send(StreamItem::Value(ChronoTimestamped::now(person_bob())))?;
    yield_now().await; // Allow debounce to poll
    assert_no_element_emitted(&mut debounced, 0).await; // Poll the stream to let debounce see the value

    // Assert - Should NOT arrive immediately
    advance(std::time::Duration::from_millis(300)).await;
    yield_now().await;
    assert_no_element_emitted(&mut debounced, 0).await;

    // Assert - Should arrive after full debounce period
    advance(std::time::Duration::from_millis(200)).await;
    yield_now().await;
    assert_eq!(
        unwrap_stream(&mut debounced, 100).await.unwrap().value,
        person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_debounce_error_discards_pending() -> anyhow::Result<()> {
    pause(); // Mock time for instant test execution

    // Arrange
    let (tx, stream) = test_channel_with_errors::<ChronoTimestamped<TestData>>();
    let debounce_duration = Duration::milliseconds(500);
    let mut debounced = FluxionStream::new(stream).debounce(debounce_duration);

    // Act - Send Alice
    tx.send(StreamItem::Value(ChronoTimestamped::now(person_alice())))?;

    // Advance partway through debounce
    advance(std::time::Duration::from_millis(200)).await;
    yield_now().await;

    // Act - Send Bob (replaces Alice)
    tx.send(StreamItem::Value(ChronoTimestamped::now(person_bob())))?;

    // Advance a bit more
    advance(std::time::Duration::from_millis(200)).await;
    yield_now().await;

    // Act - Error arrives (should discard Bob)
    tx.send(StreamItem::Error(FluxionError::stream_error("test error")))?;

    // Assert - Error arrives immediately
    yield_now().await;
    let error_result = unwrap_stream(&mut debounced, 100).await;
    assert!(matches!(error_result, StreamItem::Error(_)));

    // Assert - Bob should NOT arrive even after advancing past debounce window
    advance(std::time::Duration::from_millis(500)).await;
    yield_now().await;
    assert_no_element_emitted(&mut debounced, 100).await;

    Ok(())
}
