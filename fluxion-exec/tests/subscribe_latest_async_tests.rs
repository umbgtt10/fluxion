use fluxion_exec::subscribe_latest_async::SubscribeLatestAsyncExt;
use fluxion_stream::TestChannel;
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
use tokio_stream::StreamExt as _;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn test_subscribe_latest_async_no_skipping_no_error_no_cancellation() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();
    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();

    let channel = TestChannel::new();
    let stream = channel.stream.map(|timestamped| timestamped.value);

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
    push(person_alice(), &channel.sender);
    notify_rx.recv().await.expect("Alice processed");

    push(person_bob(), &channel.sender);
    notify_rx.recv().await.expect("Bob processed");

    push(person_charlie(), &channel.sender);
    notify_rx.recv().await.expect("Charlie processed");

    push(person_diane(), &channel.sender);
    notify_rx.recv().await.expect("Diane processed");

    push(person_dave(), &channel.sender);
    notify_rx.recv().await.expect("Dave processed");

    push(animal_dog(), &channel.sender);
    notify_rx.recv().await.expect("Dog processed");

    push(animal_cat(), &channel.sender);
    notify_rx.recv().await.expect("Cat processed");

    push(animal_ant(), &channel.sender);
    notify_rx.recv().await.expect("Ant processed");

    push(animal_spider(), &channel.sender);
    notify_rx.recv().await.expect("Spider processed");

    push(plant_rose(), &channel.sender);
    notify_rx.recv().await.expect("Rose processed");

    // Assert
    let processed = collected_items.lock().await;
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
}

#[tokio::test]
async fn test_subscribe_latest_async_with_skipping_no_error_no_cancellation() {
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

    let channel = TestChannel::new();
    let stream = channel.stream.map(|timestamped| timestamped.value);

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
    push(person_alice(), &channel.sender);
    start_rx.recv().await.expect("first processing started");

    // While the first item is blocked, send 4 more items rapidly
    push(person_bob(), &channel.sender);
    push(person_charlie(), &channel.sender);
    push(person_diane(), &channel.sender);
    push(person_dave(), &channel.sender); // latest

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
    assert_eq!(processed.len(), 2,);
    assert_eq!(processed[0], person_alice(),);
    assert_eq!(processed[1], person_dave(),);
    assert!(!processed.contains(&person_bob()));
    assert!(!processed.contains(&person_charlie()));
    assert!(!processed.contains(&person_diane()));
}

#[tokio::test]
async fn test_subscribe_latest_async_no_skipping_with_error_no_cancellation() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();
    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();

    let channel = TestChannel::new();
    let stream = channel.stream.map(|timestamped| timestamped.value);

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
    push(person_alice(), &channel.sender);
    notify_rx.recv().await.expect("Alice processed");

    push(person_bob(), &channel.sender); // Error
    notify_rx.recv().await.expect("Bob handled (error)");

    push(person_charlie(), &channel.sender);
    notify_rx.recv().await.expect("Charlie processed");

    push(person_dave(), &channel.sender); // Error
    notify_rx.recv().await.expect("Dave handled (error)");

    push(animal_dog(), &channel.sender);
    notify_rx.recv().await.expect("Dog processed");

    // Assert
    let processed = collected_items.lock().await;
    assert_eq!(processed.len(), 3);
    assert!(processed.contains(&person_alice()),);
    assert!(processed.contains(&person_charlie()),);
    assert!(processed.contains(&animal_dog()),);
    assert!(!processed.contains(&person_bob()),);
    assert!(!processed.contains(&person_dave()),);
}

#[tokio::test]
async fn test_subscribe_latest_async_no_skipping_no_errors_with_cancellation() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();
    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();
    let channel = TestChannel::new();
    let stream = channel.stream.map(|timestamped| timestamped.value);

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
                let _ = notify_tx.send(());

                Ok(())
            }
        }
    };

    let error_callback = |_err: ()| {
        eprintln!("Error occurred.");
    };

    let cancellation_token = CancellationToken::new();
    tokio::spawn({
        let cancellation_token = cancellation_token.clone();

        async move {
            stream
                .subscribe_latest_async(func, Some(error_callback), Some(cancellation_token))
                .await;
        }
    });

    // Act
    push(person_alice(), &channel.sender);
    notify_rx.recv().await.expect("Alice processed");

    push(person_bob(), &channel.sender);
    notify_rx.recv().await.expect("Bob processed");

    push(person_charlie(), &channel.sender);
    notify_rx.recv().await.expect("Charlie processed");

    // Cancel further processing
    cancellation_token.cancel();

    push(person_dave(), &channel.sender);
    push(animal_dog(), &channel.sender);

    // Assert
    let processed = collected_items.lock().await;
    assert_eq!(processed.len(), 3,);
    assert_eq!(processed[0], person_alice());
    assert_eq!(processed[1], person_bob());
    assert_eq!(processed[2], person_charlie());
    assert!(!processed.contains(&person_dave()));
    assert!(!processed.contains(&animal_dog()));
}

#[tokio::test]
async fn test_subscribe_latest_async_no_skipping_with_cancellation_and_errors() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();
    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();

    let channel = TestChannel::new();
    let stream = channel.stream.map(|timestamped| timestamped.value);

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
                let _ = notify_tx.send(());

                Ok(())
            }
        }
    };

    let error_callback = |err: ()| {
        eprintln!("Error occurred: {:?}", err);
    };

    let cancellation_token = CancellationToken::new();
    tokio::spawn({
        let cancellation_token = cancellation_token.clone();

        async move {
            stream
                .subscribe_latest_async(func, Some(error_callback), Some(cancellation_token))
                .await;
        }
    });

    // Act
    push(person_alice(), &channel.sender);
    notify_rx.recv().await.expect("Alice processed");

    push(animal_dog(), &channel.sender); // Error
    notify_rx.recv().await.expect("Dog handled (error)");

    push(person_bob(), &channel.sender);
    notify_rx.recv().await.expect("Bob processed");

    cancellation_token.cancel();

    push(person_diane(), &channel.sender);
    push(animal_cat(), &channel.sender);

    // Assert
    let processed = collected_items.lock().await;
    assert_eq!(processed.len(), 2);
    assert!(processed.contains(&person_alice()),);
    assert!(processed.contains(&person_bob()));
    assert!(!processed.contains(&person_diane()));
    assert!(!processed.contains(&animal_cat()));
    assert!(!processed.contains(&animal_dog()));
}

#[tokio::test]
async fn test_subscribe_latest_async_no_skipping_no_error_no_cancellation_no_concurrent_processing()
{
    // Arrange
    let active_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let max_concurrent = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let processed_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();

    // Signal when processing starts; control completion via a finish gate
    let (started_tx, mut started_rx) = mpsc::unbounded_channel::<()>();
    let (finish_tx, finish_rx) = mpsc::unbounded_channel::<()>();
    let finish_rx_shared = Arc::new(Mutex::new(finish_rx));

    let channel = TestChannel::new();
    let stream = channel.stream.map(|timestamped| timestamped.value);

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
                let current = active_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                max_concurrent.fetch_max(current, std::sync::atomic::Ordering::SeqCst);

                // Notify that processing has started
                let _ = started_tx.send(());

                // Wait until test signals completion for this item
                let _ = finish_rx_shared.lock().await.recv().await;

                // Decrement active count and mark processed
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

    tokio::spawn({
        async move {
            stream
                .subscribe_latest_async(func, Some(error_callback), None)
                .await;
        }
    });

    // Act - Drive N sequential processings while always having the next item queued
    let n = 10;

    push(person_alice(), &channel.sender);

    for i in 0..n {
        // Wait until current processing has started
        started_rx.recv().await.expect("processing started");

        // Queue next item before finishing current to try to induce overlap
        if i + 1 < n {
            push(person_alice(), &channel.sender);
        }

        // Now allow current processing to complete and wait for completion notification
        let _ = finish_tx.send(());
        notify_rx.recv().await.expect("processing completed");
    }

    // Assert
    let max = max_concurrent.load(std::sync::atomic::Ordering::SeqCst);
    assert_eq!(max, 1);

    let final_active = active_count.load(std::sync::atomic::Ordering::SeqCst);
    assert_eq!(final_active, 0);

    let done = processed_count.load(std::sync::atomic::Ordering::SeqCst);
    assert_eq!(done, n);
}

#[tokio::test]
async fn test_subscribe_latest_async_no_skipping_no_error_no_cancellation_token_empty_stream() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::<TestData>::new()));
    let collected_items_clone = collected_items.clone();

    let channel = TestChannel::new();
    let stream = channel.stream.map(|timestamped| timestamped.value);

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
    drop(channel.sender);
    task_handle.await.unwrap();

    // Assert
    assert_eq!(*collected_items.lock().await, Vec::<TestData>::new());
}

#[tokio::test]
async fn test_subscribe_latest_async_high_volume() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();
    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();

    // Barrier and start channels for deterministic high-load skipping
    let (gate_tx, gate_rx) = mpsc::unbounded_channel::<()>();
    let gate_rx_shared = Arc::new(Mutex::new(Some(gate_rx)));
    let (start_tx, mut start_rx) = mpsc::unbounded_channel::<()>();
    let start_tx_shared = Arc::new(Mutex::new(Some(start_tx)));
    // Flood completion channel to ensure bulk enqueue (including Bob) happens
    // before we allow the first processing to proceed
    let (flood_done_tx, flood_done_rx) = mpsc::unbounded_channel::<()>();
    let flood_done_rx_shared = Arc::new(Mutex::new(Some(flood_done_rx)));

    let channel = TestChannel::new();
    let stream = channel.stream.map(|timestamped| timestamped.value);

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
            let flood_done_rx_shared = flood_done_rx_shared.clone();
            async move {
                // Signal first processing start once
                if let Some(tx) = start_tx_shared.lock().await.take() {
                    let _ = tx.send(());
                }
                // For the first processing, wait until the flood is declared done,
                // then wait for the external gate release
                if let Some(mut rx) = flood_done_rx_shared.lock().await.take() {
                    let _ = rx.recv().await;
                }
                // Only first processing waits for the gate
                if let Some(mut rx) = gate_rx_shared.lock().await.take() {
                    let _ = rx.recv().await;
                }

                collected_items.lock().await.push(item);
                let _ = notify_tx.send(());
                Ok::<(), ()>(())
            }
        }
    };

    let error_callback = |_err: ()| {
        eprintln!("Unexpected error");
    };

    tokio::spawn({
        async move {
            stream
                .subscribe_latest_async(func, Some(error_callback), None)
                .await;
        }
    });

    // Act - Block first, flood many, ensure flood is done, then release gate
    push(person_alice(), &channel.sender);
    start_rx.recv().await.expect("first processing started");

    // Flood with many identical items, then a distinct last item
    for _ in 0..500 {
        push(person_alice(), &channel.sender);
    }
    push(person_bob(), &channel.sender); // sentinel latest

    // Signal that flooding (including Bob) is complete
    let _ = flood_done_tx.send(());

    // Now release the first processing
    let _ = gate_tx.send(());

    // Expect exactly two completions (Alice, then Bob)
    notify_rx
        .recv()
        .await
        .expect("first item completed (Alice)");
    notify_rx.recv().await.expect("second item completed (Bob)");

    // Assert - high volume collapses to exactly 2 processed items
    let processed = collected_items.lock().await;
    assert_eq!(processed.len(), 2,);
    assert_eq!(processed[0], person_alice());
}

#[tokio::test]
async fn test_subscribe_latest_async_single_item() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();
    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();

    let channel = TestChannel::new();
    let stream = channel.stream.map(|timestamped| timestamped.value);

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
    push(person_alice(), &channel.sender);

    // Wait for item to complete
    notify_rx.recv().await.unwrap();

    // Assert
    let processed = collected_items.lock().await;
    assert_eq!(processed.len(), 1, "Exactly 1 item should be processed");
    assert_eq!(processed[0], person_alice(), "Should be Alice");

    // Cleanup
    drop(channel.sender);
    task_handle.await.unwrap();
}
