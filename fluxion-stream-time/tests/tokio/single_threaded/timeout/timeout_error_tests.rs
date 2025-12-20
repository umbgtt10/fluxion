// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream_time::prelude::*;
use fluxion_stream_time::timer::Timer;
use fluxion_stream_time::TokioTimer;
use fluxion_stream_time::TokioTimestamped;
use fluxion_test_utils::{
    helpers::recv_timeout, test_channel_with_errors, test_data::person_alice, TestData,
};
use futures::channel::mpsc::unbounded;
use futures::StreamExt;
use std::time::Duration;
use tokio::spawn;
use tokio::time::{advance, pause};

#[tokio::test]
async fn test_timeout_error_propagation() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    pause();

    let (tx, stream) = test_channel_with_errors::<TokioTimestamped<TestData>>();
    let timed_out = stream.timeout(Duration::from_millis(100));
    let (result_tx, mut result_rx) = unbounded();

    spawn(async move {
        let mut stream = timed_out;
        while let Some(item) = stream.next().await {
            result_tx.unbounded_send(item).unwrap();
        }
    });

    // Act & Assert
    let error = FluxionError::stream_error("Test Error");
    tx.unbounded_send(StreamItem::Error(error))?;
    assert_eq!(
        recv_timeout(&mut result_rx, 100)
            .await
            .unwrap()
            .err()
            .unwrap()
            .to_string(),
        "Stream processing error: Test Error"
    );

    advance(Duration::from_millis(50)).await;
    tx.unbounded_send(StreamItem::Value(TokioTimestamped::new(
        person_alice(),
        timer.now(),
    )))?;
    assert_eq!(
        recv_timeout(&mut result_rx, 100)
            .await
            .unwrap()
            .unwrap()
            .value,
        person_alice()
    );

    Ok(())
}
