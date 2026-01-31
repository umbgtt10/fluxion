// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::CancellationToken;
use fluxion_exec::subscribe::SubscribeExt;
use futures::channel::mpsc::unbounded;
use futures::lock::Mutex as FutureMutex;
use std::sync::Arc;
use tokio::spawn;
use tokio_stream::StreamExt as _;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Item {
    id: u32,
    value: String,
}

#[derive(Debug, thiserror::Error)]
#[error("Test error: {0}")]
struct TestError(String);

#[tokio::test]
async fn test_subscribe_example() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = unbounded::<Item>();
    let stream = rx;

    let results = Arc::new(FutureMutex::new(Vec::new()));
    let (notify_tx, mut notify_rx) = unbounded();

    let process_func = {
        let results = results.clone();
        let notify_tx = notify_tx.clone();
        move |item: Item, _ctx: CancellationToken| {
            let results = results.clone();
            let notify_tx = notify_tx.clone();
            async move {
                results.lock().await.push(item);
                let _ = notify_tx.unbounded_send(());
                Ok::<(), TestError>(())
            }
        }
    };

    let task = spawn(async move {
        stream
            .subscribe(process_func, |_| {}, None)
            .await
            .expect("subscribe should succeed");
    });

    let item1 = Item {
        id: 1,
        value: "Alice".to_string(),
    };
    let item2 = Item {
        id: 2,
        value: "Bob".to_string(),
    };
    let item3 = Item {
        id: 3,
        value: "Charlie".to_string(),
    };

    // Act
    tx.unbounded_send(item1.clone())?;
    notify_rx.next().await.unwrap();

    // Assert
    assert_eq!(*results.lock().await, vec![item1.clone()]);

    // Act
    tx.unbounded_send(item2.clone())?;
    notify_rx.next().await.unwrap();

    // Assert
    assert_eq!(*results.lock().await, vec![item1.clone(), item2.clone()]);

    // Act
    tx.unbounded_send(item3.clone())?;
    notify_rx.next().await.unwrap();

    // Assert
    assert_eq!(*results.lock().await, vec![item1, item2, item3]);

    // Clean up
    drop(tx);
    task.await.unwrap();

    Ok(())
}
