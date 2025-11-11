use fluxion_exec::subscribe_async::SubscribeAsyncExt;
use fluxion_stream::timestamped_channel::unbounded_channel;
use fluxion_test_utils::test_data::{
    TestData, animal_cat, animal_dog, person_alice, person_bob, person_charlie, person_dave,
    person_diane, push,
};
use std::{
    sync::Arc,
    sync::Mutex as StdMutex,
    time::{Duration, Instant},
};
use tokio::{
    sync::Mutex as TokioMutex,
    sync::mpsc,
    time::sleep,
};
use tokio_stream::{StreamExt as _, wrappers::UnboundedReceiverStream};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn test_subscribe_async_sequential_processing() {
    // Arrange
    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);
    let results = Arc::new(TokioMutex::new(Vec::new()));
    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();

    let func = {
        let results = results.clone();
        let notify_tx = notify_tx.clone();
        move |item, _ctx: CancellationToken| {
            let results = results.clone();
            let notify_tx = notify_tx.clone();
            async move {
                results.lock().await.push(item);
                let _ = notify_tx.send(()); // Signal completion
                Ok::<(), ()>(())
            }
        }
    };

    let error_callback = {
        move |err| {
            panic!("Unexpected error while processing: {:?}", err);
        }
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_async(func, None, Some(error_callback))
                .await
        }
    });

    // Act & Assert - wait for actual processing completion
    push(person_alice(), &sender);
    notify_rx.recv().await.unwrap();
    assert_eq!(*results.lock().await, vec![person_alice()]);

    push(person_bob(), &sender);
    notify_rx.recv().await.unwrap();
    assert_eq!(*results.lock().await, vec![person_alice(), person_bob()]);

    push(person_charlie(), &sender);
    notify_rx.recv().await.unwrap();
    assert_eq!(
        *results.lock().await,
        vec![person_alice(), person_bob(), person_charlie()]
    );

    push(person_diane(), &sender);
    notify_rx.recv().await.unwrap();
    assert_eq!(
        *results.lock().await,
        vec![
            person_alice(),
            person_bob(),
            person_charlie(),
            person_diane()
        ]
    );

    push(person_dave(), &sender);
    notify_rx.recv().await.unwrap();
    assert_eq!(
        *results.lock().await,
        vec![
            person_alice(),
            person_bob(),
            person_charlie(),
            person_diane(),
            person_dave()
        ]
    );

    // Cleanup
    drop(sender);
    task_handle.await.unwrap();
}

#[tokio::test]
async fn test_subscribe_async_with_errors() {
    // Arrange
    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);
    let results = Arc::new(TokioMutex::new(Vec::new()));
    let errors = Arc::new(StdMutex::new(Vec::new()));
    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();

    let func = {
        let results = results.clone();
        let notify_tx = notify_tx.clone();
        move |item: TestData, _ctx: CancellationToken| {
            let results = results.clone();
            let notify_tx = notify_tx.clone();
            async move {
                // Error on every animal
                if matches!(&item, TestData::Animal(_)) {
                    let _ = notify_tx.send(()); // Signal completion (error case)
                    return Err(format!("Error processing animal: {:?}", item));
                }
                results.lock().await.push(item);
                let _ = notify_tx.send(()); // Signal completion
                Ok::<(), String>(())
            }
        }
    };

    let error_callback = {
        let errors = errors.clone();
        move |err| {
            errors.lock().unwrap().push(err);
        }
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_async(func, None, Some(error_callback))
                .await;
        }
    });

    // Act & Assert - wait for processing completion
    push(person_alice(), &sender);
    notify_rx.recv().await.unwrap();
    
    push(animal_dog(), &sender); // Error
    notify_rx.recv().await.unwrap();
    
    push(person_bob(), &sender);
    notify_rx.recv().await.unwrap();
    
    push(animal_cat(), &sender); // Error
    notify_rx.recv().await.unwrap();
    
    push(person_charlie(), &sender);
    notify_rx.recv().await.unwrap();

    // Assert final state
    assert_eq!(
        *results.lock().await,
        vec![person_alice(), person_bob(), person_charlie()]
    );
    assert_eq!(errors.lock().unwrap().len(), 2);

    // Cleanup
    drop(sender);
    task_handle.await.unwrap();
}

#[tokio::test]
async fn test_subscribe_async_triggered_cancellation_token() {
    // Arrange
    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);
    let results = Arc::new(TokioMutex::new(Vec::new()));
    let cancellation_token = CancellationToken::new();
    let cancellation_token_clone = cancellation_token.clone();
    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();

    let func = {
        let results = results.clone();
        let notify_tx = notify_tx.clone();
        move |item: TestData, ctx: CancellationToken| {
            let results = results.clone();
            let notify_tx = notify_tx.clone();
            async move {
                sleep(Duration::from_millis(20)).await;

                if ctx.is_cancelled() {
                    let _ = notify_tx.send(()); // Signal completion (cancelled)
                    return Ok(());
                }

                results.lock().await.push(item);
                let _ = notify_tx.send(()); // Signal completion

                Ok::<(), ()>(())
            }
        }
    };

    let error_callback = {
        move |err| {
            panic!("Unexpected error while processing: {:?}", err);
        }
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_async(func, Some(cancellation_token), Some(error_callback))
                .await;
        }
    });

    // Act & Assert
    push(person_alice(), &sender);
    notify_rx.recv().await.unwrap();
    push(person_bob(), &sender);
    notify_rx.recv().await.unwrap();

    assert_eq!(*results.lock().await, vec![person_alice(), person_bob()]);

    // Cancel and verify no more processing
    cancellation_token_clone.cancel();
    push(person_charlie(), &sender);
    push(person_diane(), &sender);
    
    // Give a moment for potential (incorrect) processing
    sleep(Duration::from_millis(50)).await;

    assert_eq!(*results.lock().await, vec![person_alice(), person_bob()]);

    drop(sender);
    task_handle.await.unwrap();
}

#[tokio::test]
async fn test_subscribe_async_errors_and_triggered_cancellation_token() {
    // Arrange
    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);
    let results = Arc::new(TokioMutex::new(Vec::new()));
    let errors = Arc::new(StdMutex::new(Vec::new()));
    let cancellation_token = CancellationToken::new();
    let cancellation_token_clone = cancellation_token.clone();
    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();

    #[derive(Debug, PartialEq, Eq)]
    enum ProcessingError {
        Cancelled(String),
        Other(String),
    }

    let func = {
        let results = results.clone();
        let notify_tx = notify_tx.clone();
        move |item: TestData, ctx: CancellationToken| {
            let results = results.clone();
            let notify_tx = notify_tx.clone();
            async move {
                sleep(Duration::from_millis(20)).await;

                if ctx.is_cancelled() {
                    let _ = notify_tx.send(());
                    return Err(ProcessingError::Cancelled(format!(
                        "Cancelled during processing of item: {:?}",
                        item
                    )));
                }

                results.lock().await.push(item.clone());
                let _ = notify_tx.send(());

                // Error on Charlie
                if matches!(&item, TestData::Person(p) if p.name == "Charlie") {
                    Err(ProcessingError::Other(
                        "Failed to process Charlie".to_string(),
                    ))
                } else {
                    Ok::<(), ProcessingError>(())
                }
            }
        }
    };

    let error_callback = {
        let errors = errors.clone();
        move |err| {
            if let ProcessingError::Other(_) = err {
                errors.lock().unwrap().push(err);
            }
        }
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_async(func, Some(cancellation_token), Some(error_callback))
                .await;
        }
    });

    // Act & Assert
    push(person_alice(), &sender);
    notify_rx.recv().await.unwrap();
    push(person_bob(), &sender);
    notify_rx.recv().await.unwrap();

    assert_eq!(*results.lock().await, vec![person_alice(), person_bob()]);
    assert!(errors.lock().unwrap().is_empty());

    // Send Charlie (which causes error)
    push(person_charlie(), &sender);
    notify_rx.recv().await.unwrap();

    assert_eq!(
        *results.lock().await,
        vec![person_alice(), person_bob(), person_charlie()]
    );
    assert_eq!(
        *errors.lock().unwrap(),
        vec![ProcessingError::Other(
            "Failed to process Charlie".to_string()
        )]
    );

    // Cancel and send more
    cancellation_token_clone.cancel();
    push(person_diane(), &sender);
    push(animal_dog(), &sender);
    
    // Give a moment for potential (incorrect) processing
    sleep(Duration::from_millis(50)).await;

    // No new items processed after cancellation
    assert_eq!(
        *results.lock().await,
        vec![person_alice(), person_bob(), person_charlie()]
    );
    assert_eq!(
        *errors.lock().unwrap(),
        vec![ProcessingError::Other(
            "Failed to process Charlie".to_string()
        )]
    );

    drop(sender);
    task_handle.await.unwrap();
}

#[tokio::test]
async fn test_subscribe_async_empty_stream() {
    // Arrange
    let (sender, receiver) = unbounded_channel::<TestData>();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);
    let results = Arc::new(TokioMutex::new(Vec::new()));

    let func = {
        let results = results.clone();
        move |item, _ctx: CancellationToken| {
            let results = results.clone();
            async move {
                results.lock().await.push(item);
                Ok::<(), ()>(())
            }
        }
    };

    let error_callback = {
        move |err| {
            panic!("Unexpected error while processing: {:?}", err);
        }
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_async(func, None, Some(error_callback))
                .await
        }
    });

    // Act - Close stream without sending any items
    drop(sender);

    // Wait for task to complete
    task_handle.await.unwrap();

    // Assert
    assert_eq!(*results.lock().await, Vec::<TestData>::new());
}

#[tokio::test]
async fn test_subscribe_async_parallel_processing() {
    // Arrange
    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);
    let results = Arc::new(TokioMutex::new(Vec::new()));
    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();

    let func = {
        let results = results.clone();
        let notify_tx = notify_tx.clone();
        move |item, _ctx: CancellationToken| {
            let results = results.clone();
            let notify_tx = notify_tx.clone();
            async move {
                // Each task sleeps for 100ms
                sleep(Duration::from_millis(100)).await;
                results.lock().await.push(item);
                let _ = notify_tx.send(());
                Ok::<(), ()>(())
            }
        }
    };

    let error_callback = {
        move |err| {
            panic!("Unexpected error while processing: {:?}", err);
        }
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_async(func, None, Some(error_callback))
                .await
        }
    });

    // Act - Send 3 items rapidly, measure completion time
    let start = Instant::now();
    push(person_alice(), &sender);
    push(person_bob(), &sender);
    push(person_charlie(), &sender);
    
    // Wait for all 3 to complete
    notify_rx.recv().await.unwrap();
    notify_rx.recv().await.unwrap();
    notify_rx.recv().await.unwrap();
    let duration = start.elapsed();

    // Assert
    let processed = results.lock().await;
    assert_eq!(processed.len(), 3, "All 3 items should be processed");
    
    // If processed sequentially, would take ~300ms (3 * 100ms)
    // If processed in parallel, should take ~100ms
    // Allow margin for overhead
    assert!(
        duration < Duration::from_millis(250),
        "Processing should be parallel (< 250ms), but took {:?}",
        duration
    );

    // Cleanup
    drop(sender);
    task_handle.await.unwrap();
}

#[tokio::test]
async fn test_subscribe_async_high_volume() {
    // Arrange
    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);
    let results = Arc::new(TokioMutex::new(Vec::new()));
    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();

    let func = {
        let results = results.clone();
        let notify_tx = notify_tx.clone();
        move |item, _ctx: CancellationToken| {
            let results = results.clone();
            let notify_tx = notify_tx.clone();
            async move {
                results.lock().await.push(item);
                let _ = notify_tx.send(());
                Ok::<(), ()>(())
            }
        }
    };

    let error_callback = {
        move |err| {
            panic!("Unexpected error while processing: {:?}", err);
        }
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_async(func, None, Some(error_callback))
                .await
        }
    });

    // Act - Send 100 items
    for _ in 0..100 {
        push(person_alice(), &sender);
    }

    // Wait for all 100 to complete
    for _ in 0..100 {
        notify_rx.recv().await.unwrap();
    }

    // Assert
    let processed = results.lock().await;
    assert_eq!(processed.len(), 100, "All 100 items should be processed");

    // Cleanup
    drop(sender);
    task_handle.await.unwrap();
}

#[tokio::test]
async fn test_subscribe_async_precancelled_token() {
    // Arrange
    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|timestamped| timestamped.value);
    let results = Arc::new(TokioMutex::new(Vec::new()));
    let cancellation_token = CancellationToken::new();

    // Pre-cancel the token
    cancellation_token.cancel();

    let func = {
        let results = results.clone();
        move |item, _ctx: CancellationToken| {
            let results = results.clone();
            async move {
                results.lock().await.push(item);
                Ok::<(), ()>(())
            }
        }
    };

    let error_callback = {
        move |err| {
            panic!("Unexpected error while processing: {:?}", err);
        }
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_async(func, Some(cancellation_token), Some(error_callback))
                .await
        }
    });

    // Act - Send items (should not be processed due to pre-cancelled token)
    push(person_alice(), &sender);
    push(person_bob(), &sender);
    sleep(Duration::from_millis(50)).await;

    // Assert
    assert_eq!(*results.lock().await, Vec::<TestData>::new());

    // Cleanup
    drop(sender);
    task_handle.await.unwrap();
}
