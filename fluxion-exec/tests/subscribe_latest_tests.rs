// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::CancellationToken;
use fluxion_exec::subscribe_latest::SubscribeLatestExt;
use fluxion_test_utils::test_data::{
    animal_ant, animal_cat, animal_dog, animal_spider, person_alice, person_bob, person_charlie,
    person_dave, person_diane, plant_rose, TestData,
};
use fluxion_test_utils::Sequenced;
use futures::channel::mpsc::unbounded;
use futures::lock::Mutex as FutureMutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::spawn;
use tokio::task::yield_now;
use tokio::time::{sleep, Duration};
use tokio_stream::StreamExt as _;

#[derive(Debug, thiserror::Error)]
#[error("Test error: {0}")]
struct TestError(String);

impl TestError {
    fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

#[tokio::test]
async fn test_subscribe_latest_no_skipping_no_error_no_cancellation() -> anyhow::Result<()> {
    // Arrange
    let collected_items = Arc::new(FutureMutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();
    let (notify_tx, mut notify_rx) = unbounded();

    let (tx, rx) = unbounded::<Sequenced<TestData>>();
    let stream = rx;
    let stream = stream.map(|timestamped| timestamped.value);

    let func = {
        let collected_items = collected_items_clone.clone();
        let notify_tx = notify_tx.clone();
        move |item, _| {
            let collected_items = collected_items.clone();
            let notify_tx = notify_tx.clone();
            async move {
                collected_items.lock().await.push(item);
                let _ = notify_tx.unbounded_send(());
                Ok::<(), TestError>(())
            }
        }
    };

    let error_callback = |err: TestError| {
        eprintln!("Error occurred: {err:?}");
    };

    spawn({
        async move {
            stream
                .subscribe_latest(func, error_callback, None)
                .await
                .expect("subscribe_latest should succeed");
        }
    });

    // Act - emit items one at a time, waiting for each to be processed
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    notify_rx.next().await.expect("Alice processed");

    tx.unbounded_send(Sequenced::new(person_bob()))?;
    notify_rx.next().await.expect("Bob processed");

    tx.unbounded_send(Sequenced::new(person_charlie()))?;
    notify_rx.next().await.expect("Charlie processed");

    tx.unbounded_send(Sequenced::new(person_diane()))?;
    notify_rx.next().await.expect("Diane processed");

    tx.unbounded_send(Sequenced::new(person_dave()))?;
    notify_rx.next().await.expect("Dave processed");

    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    notify_rx.next().await.expect("Dog processed");

    tx.unbounded_send(Sequenced::new(animal_cat()))?;
    notify_rx.next().await.expect("Cat processed");

    tx.unbounded_send(Sequenced::new(animal_ant()))?;
    notify_rx.next().await.expect("Ant processed");

    tx.unbounded_send(Sequenced::new(animal_spider()))?;
    notify_rx.next().await.expect("Spider processed");

    tx.unbounded_send(Sequenced::new(plant_rose()))?;
    notify_rx.next().await.expect("Rose processed");

    // Assert
    let processed = {
        let g = collected_items.lock().await;
        g.clone()
    };
    assert_eq!(processed.len(), 10,);
    assert_eq!(processed[0], person_alice());
    assert_eq!(processed[1], person_bob());
    assert_eq!(processed[2], person_charlie());
    assert_eq!(processed[3], person_diane());
    assert_eq!(processed[4], person_dave());
    assert_eq!(processed[5], animal_dog());
    assert_eq!(processed[6], animal_cat());
    assert_eq!(processed[7], animal_ant());
    assert_eq!(processed[8], animal_spider());
    assert_eq!(processed[9], plant_rose());
    drop(processed);

    Ok(())
}

#[tokio::test]
async fn test_subscribe_latest_with_skipping_no_error_no_cancellation() -> anyhow::Result<()> {
    // Arrange
    let collected_items = Arc::new(FutureMutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();
    let (notify_tx, mut notify_rx) = unbounded();

    // Barrier channel: only the FIRST processed item will wait on this
    let (gate_tx, gate_rx) = unbounded::<()>();
    let gate_rx_shared = Arc::new(FutureMutex::new(Some(gate_rx)));

    // Start channel: first processing signals when it starts (to avoid races)
    let (start_tx, mut start_rx) = unbounded::<()>();
    let start_tx_shared = Arc::new(FutureMutex::new(Some(start_tx)));

    let (tx, rx) = unbounded::<Sequenced<TestData>>();
    let stream = rx;
    let stream = stream.map(|timestamped| timestamped.value);

    let func = {
        let collected_items = collected_items_clone.clone();
        let notify_tx = notify_tx.clone();
        let gate_rx_shared = gate_rx_shared.clone();
        let start_tx_shared = start_tx_shared.clone();
        move |item, _| {
            let collected_items = collected_items.clone();
            let notify_tx = notify_tx.clone();
            let gate_rx_shared = gate_rx_shared.clone();
            let start_tx_shared = start_tx_shared.clone();
            async move {
                // Signal that the first processing has started (only once)
                let value = start_tx_shared.lock().await.take();
                if let Some(tx) = value {
                    let _ = tx.unbounded_send(());
                }
                // Only the first processing waits for the gate signal
                let value = gate_rx_shared.lock().await.take();
                if let Some(mut rx) = value {
                    // Wait for external signal to unblock the first item
                    let _ = rx.next().await;
                }

                collected_items.lock().await.push(item);
                let _ = notify_tx.unbounded_send(());
                Ok::<(), TestError>(())
            }
        }
    };

    let error_callback = |err: TestError| {
        eprintln!("Error occurred: {err:?}");
    };

    spawn({
        async move {
            stream
                .subscribe_latest(func, error_callback, None)
                .await
                .expect("subscribe_latest should succeed");
        }
    });

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    start_rx.next().await.expect("first processing started");

    // While the first item is blocked, send 4 more items rapidly
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(person_charlie()))?;
    tx.unbounded_send(Sequenced::new(person_diane()))?;
    tx.unbounded_send(Sequenced::new(person_dave()))?; // latest

    // Unblock the first processing, allowing it to complete
    let _ = gate_tx.unbounded_send(());

    // Wait for exactly two processed notifications (Alice, then latest Dave)
    notify_rx
        .next()
        .await
        .expect("Alice should be processed first");
    notify_rx
        .next()
        .await
        .expect("Latest (Dave) should be processed next");

    // Assert
    let processed = {
        let g = collected_items.lock().await;
        g.clone()
    };
    assert_eq!(processed.len(), 2,);
    assert_eq!(processed[0], person_alice(),);
    assert_eq!(processed[1], person_dave(),);
    assert!(!processed.contains(&person_bob()));
    assert!(!processed.contains(&person_charlie()));
    assert!(!processed.contains(&person_diane()));

    Ok(())
}

#[tokio::test]
async fn test_subscribe_latest_no_skipping_with_error_no_cancellation() -> anyhow::Result<()> {
    // Arrange
    let collected_items = Arc::new(FutureMutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();
    let (notify_tx, mut notify_rx) = unbounded();

    let (tx, rx) = unbounded::<Sequenced<TestData>>();
    let stream = rx;
    let stream = stream.map(|timestamped| timestamped.value);

    let func = {
        let collected_items = collected_items_clone.clone();
        let notify_tx = notify_tx.clone();
        move |item: TestData, _| {
            let collected_items = collected_items.clone();
            let notify_tx = notify_tx.clone();
            async move {
                if matches!(&item, TestData::Person(p) if p.name == "Bob" || p.name == "Dave") {
                    let _ = notify_tx.unbounded_send(());
                    return Err(TestError::new(format!("Failed to process {item:?}")));
                }

                collected_items.lock().await.push(item);
                let _ = notify_tx.unbounded_send(());

                Ok::<(), TestError>(())
            }
        }
    };

    let error_callback = |err: TestError| {
        eprintln!("Error occurred: {err:?}");
    };

    spawn({
        async move {
            stream
                .subscribe_latest(func, error_callback, None)
                .await
                .expect("subscribe_latest should succeed");
        }
    });

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    notify_rx.next().await.expect("Alice processed");

    tx.unbounded_send(Sequenced::new(person_bob()))?; // Error
    notify_rx.next().await.expect("Bob handled (error)");

    tx.unbounded_send(Sequenced::new(person_charlie()))?;
    notify_rx.next().await.expect("Charlie processed");

    tx.unbounded_send(Sequenced::new(person_dave()))?; // Error
    notify_rx.next().await.expect("Dave handled (error)");

    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    notify_rx.next().await.expect("Dog processed");

    // Assert
    let processed = {
        let g = collected_items.lock().await;
        g.clone()
    };
    assert_eq!(processed.len(), 3);
    assert!(processed.contains(&person_alice()),);
    assert!(processed.contains(&person_charlie()),);
    assert!(processed.contains(&animal_dog()),);
    assert!(!processed.contains(&person_bob()),);
    assert!(!processed.contains(&person_dave()),);

    Ok(())
}

#[tokio::test]
async fn test_subscribe_latest_no_skipping_no_errors_with_cancellation() -> anyhow::Result<()> {
    // Arrange
    let collected_items = Arc::new(FutureMutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();
    let (notify_tx, mut notify_rx) = unbounded();
    let (tx, rx) = unbounded::<Sequenced<TestData>>();
    let stream = rx;
    let stream = stream.map(|timestamped| timestamped.value);

    let func = {
        let collected_items = collected_items_clone.clone();
        let notify_tx = notify_tx.clone();
        move |item, ctx: CancellationToken| {
            let collected_items = collected_items.clone();
            let notify_tx = notify_tx.clone();
            async move {
                if ctx.is_cancelled() {
                    let _ = notify_tx.unbounded_send(());
                    return Ok(());
                }

                collected_items.lock().await.push(item);
                let _ = notify_tx.unbounded_send(());

                Ok::<(), TestError>(())
            }
        }
    };

    let error_callback = |_err: TestError| {
        eprintln!("Error occurred.");
    };

    let cancellation_token = CancellationToken::new();
    spawn({
        let cancellation_token = cancellation_token.clone();

        async move {
            stream
                .subscribe_latest(func, error_callback, Some(cancellation_token))
                .await
                .expect("subscribe_latest should succeed");
        }
    });

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    notify_rx.next().await.expect("Alice processed");

    tx.unbounded_send(Sequenced::new(person_bob()))?;
    notify_rx.next().await.expect("Bob processed");

    tx.unbounded_send(Sequenced::new(person_charlie()))?;
    notify_rx.next().await.expect("Charlie processed");

    // Cancel further processing
    cancellation_token.cancel();

    tx.unbounded_send(Sequenced::new(person_dave()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Assert
    let processed = {
        let g = collected_items.lock().await;
        g.clone()
    };
    assert_eq!(processed.len(), 3,);
    assert_eq!(processed[0], person_alice());
    assert_eq!(processed[1], person_bob());
    assert_eq!(processed[2], person_charlie());
    assert!(!processed.contains(&person_dave()));
    assert!(!processed.contains(&animal_dog()));

    Ok(())
}

#[tokio::test]
async fn test_subscribe_latest_no_skipping_with_cancellation_and_errors() -> anyhow::Result<()> {
    // Arrange
    let collected_items = Arc::new(FutureMutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();
    let (notify_tx, mut notify_rx) = unbounded();

    let (tx, rx) = unbounded::<Sequenced<TestData>>();
    let stream = rx;
    let stream = stream.map(|timestamped| timestamped.value);

    let func = {
        let collected_items = collected_items_clone.clone();
        let notify_tx = notify_tx.clone();
        move |item: TestData, _| {
            let collected_items = collected_items.clone();
            let notify_tx = notify_tx.clone();
            async move {
                // Error on animals
                if matches!(&item, TestData::Animal(_)) {
                    let _ = notify_tx.unbounded_send(());
                    return Err(TestError::new("Animal processing error"));
                }

                collected_items.lock().await.push(item);
                let _ = notify_tx.unbounded_send(());

                Ok::<(), TestError>(())
            }
        }
    };

    let error_callback = |err: TestError| {
        eprintln!("Error occurred: {err:?}");
    };

    let cancellation_token = CancellationToken::new();
    spawn({
        let cancellation_token = cancellation_token.clone();

        async move {
            stream
                .subscribe_latest(func, error_callback, Some(cancellation_token))
                .await
                .expect("subscribe_latest should succeed");
        }
    });

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    notify_rx.next().await.expect("Alice processed");

    tx.unbounded_send(Sequenced::new(animal_dog()))?; // Error
    notify_rx.next().await.expect("Dog handled (error)");

    tx.unbounded_send(Sequenced::new(person_bob()))?;
    notify_rx.next().await.expect("Bob processed");

    cancellation_token.cancel();

    tx.unbounded_send(Sequenced::new(person_diane()))?;
    tx.unbounded_send(Sequenced::new(animal_cat()))?;

    // Assert
    let processed = {
        let g = collected_items.lock().await;
        g.clone()
    };
    assert_eq!(processed.len(), 2);
    assert!(processed.contains(&person_alice()),);
    assert!(processed.contains(&person_bob()));
    assert!(!processed.contains(&person_diane()));
    assert!(!processed.contains(&animal_cat()));
    assert!(!processed.contains(&animal_dog()));

    Ok(())
}

#[tokio::test]
async fn test_subscribe_latest_no_skipping_no_error_no_cancellation_no_concurrent_processing(
) -> anyhow::Result<()> {
    // Arrange
    let active_count = Arc::new(AtomicUsize::new(0));
    let max_concurrent = Arc::new(AtomicUsize::new(0));
    let processed_count = Arc::new(AtomicUsize::new(0));
    let (notify_tx, mut notify_rx) = unbounded();

    // Signal when processing starts; control completion via a finish gate
    let (started_tx, mut started_rx) = unbounded::<()>();
    let (finish_tx, finish_rx) = unbounded::<()>();
    let finish_rx_shared = Arc::new(FutureMutex::new(finish_rx));

    let (tx, rx) = unbounded::<Sequenced<TestData>>();
    let stream = rx;
    let stream = stream.map(|timestamped| timestamped.value);

    let func = {
        let active_count = active_count.clone();
        let max_concurrent = max_concurrent.clone();
        let processed_count = processed_count.clone();
        let notify_tx = notify_tx.clone();
        let started_tx = started_tx.clone();
        let finish_rx_shared = finish_rx_shared.clone();
        move |_item, _| {
            let active_count = active_count.clone();
            let max_concurrent = max_concurrent.clone();
            let processed_count = processed_count.clone();
            let notify_tx = notify_tx.clone();
            let started_tx = started_tx.clone();
            let finish_rx_shared = finish_rx_shared.clone();
            async move {
                // Increment active count and record max concurrency
                let current = active_count.fetch_add(1, Ordering::SeqCst) + 1;
                max_concurrent.fetch_max(current, Ordering::SeqCst);

                // Notify that processing has started
                let _ = started_tx.unbounded_send(());

                // Wait until test signals completion for this item
                let _ = finish_rx_shared.lock().await.next().await;

                // Decrement active count and mark processed
                active_count.fetch_sub(1, Ordering::SeqCst);
                processed_count.fetch_add(1, Ordering::SeqCst);
                let _ = notify_tx.unbounded_send(());

                Ok::<(), TestError>(())
            }
        }
    };

    let error_callback = |_err: TestError| {
        eprintln!("Unexpected error");
    };

    spawn({
        async move {
            stream
                .subscribe_latest(func, error_callback, None)
                .await
                .expect("subscribe_latest should succeed");
        }
    });

    // Act - Drive N sequential processings while always having the next item queued
    let n = 10;

    tx.unbounded_send(Sequenced::new(person_alice()))?;

    for i in 0..n {
        // Wait until current processing has started
        started_rx.next().await.expect("processing started");

        // Queue next item before finishing current to try to induce overlap
        if i + 1 < n {
            tx.unbounded_send(Sequenced::new(person_alice()))?;
        }

        // Now allow current processing to complete and wait for completion notification
        let _ = finish_tx.unbounded_send(());
        notify_rx.next().await.expect("processing completed");
    }

    // Assert
    let max = max_concurrent.load(Ordering::SeqCst);
    assert_eq!(max, 1);

    let final_active = active_count.load(Ordering::SeqCst);
    assert_eq!(final_active, 0);

    let done = processed_count.load(Ordering::SeqCst);
    assert_eq!(done, n);

    Ok(())
}

#[tokio::test]
async fn test_subscribe_latest_no_skipping_no_error_no_cancellation_token_empty_stream(
) -> anyhow::Result<()> {
    // Arrange
    let collected_items = Arc::new(FutureMutex::new(Vec::<TestData>::new()));
    let collected_items_clone = collected_items.clone();

    let (tx, rx) = unbounded::<Sequenced<TestData>>();
    let stream = rx;
    let stream = stream.map(|timestamped| timestamped.value);

    let func = move |item, _| {
        let collected_items = collected_items_clone.clone();
        async move {
            collected_items.lock().await.push(item);
            Ok::<(), TestError>(())
        }
    };

    let error_callback = |_err: TestError| {
        eprintln!("Unexpected error");
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_latest(func, error_callback, None)
                .await
                .expect("subscribe_latest should succeed");
        }
    });

    // Act - Close stream without sending any items
    drop(tx);
    task_handle.await.unwrap();

    // Assert
    assert_eq!(*collected_items.lock().await, Vec::<TestData>::new());

    Ok(())
}

#[tokio::test]
async fn test_subscribe_latest_high_volume() -> anyhow::Result<()> {
    // Arrange
    let collected_items = Arc::new(FutureMutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();
    let (notify_tx, mut notify_rx) = unbounded();

    // Gate to block first processing until we're ready
    let (gate_tx, gate_rx) = unbounded::<()>();
    let gate_rx_shared = Arc::new(FutureMutex::new(Some(gate_rx)));

    // Signal when first processing starts
    let (start_tx, mut start_rx) = unbounded::<()>();
    let start_tx_shared = Arc::new(FutureMutex::new(Some(start_tx)));

    // Track how many items have been enqueued by the for_each loop
    let enqueue_count = Arc::new(AtomicUsize::new(0));
    let enqueue_count_clone = enqueue_count.clone();

    let (tx, rx) = unbounded::<Sequenced<TestData>>();
    let stream = rx;
    // Increment counter as each item passes through the stream
    let stream = stream.map(move |timestamped| {
        enqueue_count_clone.fetch_add(1, Ordering::SeqCst);
        timestamped.value
    });

    let func = {
        let collected_items = collected_items_clone.clone();
        let notify_tx = notify_tx.clone();
        let gate_rx_shared = gate_rx_shared.clone();
        let start_tx_shared = start_tx_shared.clone();
        move |item, _| {
            let collected_items = collected_items.clone();
            let notify_tx = notify_tx.clone();
            let gate_rx_shared = gate_rx_shared.clone();
            let start_tx_shared = start_tx_shared.clone();
            async move {
                // Signal first processing start once
                if let Some(tx) = start_tx_shared.lock().await.take() {
                    let _ = tx.unbounded_send(());
                }

                // Only first processing waits for the gate
                if let Some(mut rx) = gate_rx_shared.lock().await.take() {
                    let _ = rx.next().await;
                }

                collected_items.lock().await.push(item);
                let _ = notify_tx.unbounded_send(());
                Ok::<(), TestError>(())
            }
        }
    };

    let error_callback = |_err: TestError| {
        eprintln!("Unexpected error");
    };

    spawn({
        async move {
            stream
                .subscribe_latest(func, error_callback, None)
                .await
                .expect("subscribe_latest should succeed");
        }
    });

    // Act - Block first, flood many, wait for all to be enqueued, then release gate
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    start_rx.next().await.expect("first processing started");

    // Flood with many identical items, then a distinct last item (sentinel)
    let total_flood = 500;
    for _ in 0..total_flood {
        tx.unbounded_send(Sequenced::new(person_alice()))?;
    }
    tx.unbounded_send(Sequenced::new(person_bob()))?; // sentinel latest

    // Wait until all items (1 Alice + 500 Alices + 1 Bob = 502) have passed through
    // the stream's map() and been processed by for_each's enqueue logic
    let expected_enqueue_count = 1 + total_flood + 1;
    while enqueue_count.load(Ordering::SeqCst) < expected_enqueue_count {
        yield_now().await;
    }

    // Now release the first processing - Bob is guaranteed to be the latest in state
    let _ = gate_tx.unbounded_send(());

    // Expect exactly two completions (Alice, then Bob)
    notify_rx
        .next()
        .await
        .expect("first item completed (Alice)");
    notify_rx.next().await.expect("second item completed (Bob)");

    // Assert - high volume collapses to exactly 2 processed items
    let processed = collected_items.lock().await;
    assert_eq!(processed.len(), 2);
    assert_eq!(processed[0], person_alice());
    assert_eq!(processed[1], person_bob());
    drop(processed);

    Ok(())
}

#[tokio::test]
async fn test_subscribe_latest_single_item() -> anyhow::Result<()> {
    // Arrange
    let collected_items = Arc::new(FutureMutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();
    let (notify_tx, mut notify_rx) = unbounded();

    let (tx, rx) = unbounded::<Sequenced<TestData>>();
    let stream = rx;
    let stream = stream.map(|timestamped| timestamped.value);

    let func = {
        let collected_items = collected_items_clone.clone();
        let notify_tx = notify_tx.clone();
        move |item, _| {
            let collected_items = collected_items.clone();
            let notify_tx = notify_tx.clone();
            async move {
                sleep(Duration::from_millis(50)).await;
                collected_items.lock().await.push(item);
                let _ = notify_tx.unbounded_send(());
                Ok::<(), TestError>(())
            }
        }
    };

    let error_callback = |_err: TestError| {
        eprintln!("Unexpected error");
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_latest(func, error_callback, None)
                .await
                .expect("subscribe_latest should succeed");
        }
    });

    // Act - Send only one item
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Wait for item to complete
    notify_rx.next().await.unwrap();

    // Assert
    let processed = {
        let g = collected_items.lock().await;
        g.clone()
    };
    assert_eq!(processed.len(), 1, "Exactly 1 item should be processed");
    assert_eq!(processed[0], person_alice(), "Should be Alice");

    // Cleanup
    drop(tx);
    task_handle.await.unwrap();

    Ok(())
}

#[tokio::test]
async fn test_subscribe_latest_error_aggregation_with_callback() -> anyhow::Result<()> {
    use std::sync::Mutex as StdMutex;

    // Arrange
    let (tx, rx) = unbounded::<Sequenced<TestData>>();
    let stream = rx;
    let stream = stream.map(|timestamped| timestamped.value);
    let (notify_tx, mut notify_rx) = unbounded();
    let errors = Arc::new(StdMutex::new(Vec::new()));
    let errors_clone = errors.clone();

    let func = {
        let notify_tx = notify_tx.clone();
        move |item: TestData, _ctx: CancellationToken| {
            let notify_tx = notify_tx.clone();
            async move {
                let _ = notify_tx.unbounded_send(());
                if matches!(&item, TestData::Animal(_)) {
                    Err(TestError::new(format!("Animals not allowed: {:?}", item)))
                } else {
                    Ok(())
                }
            }
        }
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_latest(
                    func,
                    move |err| {
                        errors_clone.lock().unwrap().push(err.to_string());
                    },
                    None,
                )
                .await
        }
    });

    // Act - Send mix of valid and invalid items
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    notify_rx.next().await.unwrap();

    tx.unbounded_send(Sequenced::new(animal_dog()))?; // Error
    notify_rx.next().await.unwrap();

    tx.unbounded_send(Sequenced::new(person_bob()))?;
    notify_rx.next().await.unwrap();

    tx.unbounded_send(Sequenced::new(animal_cat()))?; // Error
    notify_rx.next().await.unwrap();

    drop(tx);

    // Assert - Should complete successfully, errors handled by callback
    let result = task_handle.await.unwrap();
    assert!(result.is_ok(), "Expected success with error callback");

    let collected_errors = errors.lock().unwrap();
    assert_eq!(collected_errors.len(), 2, "Expected 2 errors collected");

    Ok(())
}
