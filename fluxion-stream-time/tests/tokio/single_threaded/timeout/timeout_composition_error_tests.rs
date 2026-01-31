// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::prelude::*;
use fluxion_stream_time::{TimeoutExt, TokioTimestamped};
use fluxion_test_utils::helpers::recv_timeout;
use fluxion_test_utils::helpers::{assert_no_recv, test_channel_with_errors};
use fluxion_test_utils::test_data::TestData;
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

    // Act
    let error = FluxionError::stream_error("Chain Error");
    tx.unbounded_send(StreamItem::Error(error))?;

    // Assert
    assert_eq!(
        recv_timeout(&mut result_rx, 100)
            .await
            .unwrap()
            .err()
            .unwrap()
            .to_string(),
        "Stream processing error: Chain Error"
    );

    // Act
    advance(Duration::from_millis(50)).await;

    // Assert
    assert_no_recv(&mut result_rx, 50).await;

    // Act
    advance(Duration::from_millis(100)).await;

    // Assert
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
