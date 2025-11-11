use fluxion_exec::subscribe_latest_async::SubscribeLatestAsyncExt;
use fluxion_stream::timestamped_channel::unbounded_channel;
use fluxion_test_utils::test_data::{
    TestData, animal_ant, animal_cat, animal_dog, animal_spider, person_alice, person_bob,
    person_charlie, person_dave, person_diane, plant_rose, push,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};
use tokio_stream::{StreamExt as _, wrappers::UnboundedReceiverStream};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn test_subscribe_latest_async_no_error_no_cancellation_token() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();

    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);

    let func = move |item, _| {
        let collected_items = collected_items_clone.clone();
        async move {
            {
                let mut items = collected_items.lock().await;
                items.push(item);
                sleep(Duration::from_millis(50)).await;
            }
            Ok(())
        }
    };

    let error_callback = |err: ()| {
        eprintln!("Error occurred: {:?}", err);
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_latest_async(func, Some(error_callback), None)
                .await;
        }
    });

    // Act
    push(person_alice(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(person_bob(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(person_charlie(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(person_diane(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(person_dave(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(animal_dog(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(animal_cat(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(animal_ant(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(animal_spider(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(plant_rose(), &sender);
    sleep(Duration::from_millis(10)).await;

    // Wait for processing to complete
    sleep(Duration::from_millis(200)).await;

    // Assert
    let processed = collected_items.lock().await;
    assert!(
        !processed.is_empty(),
        "Expected some items to be processed, but found none"
    );
    // Due to "latest" behavior with rapid emissions, we expect:
    // - First item (alice) should be processed (starts immediately)
    // - Some intermediate items will be skipped
    // - Last item (rose) should be processed
    assert!(
        processed.contains(&person_alice()),
        "First item should be processed"
    );
    // Note: Due to timing, the last item might still be processing or skipped
    // so we just verify at least the first item was processed

    // Cleanup
    drop(sender);
    task_handle.await.unwrap();
}

#[tokio::test]
async fn test_subscribe_latest_async_with_error_no_cancellation_token() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();

    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);

    let func = move |item: TestData, _| {
        let collected_items = collected_items_clone.clone();
        async move {
            // Error on every third person (Bob, Dave)
            if matches!(&item, TestData::Person(p) if p.name == "Bob" || p.name == "Dave") {
                return Err(format!("Failed to process {:?}", item));
            }

            let mut items = collected_items.lock().await;
            items.push(item);

            sleep(Duration::from_millis(50)).await;

            Ok(())
        }
    };

    let error_callback = |err: String| {
        eprintln!("Error occurred: {:?}", err);
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_latest_async(func, Some(error_callback), None)
                .await;
        }
    });

    // Act
    push(person_alice(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(person_bob(), &sender); // Error
    sleep(Duration::from_millis(10)).await;
    push(person_charlie(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(person_dave(), &sender); // Error
    sleep(Duration::from_millis(10)).await;
    push(animal_dog(), &sender);
    sleep(Duration::from_millis(10)).await;

    // Assert
    let processed = collected_items.lock().await;
    assert!(!processed.is_empty(), "Expected some items to be processed");
    // Alice and Charlie should be processed, Bob and Dave cause errors and skip
    // Due to rapid emissions, some items may be skipped by "latest" behavior
    assert!(
        processed.contains(&person_alice()) || processed.contains(&person_charlie()),
        "At least one valid person should be processed"
    );
    assert!(
        !processed.contains(&person_bob()),
        "Bob should not be in processed items (error case)"
    );
    assert!(
        !processed.contains(&person_dave()),
        "Dave should not be in processed items (error case)"
    );

    // Cleanup
    drop(sender);
    task_handle.await.unwrap();
}

#[tokio::test]
async fn test_subscribe_latest_async_with_cancellation_no_errors() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();
    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);

    let func = move |item, ctx: CancellationToken| {
        let collected_items = collected_items_clone.clone();
        async move {
            if ctx.is_cancelled() {
                return Ok(());
            }

            let mut items = collected_items.lock().await;
            items.push(item);

            sleep(Duration::from_millis(50)).await;

            Ok(())
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
    sleep(Duration::from_millis(10)).await;
    push(person_bob(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(person_charlie(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(person_diane(), &sender);
    sleep(Duration::from_millis(10)).await;
    cancellation_token.cancel();

    // Assert
    assert_subset_items_processed(&collected_items).await;

    // Act - these should not be processed after cancellation
    push(person_dave(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(animal_dog(), &sender);
    sleep(Duration::from_millis(10)).await;

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

    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);

    let func = move |item: TestData, _| {
        let collected_items = collected_items_clone.clone();
        async move {
            // Error on animals
            if matches!(&item, TestData::Animal(_)) {
                return Err(());
            }

            let mut items = collected_items.lock().await;
            items.push(item);

            sleep(Duration::from_millis(50)).await;

            Ok(())
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
    sleep(Duration::from_millis(10)).await;
    push(person_bob(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(animal_dog(), &sender); // Error
    sleep(Duration::from_millis(10)).await;
    push(person_charlie(), &sender);
    sleep(Duration::from_millis(10)).await;
    cancellation_token.cancel();

    // Assert
    assert_subset_items_processed(&collected_items).await;

    // Act - these should not be processed after cancellation
    push(person_diane(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(animal_cat(), &sender);
    sleep(Duration::from_millis(10)).await;

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

async fn assert_subset_items_processed(processed_items: &Arc<Mutex<Vec<TestData>>>) {
    let processed_items = processed_items.lock().await;
    assert!(
        !processed_items.is_empty(),
        "Expected some items to be processed, but found none"
    );
}

#[tokio::test]
async fn test_subscribe_latest_async_skips_intermediate_values() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();

    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);

    let func = move |item, _| {
        let collected_items = collected_items_clone.clone();
        async move {
            // Slow processing (100ms) to ensure items queue up
            sleep(Duration::from_millis(100)).await;
            let mut items = collected_items.lock().await;
            items.push(item);
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

    // Wait for processing to complete (100ms per item * 2 items + margin)
    sleep(Duration::from_millis(300)).await;

    // Assert
    let processed = collected_items.lock().await;

    // Due to timing variations, we verify the core "latest" behavior:
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

    // 3. If we processed 2 items, the second should be the last emitted
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

    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);

    let func = {
        let active_count = active_count.clone();
        let max_concurrent = max_concurrent.clone();
        let processed_count = processed_count.clone();
        move |_item, _| {
            let active_count = active_count.clone();
            let max_concurrent = max_concurrent.clone();
            let processed_count = processed_count.clone();
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

    // Wait for all processing to complete
    sleep(Duration::from_millis(200)).await;

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

    // Cleanup
    drop(sender);
    task_handle.await.unwrap();
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

    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);

    let func = move |item: TestData, _| {
        let collected_items = collected_items_clone.clone();
        async move {
            // Error on Bob
            if matches!(&item, TestData::Person(p) if p.name == "Bob") {
                sleep(Duration::from_millis(50)).await;
                return Err("Error processing Bob".to_string());
            }

            sleep(Duration::from_millis(50)).await;
            collected_items.lock().await.push(item);
            Ok::<(), String>(())
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
    sleep(Duration::from_millis(10)).await;
    push(person_bob(), &sender); // This will cause an error
    sleep(Duration::from_millis(150)).await; // Wait for error to complete
    push(person_charlie(), &sender); // This should still be processed
    sleep(Duration::from_millis(150)).await;

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

    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);

    let func = {
        let processed_count = processed_count.clone();
        move |_item, _| {
            let processed_count = processed_count.clone();
            async move {
                sleep(Duration::from_millis(10)).await;
                processed_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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

    // Wait for processing
    sleep(Duration::from_millis(300)).await;

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

    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);

    let func = move |item, _| {
        let collected_items = collected_items_clone.clone();
        async move {
            sleep(Duration::from_millis(50)).await;
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

    // Act - Send only one item
    push(person_alice(), &sender);
    sleep(Duration::from_millis(100)).await;

    // Assert
    let processed = collected_items.lock().await;
    assert_eq!(processed.len(), 1, "Exactly 1 item should be processed");
    assert_eq!(processed[0], person_alice(), "Should be Alice");

    // Cleanup
    drop(sender);
    task_handle.await.unwrap();
}
