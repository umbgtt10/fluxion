// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_exec::subscribe_async::SubscribeAsyncExt;
use fluxion_test_utils::FluxionChannel;
use fluxion_test_utils::test_data::{
    TestData, animal_cat, animal_dog, person_alice, person_bob, person_charlie, person_dave,
    person_diane, push,
};
use std::{sync::Arc, sync::Mutex as StdMutex};
use tokio::{sync::Mutex as TokioMutex, sync::mpsc};
use tokio_stream::StreamExt as _;
use tokio_util::sync::CancellationToken;

#[derive(Debug, thiserror::Error)]
#[error("Test error: {0}")]
struct TestError(String);

impl TestError {
    fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
enum ProcessingError {
    #[error("Cancelled: {0}")]
    Cancelled(String),
    #[error("Other error: {0}")]
    Other(String),
}

#[tokio::test]
async fn test_subscribe_async_processes_items_when_waiting_per_item() {
    // Arrange
    let channel = FluxionChannel::new();
    let stream = channel.stream.map(|timestamped| timestamped.value);
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
                Ok::<(), TestError>(())
            }
        }
    };

    let error_callback = {
        move |err| {
            panic!("Unexpected error while processing: {err:?}");
        }
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_async(func, None, Some(error_callback))
                .await;
        }
    });

    // Act & Assert - wait for actual processing completion
    push(person_alice(), &channel.sender);
    notify_rx.recv().await.unwrap();
    assert_eq!(*results.lock().await, vec![person_alice()]);

    push(person_bob(), &channel.sender);
    notify_rx.recv().await.unwrap();
    assert_eq!(*results.lock().await, vec![person_alice(), person_bob()]);

    push(person_charlie(), &channel.sender);
    notify_rx.recv().await.unwrap();
    assert_eq!(
        *results.lock().await,
        vec![person_alice(), person_bob(), person_charlie()]
    );

    push(person_diane(), &channel.sender);
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

    push(person_dave(), &channel.sender);
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
    drop(channel.sender);
    task_handle.await.unwrap();
}

#[tokio::test]
async fn test_subscribe_async_reports_errors_for_animals_and_collects_people() {
    // Arrange
    let channel = FluxionChannel::new();
    let stream = channel.stream.map(|timestamped| timestamped.value);
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
                    return Err(TestError::new(
                        format!("Error processing animal: {item:?}",),
                    ));
                }
                results.lock().await.push(item);
                let _ = notify_tx.send(()); // Signal completion
                Ok::<(), TestError>(())
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
    push(person_alice(), &channel.sender);
    notify_rx.recv().await.unwrap();

    push(animal_dog(), &channel.sender); // Error
    notify_rx.recv().await.unwrap();

    push(person_bob(), &channel.sender);
    notify_rx.recv().await.unwrap();

    push(animal_cat(), &channel.sender); // Error
    notify_rx.recv().await.unwrap();

    push(person_charlie(), &channel.sender);
    notify_rx.recv().await.unwrap();

    // Assert final state
    assert_eq!(
        *results.lock().await,
        vec![person_alice(), person_bob(), person_charlie()]
    );
    assert_eq!(errors.lock().unwrap().len(), 2);

    // Cleanup
    drop(channel.sender);
    task_handle.await.unwrap();
}

#[tokio::test]
async fn test_subscribe_async_cancels_midstream_no_post_cancel_processing() {
    // Arrange
    let channel = FluxionChannel::new();
    let stream = channel.stream.map(|timestamped| timestamped.value);
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
                if ctx.is_cancelled() {
                    let _ = notify_tx.send(()); // Signal completion (cancelled)
                    return Ok(());
                }

                results.lock().await.push(item);
                let _ = notify_tx.send(()); // Signal completion

                Ok::<(), TestError>(())
            }
        }
    };

    let error_callback = {
        move |err| {
            panic!("Unexpected error while processing: {err:?}");
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
    push(person_alice(), &channel.sender);
    notify_rx.recv().await.unwrap();
    push(person_bob(), &channel.sender);
    notify_rx.recv().await.unwrap();

    assert_eq!(*results.lock().await, vec![person_alice(), person_bob()]);

    // Cancel and verify no more processing
    cancellation_token_clone.cancel();
    push(person_charlie(), &channel.sender);
    push(person_diane(), &channel.sender);

    // Close the stream and wait for the task to complete deterministically
    drop(channel.sender);
    task_handle.await.unwrap();

    // Assert no further items were processed after cancellation
    assert_eq!(*results.lock().await, vec![person_alice(), person_bob()]);
}

#[tokio::test]
async fn test_subscribe_async_errors_then_cancellation_no_post_cancel_processing() {
    // Arrange
    let channel = FluxionChannel::new();
    let stream = channel.stream.map(|timestamped| timestamped.value);
    let results = Arc::new(TokioMutex::new(Vec::new()));
    let errors = Arc::new(StdMutex::new(Vec::new()));
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
                if ctx.is_cancelled() {
                    let _ = notify_tx.send(());
                    return Err(ProcessingError::Cancelled(format!(
                        "Cancelled during processing of item: {item:?}"
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
    push(person_alice(), &channel.sender);
    notify_rx.recv().await.unwrap();
    push(person_bob(), &channel.sender);
    notify_rx.recv().await.unwrap();

    assert_eq!(*results.lock().await, vec![person_alice(), person_bob()]);
    assert!(errors.lock().unwrap().is_empty());

    // Send Charlie (which causes error)
    push(person_charlie(), &channel.sender);
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
    push(person_diane(), &channel.sender);
    push(animal_dog(), &channel.sender);

    // Close the stream and await task completion deterministically
    drop(channel.sender);
    task_handle.await.unwrap();

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
}

#[tokio::test]
async fn test_subscribe_async_empty_stream_completes_without_items() {
    // Arrange
    let channel = FluxionChannel::<TestData>::new();
    let stream = channel.stream.map(|timestamped| timestamped.value);
    let results = Arc::new(TokioMutex::new(Vec::new()));

    let func = {
        let results = results.clone();
        move |item, _ctx: CancellationToken| {
            let results = results.clone();
            async move {
                results.lock().await.push(item);
                Ok::<(), TestError>(())
            }
        }
    };

    let error_callback = {
        move |err| {
            panic!("Unexpected error while processing: {err:?}");
        }
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_async(func, None, Some(error_callback))
                .await;
        }
    });

    // Act - Close stream without sending any items
    drop(channel.sender);

    // Wait for task to complete
    task_handle.await.unwrap();

    // Assert
    assert_eq!(*results.lock().await, Vec::<TestData>::new());
}

#[tokio::test]
async fn test_subscribe_async_parallelism_max_active_ge_2() {
    // Arrange
    let channel = FluxionChannel::new();
    let stream = channel.stream.map(|timestamped| timestamped.value);
    let results = Arc::new(TokioMutex::new(Vec::new()));
    let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();

    // Concurrency counters
    let active = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let max_active = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    // Gate to hold completion so tasks overlap; and starter to know when tasks begin
    let (finish_tx, finish_rx) = mpsc::unbounded_channel::<()>();
    let finish_rx_shared = Arc::new(TokioMutex::new(finish_rx));
    let (started_tx, mut started_rx) = mpsc::unbounded_channel::<()>();

    let func = {
        let results = results.clone();
        let notify_tx = notify_tx.clone();
        let active = active.clone();
        let max_active = max_active.clone();
        let finish_rx_shared = finish_rx_shared.clone();
        let started_tx = started_tx.clone();
        move |item, _ctx: CancellationToken| {
            let results = results.clone();
            let notify_tx = notify_tx.clone();
            let active = active.clone();
            let max_active = max_active.clone();
            let finish_rx_shared = finish_rx_shared.clone();
            let started_tx = started_tx.clone();
            async move {
                // Mark task active and track max concurrency
                let cur = active.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                max_active.fetch_max(cur, std::sync::atomic::Ordering::SeqCst);
                let _ = started_tx.send(());

                // Wait until released
                let _ = finish_rx_shared.lock().await.recv().await;

                // Complete work
                active.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                results.lock().await.push(item);
                let _ = notify_tx.send(());
                Ok::<(), TestError>(())
            }
        }
    };

    let error_callback = {
        move |err| {
            panic!("Unexpected error while processing: {err:?}");
        }
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_async(func, None, Some(error_callback))
                .await;
        }
    });

    // Act - Send 3 items rapidly and ensure they overlap
    push(person_alice(), &channel.sender);
    push(person_bob(), &channel.sender);
    push(person_charlie(), &channel.sender);

    // Wait until all three tasks have started
    started_rx.recv().await.unwrap();
    started_rx.recv().await.unwrap();
    started_rx.recv().await.unwrap();

    // Release them to finish and await completions
    let _ = finish_tx.send(());
    let _ = finish_tx.send(());
    let _ = finish_tx.send(());
    notify_rx.recv().await.unwrap();
    notify_rx.recv().await.unwrap();
    notify_rx.recv().await.unwrap();

    // Assert
    {
        let processed = results.lock().await;
        assert_eq!(processed.len(), 3, "All 3 items should be processed");
        drop(processed);
    }
    let max = max_active.load(std::sync::atomic::Ordering::SeqCst);
    assert!(
        max >= 2,
        "Expected parallelism (max_active >= 2), got {max}"
    );

    // Cleanup
    drop(channel.sender);
    task_handle.await.unwrap();
}

#[tokio::test]
async fn test_subscribe_async_high_volume_processes_all() {
    // Arrange
    let channel = FluxionChannel::new();
    let stream = channel.stream.map(|timestamped| timestamped.value);
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
                Ok::<(), TestError>(())
            }
        }
    };

    let error_callback = {
        move |err| {
            panic!("Unexpected error while processing: {err:?}");
        }
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_async(func, None, Some(error_callback))
                .await;
        }
    });

    // Act - Send 100 items
    for _ in 0..100 {
        push(person_alice(), &channel.sender);
    }

    // Wait for all 100 to complete
    for _ in 0..100 {
        notify_rx.recv().await.unwrap();
    }

    // Assert
    {
        let processed = results.lock().await;
        assert_eq!(processed.len(), 100, "All 100 items should be processed");
        drop(processed);
    }

    // Cleanup
    drop(channel.sender);
    task_handle.await.unwrap();
}

#[tokio::test]
async fn test_subscribe_async_precancelled_token_processes_nothing() {
    // Arrange
    let channel = FluxionChannel::new();
    let stream = channel.stream.map(|timestamped| timestamped.value);
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
                Ok::<(), TestError>(())
            }
        }
    };

    let error_callback = {
        move |err| {
            panic!("Unexpected error while processing: {err:?}");
        }
    };

    tokio::spawn({
        async move {
            stream
                .subscribe_async(func, Some(cancellation_token), Some(error_callback))
                .await;
        }
    });

    // Act - Send items (should not be processed due to pre-cancelled token)
    push(person_alice(), &channel.sender);
    push(person_bob(), &channel.sender);
    push(person_charlie(), &channel.sender);
    push(person_diane(), &channel.sender);
    push(person_dave(), &channel.sender);
    drop(channel.sender);

    // Assert
    assert_eq!(*results.lock().await, Vec::<TestData>::new());
}
