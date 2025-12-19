// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::prelude::*;
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
async fn test_emit_when_delay_error_propagation() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx_source, source) = test_channel_with_errors::<TokioTimestamped<TestData>>();
    let (tx_filter, filter) = test_channel_with_errors::<TokioTimestamped<TestData>>();
    let mut processed = source
        .emit_when(filter, |_| true)
        .delay(Duration::from_millis(200));

    // Act & Assert
    tx_filter.send(StreamItem::Value(TokioTimestamped::new(
        person_bob(),
        timer.now(),
    )))?;
    tx_source.send(StreamItem::Value(TokioTimestamped::new(
        person_alice(),
        timer.now(),
    )))?;
    assert_no_element_emitted(&mut processed, 0).await;

    let error = FluxionError::stream_error("filter error");
    tx_filter.send(StreamItem::Error(error.clone()))?;
    assert_eq!(
        unwrap_stream(&mut processed, 100)
            .await
            .err()
            .unwrap()
            .to_string(),
        error.to_string()
    );

    advance(Duration::from_millis(200)).await;
    assert_eq!(
        unwrap_stream(&mut processed, 100).await.unwrap().value,
        person_alice()
    );

    Ok(())
}
