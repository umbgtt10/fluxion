// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::prelude::*;
use fluxion_stream_time::prelude::*;
use fluxion_stream_time::ChronoTimestamped;
use fluxion_test_utils::helpers::assert_no_recv;
use fluxion_test_utils::{helpers::recv_timeout, test_channel_with_errors, TestData};
use futures::StreamExt;
use std::time::Duration;
use tokio::time::{advance, pause};
use tokio::{spawn, sync::mpsc::unbounded_channel};

#[tokio::test]
async fn test_timeout_chained_error_propagation() -> anyhow::Result<()> {
    // Arrange
    pause();

    let (tx, stream) = test_channel_with_errors::<ChronoTimestamped<TestData>>();
    let timeout_duration = Duration::from_millis(100);

    let pipeline = stream
        .map_ordered(|item| ChronoTimestamped::new(item.value, item.timestamp))
        .timeout(timeout_duration);

    let (result_tx, mut result_rx) = unbounded_channel();

    spawn(async move {
        let mut stream = pipeline;
        while let Some(item) = stream.next().await {
            result_tx.send(item).unwrap();
        }
    });

    // Act & Assert
    let error = FluxionError::stream_error("Chain Error");
    tx.send(StreamItem::Error(error))?;
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
