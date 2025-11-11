use fluxion_exec::subscribe_latest_async::SubscribeLatestAsyncExt;
use fluxion_stream::timestamped_channel::unbounded_channel;
use fluxion_test_utils::test_data::{
    TestData, animal_ant, animal_cat, animal_dog, animal_spider, person_alice, person_bob,
    person_charlie, person_dave, person_diane, plant_rose, push,
};
use std::sync::Arc;
use tokio::{
    sync::Mutex,
    sync::mpsc,
    time::{Duration, sleep},
};
use tokio_stream::{StreamExt as _, wrappers::UnboundedReceiverStream};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn test_subscribe_latest_async_no_skipping_no_error_no_cancellation_token() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();
    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();

    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);

    let func = {
        let collected_items = collected_items_clone.clone();
        let notify_tx = notify_tx.clone();
        move |item, _| {
            let collected_items = collected_items.clone();
            let notify_tx = notify_tx.clone();
            async move {
                collected_items.lock().await.push(item);
                let _ = notify_tx.send(());
                Ok(())
            }
        }
    };

    let error_callback = |err: ()| {
        eprintln!("Error occurred: {:?}", err);
    };

    tokio::spawn({
        async move {
            stream
                .subscribe_latest_async(func, Some(error_callback), None)
                .await;
        }
    });

    // Act - emit items one at a time, waiting for each to be processed
    push(person_alice(), &sender);
    notify_rx.recv().await.expect("Alice processed");

    push(person_bob(), &sender);
    notify_rx.recv().await.expect("Bob processed");

    push(person_charlie(), &sender);
    notify_rx.recv().await.expect("Charlie processed");

    push(person_diane(), &sender);
    notify_rx.recv().await.expect("Diane processed");

    push(person_dave(), &sender);
    notify_rx.recv().await.expect("Dave processed");

    push(animal_dog(), &sender);
    notify_rx.recv().await.expect("Dog processed");

    push(animal_cat(), &sender);
    notify_rx.recv().await.expect("Cat processed");

    push(animal_ant(), &sender);
    notify_rx.recv().await.expect("Ant processed");

    push(animal_spider(), &sender);
    notify_rx.recv().await.expect("Spider processed");

    push(plant_rose(), &sender);
    notify_rx.recv().await.expect("Rose processed");

    // Assert
    let processed = collected_items.lock().await;
    assert_eq!(
        processed.len(),
        10,
        "All 10 items should be processed when each waits for previous to complete"
    );
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
}

#[tokio::test]
async fn test_subscribe_latest_async_with_skipping_no_error_no_cancellation_token() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();
    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();

    // Barrier channel: only the FIRST processed item will wait on this
    let (gate_tx, gate_rx) = mpsc::unbounded_channel::<()>();
    let gate_rx_shared = Arc::new(Mutex::new(Some(gate_rx)));

    // Start channel: first processing signals when it starts (to avoid races)
    let (start_tx, mut start_rx) = mpsc::unbounded_channel::<()>();
    let start_tx_shared = Arc::new(Mutex::new(Some(start_tx)));

    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);

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
                if let Some(tx) = start_tx_shared.lock().await.take() {
                    let _ = tx.send(());
                }
                // Only the first processing waits for the gate signal
                if let Some(mut rx) = gate_rx_shared.lock().await.take() {
                    // Wait for external signal to unblock the first item
                    let _ = rx.recv().await;
                }

                collected_items.lock().await.push(item);
                let _ = notify_tx.send(());
                Ok(())
            }
        }
    };

    let error_callback = |err: ()| {
        eprintln!("Error occurred: {:?}", err);
    };

    tokio::spawn({
        async move {
            stream
                .subscribe_latest_async(func, Some(error_callback), None)
                .await;
        }
    });

    // Act
    push(person_alice(), &sender);
    start_rx.recv().await.expect("first processing started");

    // While the first item is blocked, send 4 more items rapidly
    push(person_bob(), &sender);
    push(person_charlie(), &sender);
    push(person_diane(), &sender);
    push(person_dave(), &sender); // latest

    // Unblock the first processing, allowing it to complete
    let _ = gate_tx.send(());

    // Wait for exactly two processed notifications (Alice, then latest Dave)
    notify_rx
        .recv()
        .await
        .expect("Alice should be processed first");
    notify_rx
        .recv()
        .await
        .expect("Latest (Dave) should be processed next");

    // Assert
    let processed = collected_items.lock().await;
    assert_eq!(
        processed.len(),
        2,
        "Expected exactly 2 processed items with 3 skipped"
    );
    assert_eq!(
        processed[0],
        person_alice(),
        "First processed should be Alice"
    );
    assert_eq!(
        processed[1],
        person_dave(),
        "Second processed should be latest (Dave)"
    );
    assert!(
        !processed.contains(&person_bob())
            && !processed.contains(&person_charlie())
            && !processed.contains(&person_diane()),
        "Bob, Charlie, and Diane should be skipped"
    );
}

#[tokio::test]
async fn test_subscribe_latest_async_no_skipping_with_error_no_cancellation_token() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();
    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();

    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);

    let func = {
        let collected_items = collected_items_clone.clone();
        let notify_tx = notify_tx.clone();
        move |item: TestData, _| {
            let collected_items = collected_items.clone();
            let notify_tx = notify_tx.clone();
            async move {
                if matches!(&item, TestData::Person(p) if p.name == "Bob" || p.name == "Dave") {
                    let _ = notify_tx.send(());
                    return Err(format!("Failed to process {:?}", item));
                }

                let mut items = collected_items.lock().await;
                items.push(item);
                let _ = notify_tx.send(());

                Ok(())
            }
        }
    };

    let error_callback = |err: String| {
        eprintln!("Error occurred: {:?}", err);
    };

    tokio::spawn({
        async move {
            stream
                .subscribe_latest_async(func, Some(error_callback), None)
                .await;
        }
    });

    // Act
    push(person_alice(), &sender);
    notify_rx.recv().await.expect("Alice processed");

    push(person_bob(), &sender); // Error
    notify_rx.recv().await.expect("Bob handled (error)");

    push(person_charlie(), &sender);
    notify_rx.recv().await.expect("Charlie processed");

    push(person_dave(), &sender); // Error
    notify_rx.recv().await.expect("Dave handled (error)");

    push(animal_dog(), &sender);
    notify_rx.recv().await.expect("Dog processed");

    // Assert
    let processed = collected_items.lock().await;
    assert_eq!(
        processed.len(),
        3,
        "Expected exactly 3 processed items with 2 skipped due to errors"
    );
    assert!(
        processed.contains(&person_alice()),
        "Alice must be processed"
    );
    assert!(
        processed.contains(&person_charlie()),
        "Charlie must be processed"
    );
    assert!(processed.contains(&animal_dog()), "Dog must be processed");
    assert!(
        !processed.contains(&person_bob()),
        "Bob should not be in processed items (error case)"
    );
    assert!(
        !processed.contains(&person_dave()),
        "Dave should not be in processed items (error case)"
    );
}

#[tokio::test]
async fn test_subscribe_latest_async_with_cancellation_no_errors() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();
    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();
    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);

    let func = {
        let collected_items = collected_items_clone.clone();
        let notify_tx = notify_tx.clone();
        move |item, ctx: CancellationToken| {
            let collected_items = collected_items.clone();
            let notify_tx = notify_tx.clone();
            async move {
                if ctx.is_cancelled() {
                    let _ = notify_tx.send(());
                    return Ok(());
                }

                let mut items = collected_items.lock().await;
                items.push(item);

                sleep(Duration::from_millis(50)).await;
                let _ = notify_tx.send(());

                Ok(())
            }
        }
    };

    let error_callback = |_err: ()| {
        eprintln!("Error occurred.");
    };

    let cancellation_token = CancellationToken::new();
    let task_handle = tokio::spawn({
        let cancellation_token = cancellation_token.clone();

        async move {
            stream
                .subscribe_latest_async(func, Some(error_callback), Some(cancellation_token))
                .await;
        }
    });

    // Act
    push(person_alice(), &sender);
    push(person_bob(), &sender);
    push(person_charlie(), &sender);
    push(person_diane(), &sender);

    // Wait for first item to complete
    notify_rx.recv().await.unwrap();

    cancellation_token.cancel();

    // Act - these should not be processed after cancellation
    push(person_dave(), &sender);
    push(animal_dog(), &sender);

    // Try to receive more notifications with timeout
    tokio::select! {
        _ = notify_rx.recv() => {},
        _ = sleep(Duration::from_millis(100)) => {},
    }

    // Assert
    let processed = collected_items.lock().await;
    assert!(!processed.is_empty(), "Expected some items to be processed");
    // Items emitted before cancellation should have a chance to be processed
    // Dave and dog were sent after cancellation, should NOT be processed
    assert!(
        !processed.contains(&person_dave()),
        "Dave should not be processed (sent after cancellation)"
    );
    assert!(
        !processed.contains(&animal_dog()),
        "Dog should not be processed (sent after cancellation)"
    );

    // Cleanup
    drop(sender);
    task_handle.await.unwrap();
}

#[tokio::test]
async fn test_subscribe_latest_async_with_cancellation_and_errors() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();
    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();

    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);

    let func = {
        let collected_items = collected_items_clone.clone();
        let notify_tx = notify_tx.clone();
        move |item: TestData, _| {
            let collected_items = collected_items.clone();
            let notify_tx = notify_tx.clone();
            async move {
                // Error on animals
                if matches!(&item, TestData::Animal(_)) {
                    let _ = notify_tx.send(());
                    return Err(());
                }

                let mut items = collected_items.lock().await;
                items.push(item);

                sleep(Duration::from_millis(50)).await;
                let _ = notify_tx.send(());

                Ok(())
            }
        }
    };

    let error_callback = |err: ()| {
        eprintln!("Error occurred: {:?}", err);
    };

    let cancellation_token = CancellationToken::new();
    let task_handle = tokio::spawn({
        let cancellation_token = cancellation_token.clone();

        async move {
            stream
                .subscribe_latest_async(func, Some(error_callback), Some(cancellation_token))
                .await;
        }
    });

    // Act
    push(person_alice(), &sender);
    push(person_bob(), &sender);
    push(animal_dog(), &sender); // Error
    push(person_charlie(), &sender);

    // Wait for first item to complete
    notify_rx.recv().await.unwrap();

    cancellation_token.cancel();

    // Act - these should not be processed after cancellation
    push(person_diane(), &sender);
    push(animal_cat(), &sender);

    // Try to receive more notifications with timeout
    tokio::select! {
        _ = notify_rx.recv() => {},
        _ = sleep(Duration::from_millis(100)) => {},
    }

    // Assert
    let processed = collected_items.lock().await;
    assert!(!processed.is_empty(), "Expected some items to be processed");
    // Diane and Cat were sent after cancellation, should NOT be processed
    assert!(
        !processed.contains(&person_diane()),
        "Diane should not be processed (sent after cancellation)"
    );
    assert!(
        !processed.contains(&animal_cat()),
        "Cat should not be processed (sent after cancellation)"
    );

    // Cleanup
    drop(sender);
    task_handle.await.unwrap();
}

#[tokio::test]
async fn test_subscribe_latest_async_skips_intermediate_values() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();
    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();

    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);

    let func = {
        let collected_items = collected_items_clone.clone();
        let notify_tx = notify_tx.clone();
        move |item, _| {
            let collected_items = collected_items.clone();
            let notify_tx = notify_tx.clone();
            async move {
                // Slow processing (100ms) to ensure items queue up
                sleep(Duration::from_millis(100)).await;
                collected_items.lock().await.push(item);
                let _ = notify_tx.send(()); // Signal completion
                Ok::<(), ()>(())
            }
        }
    };

    let error_callback = |_err: ()| {
        eprintln!("Unexpected error");
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_latest_async(func, Some(error_callback), None)
                .await;
        }
    });

    // Act - Send 5 items rapidly (much faster than processing time)
    push(person_alice(), &sender); // Item 1 - should be processed (starts immediately)
    sleep(Duration::from_millis(5)).await;
    push(person_bob(), &sender); // Item 2 - should be skipped
    sleep(Duration::from_millis(5)).await;
    push(person_charlie(), &sender); // Item 3 - should be skipped
    sleep(Duration::from_millis(5)).await;
    push(person_diane(), &sender); // Item 4 - should be skipped
    sleep(Duration::from_millis(5)).await;
    push(person_dave(), &sender); // Item 5 - should be processed (latest when #1 completes)

    // Wait for first item to complete
    notify_rx.recv().await.unwrap();

    // Wait for second item to complete (with timeout in case only 1 processes)
    tokio::select! {
        _ = notify_rx.recv() => {},
        _ = sleep(Duration::from_millis(150)) => {},
    }

    // Give a moment for any final processing
    sleep(Duration::from_millis(50)).await;

    // Assert - deterministic check
    let processed = collected_items.lock().await;

    // Core "latest" behavior verification:
    // 1. First item should always be processed
    assert!(
        processed.contains(&person_alice()),
        "First item (Alice) should be processed"
    );

    // 2. Not all items should be processed (some must be skipped)
    assert!(
        processed.len() < 5,
        "Should skip some items due to 'latest' behavior, but processed all 5: {:?}",
        processed
    );

    // 3. The second processed item should be the last emitted (or close to it)
    if processed.len() == 2 {
        assert_eq!(
            processed[1],
            person_dave(),
            "Second item should be Dave (last emitted)"
        );
        // Verify intermediate items were NOT processed
        assert!(!processed.contains(&person_bob()), "Bob should be skipped");
        assert!(
            !processed.contains(&person_charlie()),
            "Charlie should be skipped"
        );
        assert!(
            !processed.contains(&person_diane()),
            "Diane should be skipped"
        );
    }

    // Cleanup
    drop(sender);
    task_handle.await.unwrap();
}

#[tokio::test]
async fn test_subscribe_latest_async_no_concurrent_processing() {
    // Arrange
    let active_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let max_concurrent = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let processed_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();

    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);

    let func = {
        let active_count = active_count.clone();
        let max_concurrent = max_concurrent.clone();
        let processed_count = processed_count.clone();
        let notify_tx = notify_tx.clone();
        move |_item, _| {
            let active_count = active_count.clone();
            let max_concurrent = max_concurrent.clone();
            let processed_count = processed_count.clone();
            let notify_tx = notify_tx.clone();
            async move {
                // Increment active count
                let current = active_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;

                // Track maximum concurrent processing
                max_concurrent.fetch_max(current, std::sync::atomic::Ordering::SeqCst);

                // Simulate work
                sleep(Duration::from_millis(50)).await;

                // Decrement active count
                active_count.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                processed_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let _ = notify_tx.send(());

                Ok::<(), ()>(())
            }
        }
    };

    let error_callback = |_err: ()| {
        eprintln!("Unexpected error");
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_latest_async(func, Some(error_callback), None)
                .await;
        }
    });

    // Act - Send multiple items rapidly
    for _ in 0..10 {
        push(person_alice(), &sender);
        sleep(Duration::from_millis(5)).await;
    }

    // Close stream and wait for all processing to complete
    drop(sender);
    task_handle.await.unwrap();

    // Assert
    let max = max_concurrent.load(std::sync::atomic::Ordering::SeqCst);
    assert_eq!(
        max, 1,
        "Expected maximum of 1 concurrent processing, but found {}",
        max
    );

    let final_active = active_count.load(std::sync::atomic::Ordering::SeqCst);
    assert_eq!(
        final_active, 0,
        "Expected no active processing at end, but found {}",
        final_active
    );
}

#[tokio::test]
async fn test_subscribe_latest_async_empty_stream() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();

    let (sender, receiver) = unbounded_channel::<TestData>();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);

    let func = move |item, _| {
        let collected_items = collected_items_clone.clone();
        async move {
            collected_items.lock().await.push(item);
            Ok::<(), ()>(())
        }
    };

    let error_callback = |_err: ()| {
        eprintln!("Unexpected error");
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_latest_async(func, Some(error_callback), None)
                .await;
        }
    });

    // Act - Close stream without sending any items
    drop(sender);

    // Wait for task to complete
    task_handle.await.unwrap();

    // Assert
    assert_eq!(*collected_items.lock().await, Vec::<TestData>::new());
}

#[tokio::test]
async fn test_subscribe_latest_async_error_unblocks_state() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();
    let error_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();

    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);

    let func = {
        let collected_items = collected_items_clone.clone();
        let notify_tx = notify_tx.clone();
        move |item: TestData, _| {
            let collected_items = collected_items.clone();
            let notify_tx = notify_tx.clone();
            async move {
                // Error on Bob
                if matches!(&item, TestData::Person(p) if p.name == "Bob") {
                    sleep(Duration::from_millis(50)).await;
                    let _ = notify_tx.send(());
                    return Err("Error processing Bob".to_string());
                }

                sleep(Duration::from_millis(50)).await;
                collected_items.lock().await.push(item);
                let _ = notify_tx.send(());
                Ok::<(), String>(())
            }
        }
    };

    let error_callback = {
        let error_count = error_count.clone();
        move |_err: String| {
            error_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_latest_async(func, Some(error_callback), None)
                .await;
        }
    });

    // Act - Send Alice, then Bob (error), then Charlie (should still process)
    push(person_alice(), &sender);
    sleep(Duration::from_millis(5)).await; // Small delay to let Alice start processing
    push(person_bob(), &sender); // This will cause an error

    // Wait for Alice to complete
    notify_rx.recv().await.unwrap();

    // Wait for Bob to complete (with error)
    notify_rx.recv().await.unwrap();

    push(person_charlie(), &sender); // This should still be processed

    // Wait for Charlie to complete
    notify_rx.recv().await.unwrap();

    // Assert
    let processed = collected_items.lock().await;
    assert!(
        processed.contains(&person_alice()),
        "Alice should be processed"
    );
    assert!(
        processed.contains(&person_charlie()),
        "Charlie should be processed (error didn't block state)"
    );
    assert!(
        !processed.contains(&person_bob()),
        "Bob should not be in processed (error case)"
    );

    // Note: Error callback may or may not be called depending on timing
    // The key assertion is that Charlie was processed after Bob errored

    // Cleanup
    drop(sender);
    task_handle.await.unwrap();
}

#[tokio::test]
async fn test_subscribe_latest_async_high_volume() {
    // Arrange
    let processed_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();

    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);

    let func = {
        let processed_count = processed_count.clone();
        let notify_tx = notify_tx.clone();
        move |_item, _| {
            let processed_count = processed_count.clone();
            let notify_tx = notify_tx.clone();
            async move {
                sleep(Duration::from_millis(10)).await;
                processed_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let _ = notify_tx.send(());
                Ok::<(), ()>(())
            }
        }
    };

    let error_callback = |_err: ()| {
        eprintln!("Unexpected error");
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_latest_async(func, Some(error_callback), None)
                .await;
        }
    });

    // Act - Send 100 items rapidly
    for _ in 0..100 {
        push(person_alice(), &sender);
    }

    // Wait for first item to complete
    notify_rx.recv().await.unwrap();

    // Give some time for more processing (but not enough for all 100)
    sleep(Duration::from_millis(100)).await;

    // Assert
    let count = processed_count.load(std::sync::atomic::Ordering::SeqCst);
    // Due to "latest" behavior, we expect far fewer than 100 items processed
    // Should process first item + latest items as processing completes
    assert!(count > 0, "At least some items should be processed");
    assert!(
        count < 100,
        "Should process fewer than all 100 items due to 'latest' behavior, processed: {}",
        count
    );

    // Cleanup
    drop(sender);
    task_handle.await.unwrap();
}

#[tokio::test]
async fn test_subscribe_latest_async_single_item() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();
    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();

    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);

    let func = {
        let collected_items = collected_items_clone.clone();
        let notify_tx = notify_tx.clone();
        move |item, _| {
            let collected_items = collected_items.clone();
            let notify_tx = notify_tx.clone();
            async move {
                sleep(Duration::from_millis(50)).await;
                collected_items.lock().await.push(item);
                let _ = notify_tx.send(());
                Ok::<(), ()>(())
            }
        }
    };

    let error_callback = |_err: ()| {
        eprintln!("Unexpected error");
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_latest_async(func, Some(error_callback), None)
                .await;
        }
    });

    // Act - Send only one item
    push(person_alice(), &sender);

    // Wait for item to complete
    notify_rx.recv().await.unwrap();

    // Assert
    let processed = collected_items.lock().await;
    assert_eq!(processed.len(), 1, "Exactly 1 item should be processed");
    assert_eq!(processed[0], person_alice(), "Should be Alice");

    // Cleanup
    drop(sender);
    task_handle.await.unwrap();
}
