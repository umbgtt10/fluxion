// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use async_channel::unbounded;
use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream_time::{SampleExt, TokioTimestamped};
use fluxion_test_utils::{helpers::recv_timeout, test_channel_with_errors, TestData};
use futures::StreamExt;
use std::time::Duration;
use tokio::spawn;
use tokio::time::pause;

#[tokio::test]
async fn test_sample_propagates_errors() -> anyhow::Result<()> {
    // Arrange
    pause();

    let (tx, stream) = test_channel_with_errors::<TokioTimestamped<TestData>>();
    let sampled = stream.sample(Duration::from_millis(100));

    let (result_tx, result_rx) = unbounded();

    spawn(async move {
        let mut stream = sampled;
        while let Some(item) = stream.next().await {
            result_tx.try_send(item).unwrap();
        }
    });

    // Act & Assert
    let error = FluxionError::stream_error("Test Error");
    tx.try_send(StreamItem::Error(error.clone()))?;
    assert!(matches!(
        recv_timeout(&result_rx, 100).await.unwrap(),
        StreamItem::Error(e) if e.to_string() == "Stream processing error: Test Error"
    ));

    Ok(())
}
