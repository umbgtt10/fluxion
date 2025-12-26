// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::CancellationToken;
use fluxion_exec::ignore_errors;
use fluxion_exec::subscribe_latest::SubscribeLatestExt;
use futures::channel::mpsc::unbounded;
use futures::lock::Mutex as FutureMutex;
use std::sync::Arc;
use tokio::spawn;
use tokio_stream::StreamExt as _;

/// Example demonstrating subscribe_latest with automatic substitution
#[tokio::test]
async fn test_subscribe_latest_example() -> anyhow::Result<()> {
    #[derive(Debug, thiserror::Error)]
    #[error("Error")]
    struct Err;

    let (tx, rx) = unbounded::<u32>();
    let completed = Arc::new(FutureMutex::new(Vec::new()));
    let (notify_tx, mut notify_rx) = unbounded();
    let (gate_tx, gate_rx) = unbounded::<()>();
    let gate_rx_shared = Arc::new(FutureMutex::new(Some(gate_rx)));
    let (start_tx, mut start_rx) = unbounded::<()>();
    let start_tx_shared = Arc::new(FutureMutex::new(Some(start_tx)));

    let process = {
        let completed = completed.clone();
        move |id: u32, token: CancellationToken| {
            let completed = completed.clone();
            let notify_tx = notify_tx.clone();
            let gate_rx_shared = gate_rx_shared.clone();
            let start_tx_shared = start_tx_shared.clone();
            async move {
                if let Some(tx) = start_tx_shared.lock().await.take() {
                    tx.unbounded_send(()).ok();
                }
                if let Some(mut rx) = gate_rx_shared.lock().await.take() {
                    rx.next().await;
                }
                if !token.is_cancelled() {
                    completed.lock().await.push(id);
                    notify_tx.unbounded_send(()).ok();
                }
                Ok::<(), Err>(())
            }
        }
    };

    spawn(async move {
        rx.subscribe_latest(process, ignore_errors, None)
            .await
            .unwrap();
    });

    tx.unbounded_send(1)?;
    start_rx.next().await.unwrap();
    for i in 2..=5 {
        tx.unbounded_send(i)?;
    }
    gate_tx.unbounded_send(())?;
    notify_rx.next().await.unwrap();
    notify_rx.next().await.unwrap();

    let result = completed.lock().await;
    assert_eq!(*result, vec![1, 5]);

    Ok(())
}
