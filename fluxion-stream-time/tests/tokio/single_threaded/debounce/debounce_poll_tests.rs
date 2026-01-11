// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_runtime::impls::tokio::TokioTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream_time::{DebounceExt, TokioTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel,
    test_data::{person_alice, person_bob},
    TestData,
};
use futures::StreamExt;
use std::time::Duration;
use tokio::time::{advance, pause};

#[tokio::test]
async fn test_debounce_returns_pending_while_waiting() -> anyhow::Result<()> {
    // This test covers the Poll::Pending branch when the sleep timer is still running
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut debounced = stream.debounce(Duration::from_millis(500));

    // Send a value to start the debounce timer
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;

    // Should return Pending because timer hasn't expired yet (Poll::Pending branch line 151-152)
    assert_no_element_emitted(&mut debounced, 0).await;

    // Advance past debounce and verify value eventually emits
    advance(Duration::from_millis(500)).await;
    assert_eq!(
        unwrap_stream(&mut debounced, 100).await.unwrap().value,
        person_alice()
    );

    Ok(())
}

#[tokio::test]
async fn test_debounce_pending_without_timer() -> anyhow::Result<()> {
    // This test covers the Poll::Pending branch when no value has been received yet
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut debounced = stream.debounce(Duration::from_millis(500));

    // Should return Pending - no values received yet (line 183-187 in debounce.rs)
    assert_no_element_emitted(&mut debounced, 0).await;

    // Now verify stream works normally
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    assert_no_element_emitted(&mut debounced, 0).await; // Still pending during debounce
    advance(Duration::from_millis(500)).await;
    assert_eq!(
        unwrap_stream(&mut debounced, 100).await.unwrap().value,
        person_alice()
    );

    Ok(())
}

#[tokio::test]
async fn test_debounce_pending_then_new_value_resets() -> anyhow::Result<()> {
    // Test that a pending timer is properly reset when a new value arrives
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut debounced = stream.debounce(Duration::from_millis(500));

    // Send first value
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;

    // Should be pending
    assert_no_element_emitted(&mut debounced, 0).await;

    // Advance partway through debounce period
    advance(Duration::from_millis(300)).await;

    // Send new value - this should reset the timer
    tx.unbounded_send(TokioTimestamped::new(person_bob(), timer.now()))?;

    // Should still be pending with new timer
    assert_no_element_emitted(&mut debounced, 0).await;

    // Advance only 300ms more (600ms total from first value, but only 300ms from second)
    advance(Duration::from_millis(300)).await;

    // Should still be pending (need 500ms from Bob)
    assert_no_element_emitted(&mut debounced, 0).await;

    // Advance remaining time
    advance(Duration::from_millis(200)).await;

    // Now Bob should be emitted
    assert_eq!(
        unwrap_stream(&mut debounced, 100).await.unwrap().value,
        person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_debounce_pending_with_stream_end() -> anyhow::Result<()> {
    // Test Poll::Pending followed by stream end with pending value
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut debounced = stream.debounce(Duration::from_millis(500));

    // Send value
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;

    // Should be pending
    assert_no_element_emitted(&mut debounced, 0).await;

    // Drop sender to end stream
    drop(tx);

    // Stream should now emit the pending value immediately
    assert_eq!(
        unwrap_stream(&mut debounced, 100).await.unwrap().value,
        person_alice()
    );

    // Next poll should return None
    assert!(debounced.next().await.is_none());

    Ok(())
}
