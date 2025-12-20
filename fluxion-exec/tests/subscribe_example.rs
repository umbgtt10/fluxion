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

/// Example test demonstrating subscribe usage
#[tokio::test]
async fn test_subscribe_example() -> anyhow::Result<()> {
    // Define a simple data type
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Item {
        id: u32,
        value: String,
    }

    #[derive(Debug, thiserror::Error)]
    #[error("Test error: {0}")]
    struct TestError(String);

    // Step 1: Create a stream
    let (tx, rx) = unbounded::<Item>();
    let stream = rx;

    // Step 2: Create a shared results container
    let results = Arc::new(FutureMutex::new(Vec::new()));
    let (notify_tx, mut notify_rx) = unbounded();

    // Step 3: Define the async processing function
    let process_func = {
        let results = results.clone();
        let notify_tx = notify_tx.clone();
        move |item: Item, _ctx: CancellationToken| {
            let results = results.clone();
            let notify_tx = notify_tx.clone();
            async move {
                // Process the item (in this example, just store it)
                results.lock().await.push(item);
                let _ = notify_tx.unbounded_send(()); // Signal completion
                Ok::<(), TestError>(())
            }
        }
    };

    // Step 4: Subscribe to the stream
    let task = spawn(async move {
        stream
            .subscribe(process_func, None, None::<fn(TestError)>)
            .await
            .expect("subscribe should succeed");
    });

    // Step 5: Publish items and assert results
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

    // Act & Assert
    tx.unbounded_send(item1.clone())?;
    notify_rx.next().await.unwrap();
    assert_eq!(*results.lock().await, vec![item1.clone()]);

    tx.unbounded_send(item2.clone())?;
    notify_rx.next().await.unwrap();
    assert_eq!(*results.lock().await, vec![item1.clone(), item2.clone()]);

    tx.unbounded_send(item3.clone())?;
    notify_rx.next().await.unwrap();
    assert_eq!(*results.lock().await, vec![item1, item2, item3]);

    // Clean up
    drop(tx);
    task.await.unwrap();

    Ok(())
}
