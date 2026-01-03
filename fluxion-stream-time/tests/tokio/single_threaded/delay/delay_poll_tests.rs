// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use fluxion_stream_time::timer::Timer;
use fluxion_stream_time::{prelude::*, TokioTimer, TokioTimestamped};
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
async fn test_delay_returns_pending_while_waiting() -> anyhow::Result<()> {
    // This test covers the Poll::Pending branch when delay timers are in flight
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut delayed = stream.delay(Duration::from_millis(500));

    // Send value - starts delay
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;

    // Should return Pending because delay hasn't completed (line 177 in delay.rs)
    assert_no_element_emitted(&mut delayed, 0).await;

    // Advance past delay
    advance(Duration::from_millis(500)).await;

    // Value should now be ready
    assert_eq!(
        unwrap_stream(&mut delayed, 100).await.unwrap().value,
        person_alice()
    );

    Ok(())
}

#[tokio::test]
async fn test_delay_pending_without_values() -> anyhow::Result<()> {
    // This test covers Poll::Pending when no values received yet
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut delayed = stream.delay(Duration::from_millis(500));

    // Should return Pending - no values yet
    assert_no_element_emitted(&mut delayed, 0).await;

    // Now verify stream works normally
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    assert_no_element_emitted(&mut delayed, 0).await; // Still pending during delay
    advance(Duration::from_millis(500)).await;
    assert_eq!(
        unwrap_stream(&mut delayed, 100).await.unwrap().value,
        person_alice()
    );

    Ok(())
}

#[tokio::test]
async fn test_delay_pending_with_multiple_in_flight() -> anyhow::Result<()> {
    // Test Poll::Pending with multiple delayed values in flight
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut delayed = stream.delay(Duration::from_millis(500));

    // Send multiple values
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    advance(Duration::from_millis(100)).await;
    tx.unbounded_send(TokioTimestamped::new(person_bob(), timer.now()))?;

    // Should be Pending (Alice's delay not done yet)
    assert_no_element_emitted(&mut delayed, 0).await;

    // Advance to complete Alice's delay
    advance(Duration::from_millis(400)).await;
    assert_eq!(
        unwrap_stream(&mut delayed, 100).await.unwrap().value,
        person_alice()
    );

    // Complete Bob's delay (400ms already passed above, need 100ms more)
    advance(Duration::from_millis(100)).await;
    assert_eq!(
        unwrap_stream(&mut delayed, 100).await.unwrap().value,
        person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_delay_pending_until_upstream_ends() -> anyhow::Result<()> {
    // Test Poll::Pending when upstream ends with in-flight delays
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut delayed = stream.delay(Duration::from_millis(500));

    // Send value
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;

    // End stream
    drop(tx);

    // Should be Pending (delay not complete)
    assert_no_element_emitted(&mut delayed, 0).await;

    // Complete delay
    advance(Duration::from_millis(500)).await;

    // Value should emit
    assert_eq!(
        unwrap_stream(&mut delayed, 100).await.unwrap().value,
        person_alice()
    );

    // Stream should end
    assert!(delayed.next().await.is_none());

    Ok(())
}
