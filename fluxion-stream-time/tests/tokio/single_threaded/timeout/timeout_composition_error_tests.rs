// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::prelude::*;
use fluxion_stream_time::prelude::*;
use fluxion_stream_time::TokioTimestamped;
use fluxion_test_utils::helpers::assert_no_recv;
use fluxion_test_utils::{helpers::recv_timeout, test_channel_with_errors, TestData};
use futures::channel::mpsc::unbounded;
use futures::StreamExt;
use std::time::Duration;
use tokio::spawn;
use tokio::time::{advance, pause};

#[tokio::test]
async fn test_timeout_chained_error_propagation() -> anyhow::Result<()> {
    // Arrange
    pause();

    let (tx, stream) = test_channel_with_errors::<TokioTimestamped<TestData>>();
    let pipeline = stream
        .map_ordered(|item| TokioTimestamped::new(item.value, item.timestamp))
        .timeout(Duration::from_millis(100));
    let (result_tx, mut result_rx) = unbounded();

    spawn(async move {
        let mut stream = pipeline;
        while let Some(item) = stream.next().await {
            result_tx.unbounded_send(item).unwrap();
        }
    });

    // Act & Assert
    let error = FluxionError::stream_error("Chain Error");
    tx.unbounded_send(StreamItem::Error(error))?;
    assert_eq!(
        recv_timeout(&mut result_rx, 100)
            .await
            .unwrap()
            .err()
            .unwrap()
            .to_string(),
        "Stream processing error: Chain Error"
    );

    advance(Duration::from_millis(50)).await;
    assert_no_recv(&mut result_rx, 50).await;

    advance(Duration::from_millis(100)).await;
    assert_eq!(
        recv_timeout(&mut result_rx, 100)
            .await
            .unwrap()
            .err()
            .unwrap()
            .to_string(),
        "Timeout error: Timeout"
    );

    Ok(())
}
