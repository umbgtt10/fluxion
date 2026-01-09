// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_runtime::impls::tokio::TokioTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream_time::{TimeoutExt, TokioTimestamped};
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
async fn test_timeout_returns_pending_while_waiting() -> anyhow::Result<()> {
    // This test covers the Poll::Pending branch when waiting for next value
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut timeout_stream = stream.timeout(Duration::from_millis(500));

    // Send first value - resets timer
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    assert_eq!(
        unwrap_stream(&mut timeout_stream, 100).await.unwrap().value,
        person_alice()
    );

    // Should return Pending because stream has no values and timer hasn't expired yet (line 150 in timeout.rs)
    assert_no_element_emitted(&mut timeout_stream, 0).await;

    // Send next value before timeout
    advance(Duration::from_millis(300)).await;
    tx.unbounded_send(TokioTimestamped::new(person_bob(), timer.now()))?;
    assert_eq!(
        unwrap_stream(&mut timeout_stream, 100).await.unwrap().value,
        person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_timeout_pending_without_values() -> anyhow::Result<()> {
    // This test covers Poll::Pending when no values received yet
    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut timeout_stream = stream.timeout(Duration::from_millis(500));

    // Should return Pending - no values yet
    assert_no_element_emitted(&mut timeout_stream, 0).await;

    // Stream still works normally after
    let timer = TokioTimer;
    pause();
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    assert_eq!(
        unwrap_stream(&mut timeout_stream, 100).await.unwrap().value,
        person_alice()
    );

    Ok(())
}

#[tokio::test]
async fn test_timeout_pending_then_timer_expires() -> anyhow::Result<()> {
    // Test transition from Pending to timeout error
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut timeout_stream = stream.timeout(Duration::from_millis(500));

    // Send value to start timer
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    assert_eq!(
        unwrap_stream(&mut timeout_stream, 100).await.unwrap().value,
        person_alice()
    );

    // Advance partway - should be Pending
    advance(Duration::from_millis(300)).await;
    assert_no_element_emitted(&mut timeout_stream, 0).await;

    // Advance past timeout
    advance(Duration::from_millis(200)).await;

    // Should get timeout error
    let result = unwrap_stream(&mut timeout_stream, 100).await;
    assert!(result.is_error());

    Ok(())
}

#[tokio::test]
async fn test_timeout_pending_with_stream_end() -> anyhow::Result<()> {
    // Test Poll::Pending followed by stream end
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();
    let mut timeout_stream = stream.timeout(Duration::from_millis(500));

    // Send value
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    assert_eq!(
        unwrap_stream(&mut timeout_stream, 100).await.unwrap().value,
        person_alice()
    );

    // Should be Pending
    assert_no_element_emitted(&mut timeout_stream, 0).await;

    // End stream
    drop(tx);

    // Stream should end gracefully (no timeout error)
    assert!(timeout_stream.next().await.is_none());

    Ok(())
}
