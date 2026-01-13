// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_runtime::impls::tokio::TokioTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream_time::{ThrottleExt, TokioTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel,
    test_data::{person_alice, person_bob, person_charlie},
    TestData,
};
use std::time::Duration;
use tokio::time::{advance, pause};

#[tokio::test]
async fn test_throttle_returns_pending_while_throttling() -> anyhow::Result<()> {
    // This test covers the Poll::Pending branch when throttling is active
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut throttled = stream.throttle(Duration::from_millis(500));

    // Send first value - should emit immediately
    tx.try_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    assert_eq!(
        unwrap_stream(&mut throttled, 100).await.unwrap().value,
        person_alice()
    );

    // Send second value immediately - should be dropped (throttling active)
    tx.try_send(TokioTimestamped::new(person_bob(), timer.now()))?;

    // Should return Pending because throttling is active (line 170 in throttle.rs)
    assert_no_element_emitted(&mut throttled, 0).await;

    // Advance past throttle period
    advance(Duration::from_millis(500)).await;

    // Send another value - should emit now
    tx.try_send(TokioTimestamped::new(person_charlie(), timer.now()))?;
    assert_eq!(
        unwrap_stream(&mut throttled, 100).await.unwrap().value,
        person_charlie()
    );

    Ok(())
}

#[tokio::test]
async fn test_throttle_pending_without_values() -> anyhow::Result<()> {
    // This test covers the Poll::Pending branch when no values have been received
    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut throttled = stream.throttle(Duration::from_millis(500));

    // Should return Pending - no values yet
    assert_no_element_emitted(&mut throttled, 0).await;
    // Should return Pending - no values yet
    assert_no_element_emitted(&mut throttled, 0).await;

    // Stream still works normally after
    let timer = TokioTimer;
    pause();
    tx.try_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    assert_eq!(
        unwrap_stream(&mut throttled, 100).await.unwrap().value,
        person_alice()
    );

    Ok(())
}

#[tokio::test]
async fn test_throttle_pending_during_throttle_period() -> anyhow::Result<()> {
    // Test that polling during throttle period returns Pending
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut throttled = stream.throttle(Duration::from_millis(500));

    // Send first value - emits immediately
    tx.try_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    assert_eq!(
        unwrap_stream(&mut throttled, 100).await.unwrap().value,
        person_alice()
    );

    // Advance partway through throttle period
    advance(Duration::from_millis(250)).await;

    // Send values during throttle - they should be dropped
    tx.try_send(TokioTimestamped::new(person_bob(), timer.now()))?;
    tx.try_send(TokioTimestamped::new(person_charlie(), timer.now()))?;

    // Should be pending
    assert_no_element_emitted(&mut throttled, 0).await;

    // Complete throttle period
    advance(Duration::from_millis(250)).await;

    // Still Pending because we dropped Bob and Charlie
    assert_no_element_emitted(&mut throttled, 0).await;

    Ok(())
}

#[tokio::test]
async fn test_throttle_pending_then_timer_expires() -> anyhow::Result<()> {
    // Test transition from Pending (throttling) to Ready when timer expires
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut throttled = stream.throttle(Duration::from_millis(500));

    // Send first value
    tx.try_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    assert_eq!(
        unwrap_stream(&mut throttled, 100).await.unwrap().value,
        person_alice()
    );

    // Send second value (will be dropped)
    tx.try_send(TokioTimestamped::new(person_bob(), timer.now()))?;

    // Advance to just before timer expires
    advance(Duration::from_millis(499)).await;

    // Should be Pending
    assert_no_element_emitted(&mut throttled, 0).await;

    // Advance past timer
    advance(Duration::from_millis(1)).await;

    // Send new value - should emit now
    tx.try_send(TokioTimestamped::new(person_charlie(), timer.now()))?;
    assert_eq!(
        unwrap_stream(&mut throttled, 100).await.unwrap().value,
        person_charlie()
    );

    Ok(())
}
