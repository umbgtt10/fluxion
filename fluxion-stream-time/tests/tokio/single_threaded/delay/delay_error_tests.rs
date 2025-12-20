// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream_time::prelude::*;
use fluxion_stream_time::timer::Timer;
use fluxion_stream_time::TokioTimer;
use fluxion_stream_time::TokioTimestamped;
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
    tx.unbounded_send(StreamItem::Value(TokioTimestamped::new(
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

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("test error")))?;
    assert!(matches!(
        unwrap_stream(&mut delayed, 100).await,
        StreamItem::Error(_)
    ));

    tx.unbounded_send(StreamItem::Value(TokioTimestamped::new(
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
