// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream_time::timer::Timer;
use fluxion_stream_time::{prelude::*, TokioTimer, TokioTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel,
    test_data::{person_alice, person_bob, person_charlie},
    TestData,
};
use futures::StreamExt;
use std::time::Duration;
use tokio::time::{advance, pause};

#[tokio::test]
async fn test_sample_returns_pending_while_waiting() -> anyhow::Result<()> {
    // This test covers the Poll::Pending branch when waiting for sample period
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut sampled = stream.sample(Duration::from_millis(500));

    // Send value
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;

    // Should return Pending because sample period hasn't elapsed (line 176 in sample.rs)
    assert_no_element_emitted(&mut sampled, 0).await;

    // Advance past sample period
    advance(Duration::from_millis(500)).await;

    // Should emit Alice
    assert_eq!(
        unwrap_stream(&mut sampled, 100).await.unwrap().value,
        person_alice()
    );

    Ok(())
}

#[tokio::test]
async fn test_sample_pending_without_values() -> anyhow::Result<()> {
    // This test covers Poll::Pending when sample period fires but no value received
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut sampled = stream.sample(Duration::from_millis(500));

    // Don't send anything - advance to sample period
    advance(Duration::from_millis(500)).await;

    // Should return Pending (no value to emit, line 173 in sample.rs)
    assert_no_element_emitted(&mut sampled, 0).await;

    // Send value in next period
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    advance(Duration::from_millis(500)).await;

    assert_eq!(
        unwrap_stream(&mut sampled, 100).await.unwrap().value,
        person_alice()
    );

    Ok(())
}

#[tokio::test]
async fn test_sample_pending_with_value_updates() -> anyhow::Result<()> {
    // Test that pending value gets updated while waiting for sample period
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut sampled = stream.sample(Duration::from_millis(500));

    // Send Alice
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;

    // Should be Pending
    assert_no_element_emitted(&mut sampled, 0).await;

    // Advance partway, send Bob (replaces Alice)
    advance(Duration::from_millis(250)).await;
    tx.unbounded_send(TokioTimestamped::new(person_bob(), timer.now()))?;

    // Still Pending
    assert_no_element_emitted(&mut sampled, 0).await;

    // Advance rest of period, send Charlie (replaces Bob)
    advance(Duration::from_millis(200)).await;
    tx.unbounded_send(TokioTimestamped::new(person_charlie(), timer.now()))?;

    // Complete sample period
    advance(Duration::from_millis(50)).await;

    // Should emit Charlie (latest value)
    assert_eq!(
        unwrap_stream(&mut sampled, 100).await.unwrap().value,
        person_charlie()
    );

    Ok(())
}

#[tokio::test]
async fn test_sample_pending_then_stream_ends() -> anyhow::Result<()> {
    // Test Poll::Pending followed by stream end
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut sampled = stream.sample(Duration::from_millis(500));

    // Send value
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;

    // Should be Pending
    assert_no_element_emitted(&mut sampled, 0).await;

    // End stream before sample period completes
    drop(tx);

    // Stream should end (pending value not emitted, needs sample period)
    assert!(sampled.next().await.is_none());

    Ok(())
}

#[tokio::test]
async fn test_sample_multiple_periods_all_pending() -> anyhow::Result<()> {
    // Test multiple sample periods with no values
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut sampled = stream.sample(Duration::from_millis(500));

    // Advance through 3 sample periods without sending values
    for _ in 0..3 {
        advance(Duration::from_millis(500)).await;
        // Each sample period fires but returns Pending (no value)
        assert_no_element_emitted(&mut sampled, 0).await;
    }

    // Send value and advance
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    advance(Duration::from_millis(500)).await;

    // Should emit now
    assert_eq!(
        unwrap_stream(&mut sampled, 100).await.unwrap().value,
        person_alice()
    );

    Ok(())
}
