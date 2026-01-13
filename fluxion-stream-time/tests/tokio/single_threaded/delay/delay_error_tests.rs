// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_runtime::impls::tokio::TokioTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream_time::{DelayExt, TokioTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel_with_errors,
    test_data::{person_alice, person_bob},
    TestData,
};
use std::time::Duration;
use tokio::time::{advance, pause};

#[tokio::test]
async fn test_delay_errors_pass_through() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel_with_errors::<TokioTimestamped<TestData>>();
    let mut delayed = stream.delay(Duration::from_secs(1));

    // Act & Assert
    tx.try_send(StreamItem::Value(TokioTimestamped::new(
        person_alice(),
        timer.now(),
    )))?;
    advance(Duration::from_millis(100)).await;
    assert_no_element_emitted(&mut delayed, 100).await;

    advance(Duration::from_millis(900)).await;
    assert_eq!(
        unwrap_stream(&mut delayed, 100).await.unwrap().value,
        person_alice()
    );

    tx.try_send(StreamItem::Error(FluxionError::stream_error("test error")))?;
    assert!(matches!(
        unwrap_stream(&mut delayed, 100).await,
        StreamItem::Error(_)
    ));

    tx.try_send(StreamItem::Value(TokioTimestamped::new(
        person_bob(),
        timer.now(),
    )))?;
    advance(Duration::from_millis(100)).await;
    assert_no_element_emitted(&mut delayed, 100).await;

    advance(Duration::from_millis(900)).await;
    assert_eq!(
        unwrap_stream(&mut delayed, 100).await.unwrap().value,
        person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_delay_multiple_errors_with_values_in_flight() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel_with_errors::<TokioTimestamped<TestData>>();
    let mut delayed = stream.delay(Duration::from_secs(1));

    // Act - send value, then error while value is still delayed
    tx.try_send(StreamItem::Value(TokioTimestamped::new(
        person_alice(),
        timer.now(),
    )))?;

    // Send error while Alice is in flight
    tx.try_send(StreamItem::Error(FluxionError::stream_error("error 1")))?;

    // Assert - error passes through immediately
    assert!(matches!(
        unwrap_stream(&mut delayed, 100).await,
        StreamItem::Error(_)
    ));

    // Send another value and error
    tx.try_send(StreamItem::Value(TokioTimestamped::new(
        person_bob(),
        timer.now(),
    )))?;

    advance(Duration::from_millis(200)).await;

    tx.try_send(StreamItem::Error(FluxionError::stream_error("error 2")))?;

    // Assert - second error also passes through immediately
    assert!(matches!(
        unwrap_stream(&mut delayed, 100).await,
        StreamItem::Error(_)
    ));

    // Advance remaining time - Alice should still emit (error doesn't cancel in-flight)
    // Note: Alice's delay sleep was created at t=0 when we first polled for error 1
    // So we need to advance to t=1000 for the 1000ms sleep to complete
    advance(Duration::from_millis(800)).await; // t=200 + 800 = 1000
    assert_eq!(
        unwrap_stream(&mut delayed, 100).await.unwrap().value,
        person_alice()
    );

    // Advance for Bob
    // Bob's delay sleep was created at t=200 when we polled for error 2
    // So we need to advance to t=1200 for Bob's 1000ms sleep to complete
    // We're currently at t=1000, so advance 200ms
    advance(Duration::from_millis(200)).await; // t=1000 + 200 = 1200
    assert_eq!(
        unwrap_stream(&mut delayed, 100).await.unwrap().value,
        person_bob()
    );

    Ok(())
}
