// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_exec::subscribe_latest::SubscribeLatestExt;
use std::sync::Arc;
use tokio::spawn;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::Mutex as TokioMutex;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_util::sync::CancellationToken;

/// Example demonstrating subscribe_latest with automatic substitution
#[tokio::test]
async fn test_subscribe_latest_example() -> anyhow::Result<()> {
    #[derive(Debug, thiserror::Error)]
    #[error("Error")]
    struct Err;

    let (tx, rx) = unbounded_channel::<u32>();
    let completed = Arc::new(TokioMutex::new(Vec::new()));
    let (notify_tx, mut notify_rx) = unbounded_channel();
    let (gate_tx, gate_rx) = unbounded_channel::<()>();
    let gate_rx_shared = Arc::new(TokioMutex::new(Some(gate_rx)));
    let (start_tx, mut start_rx) = unbounded_channel::<()>();
    let start_tx_shared = Arc::new(TokioMutex::new(Some(start_tx)));

    let process = {
        let completed = completed.clone();
        move |id: u32, token: CancellationToken| {
            let completed = completed.clone();
            let notify_tx = notify_tx.clone();
            let gate_rx_shared = gate_rx_shared.clone();
            let start_tx_shared = start_tx_shared.clone();
            async move {
                if let Some(tx) = start_tx_shared.lock().await.take() {
                    tx.send(()).ok();
                }
                if let Some(mut rx) = gate_rx_shared.lock().await.take() {
                    rx.recv().await;
                }
                if !token.is_cancelled() {
                    completed.lock().await.push(id);
                    notify_tx.send(()).ok();
                }
                Ok::<(), Err>(())
            }
        }
    };

    spawn(async move {
        UnboundedReceiverStream::new(rx)
            .subscribe_latest(process, None::<fn(Err)>, None)
            .await
            .unwrap();
    });

    tx.send(1)?;
    start_rx.recv().await.unwrap();
    for i in 2..=5 {
        tx.send(i)?;
    }
    gate_tx.send(())?;
    notify_rx.recv().await.unwrap();
    notify_rx.recv().await.unwrap();

    let result = completed.lock().await;
    assert_eq!(*result, vec![1, 5]);

    Ok(())
}
