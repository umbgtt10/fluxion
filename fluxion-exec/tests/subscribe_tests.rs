// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use fluxion_core::CancellationToken;
use fluxion_exec::subscribe::SubscribeExt;
use fluxion_test_utils::test_data::{
    animal_cat, animal_dog, person_alice, person_bob, person_charlie, person_dave, person_diane,
    TestData,
};
use fluxion_test_utils::Sequenced;
use futures::channel::mpsc::unbounded;
use futures::lock::Mutex as FutureMutex;
use std::{sync::Arc, sync::Mutex as StdMutex};
use tokio::spawn;
use tokio_stream::StreamExt as _;

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
async fn test_subscribe_processes_items_when_waiting_per_item() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = unbounded::<Sequenced<TestData>>();
    let stream = rx;
    let stream = stream.map(|timestamped| timestamped.value);
    let results = Arc::new(FutureMutex::new(Vec::new()));
    let (notify_tx, mut notify_rx) = unbounded();

    let func = {
        let results = results.clone();
        let notify_tx = notify_tx.clone();
        move |item, _ctx: CancellationToken| {
            let results = results.clone();
            let notify_tx = notify_tx.clone();
            async move {
                results.lock().await.push(item);
                let _ = notify_tx.unbounded_send(()); // Signal completion
                Ok::<(), TestError>(())
            }
        }
    };

    let error_callback = {
        move |err| {
            panic!("Unexpected error while processing: {err:?}");
        }
    };

    let task_handle = spawn({
        async move {
            stream
                .subscribe(func, error_callback, None)
                .await
                .expect("subscribe should succeed");
        }
    });

    // Act & Assert - wait for actual processing completion
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    notify_rx.next().await.unwrap();
    assert_eq!(*results.lock().await, vec![person_alice()]);

    tx.unbounded_send(Sequenced::new(person_bob()))?;
    notify_rx.next().await.unwrap();
    assert_eq!(*results.lock().await, vec![person_alice(), person_bob()]);

    tx.unbounded_send(Sequenced::new(person_charlie()))?;
    notify_rx.next().await.unwrap();
    assert_eq!(
        *results.lock().await,
        vec![person_alice(), person_bob(), person_charlie()]
    );

    tx.unbounded_send(Sequenced::new(person_diane()))?;
    notify_rx.next().await.unwrap();
    assert_eq!(
        *results.lock().await,
        vec![
            person_alice(),
            person_bob(),
            person_charlie(),
            person_diane()
        ]
    );

    tx.unbounded_send(Sequenced::new(person_dave()))?;
    notify_rx.next().await.unwrap();
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
    drop(tx);
    task_handle.await.unwrap();

    Ok(())
}

#[tokio::test]
async fn test_subscribe_reports_errors_for_animals_and_collects_people() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = unbounded::<Sequenced<TestData>>();
    let stream = rx;
    let stream = stream.map(|timestamped| timestamped.value);
    let results = Arc::new(FutureMutex::new(Vec::new()));
    let errors = Arc::new(StdMutex::new(Vec::new()));
    let (notify_tx, mut notify_rx) = unbounded();

    let func = {
        let results = results.clone();
        let notify_tx = notify_tx.clone();
        move |item: TestData, _ctx: CancellationToken| {
            let results = results.clone();
            let notify_tx = notify_tx.clone();
            async move {
                // Error on every animal
                if matches!(&item, TestData::Animal(_)) {
                    let _ = notify_tx.unbounded_send(()); // Signal completion (error case)
                    return Err(TestError::new(
                        format!("Error processing animal: {item:?}",),
                    ));
                }
                results.lock().await.push(item);
                let _ = notify_tx.unbounded_send(()); // Signal completion
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

    let task_handle = spawn({
        async move {
            stream
                .subscribe(func, error_callback, None)
                .await
                .expect("subscribe should succeed");
        }
    });

    // Act & Assert - wait for processing completion
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    notify_rx.next().await.unwrap();

    tx.unbounded_send(Sequenced::new(animal_dog()))?; // Error
    notify_rx.next().await.unwrap();

    tx.unbounded_send(Sequenced::new(person_bob()))?;
    notify_rx.next().await.unwrap();

    tx.unbounded_send(Sequenced::new(animal_cat()))?; // Error
    notify_rx.next().await.unwrap();

    tx.unbounded_send(Sequenced::new(person_charlie()))?;
    notify_rx.next().await.unwrap();

    // Assert final state
    assert_eq!(
        *results.lock().await,
        vec![person_alice(), person_bob(), person_charlie()]
    );
    assert_eq!(errors.lock().unwrap().len(), 2);

    // Cleanup
    drop(tx);
    task_handle.await.unwrap();

    Ok(())
}

#[tokio::test]
async fn test_subscribe_cancels_midstream_no_post_cancel_processing() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = unbounded::<Sequenced<TestData>>();
    let stream = rx;
    let stream = stream.map(|timestamped| timestamped.value);
    let results = Arc::new(FutureMutex::new(Vec::new()));
    let cancellation_token = CancellationToken::new();
    let cancellation_token_clone = cancellation_token.clone();
    let (notify_tx, mut notify_rx) = unbounded();

    let func = {
        let results = results.clone();
        let notify_tx = notify_tx.clone();
        move |item: TestData, ctx: CancellationToken| {
            let results = results.clone();
            let notify_tx = notify_tx.clone();
            async move {
                if ctx.is_cancelled() {
                    let _ = notify_tx.unbounded_send(()); // Signal completion (cancelled)
                    return Ok(());
                }

                results.lock().await.push(item);
                let _ = notify_tx.unbounded_send(()); // Signal completion

                Ok::<(), TestError>(())
            }
        }
    };

    let error_callback = {
        move |err| {
            panic!("Unexpected error while processing: {err:?}");
        }
    };

    let task_handle = spawn({
        async move {
            stream
                .subscribe(func, error_callback, Some(cancellation_token))
                .await
                .expect("subscribe should succeed");
        }
    });

    // Act & Assert
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    notify_rx.next().await.unwrap();
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    notify_rx.next().await.unwrap();

    assert_eq!(*results.lock().await, vec![person_alice(), person_bob()]);

    // Cancel and verify no more processing
    cancellation_token_clone.cancel();
    tx.unbounded_send(Sequenced::new(person_charlie()))?;
    tx.unbounded_send(Sequenced::new(person_diane()))?;

    // Close the stream and wait for the task to complete deterministically
    drop(tx);
    task_handle.await.unwrap();

    // Assert no further items were processed after cancellation
    assert_eq!(*results.lock().await, vec![person_alice(), person_bob()]);

    Ok(())
}

#[tokio::test]
async fn test_subscribe_errors_then_cancellation_no_post_cancel_processing() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = unbounded::<Sequenced<TestData>>();
    let stream = rx;
    let stream = stream.map(|timestamped| timestamped.value);
    let results = Arc::new(FutureMutex::new(Vec::new()));
    let errors = Arc::new(StdMutex::new(Vec::new()));
    let cancellation_token = CancellationToken::new();
    let cancellation_token_clone = cancellation_token.clone();
    let (notify_tx, mut notify_rx) = unbounded();

    let func = {
        let results = results.clone();
        let notify_tx = notify_tx.clone();
        move |item: TestData, ctx: CancellationToken| {
            let results = results.clone();
            let notify_tx = notify_tx.clone();
            async move {
                if ctx.is_cancelled() {
                    let _ = notify_tx.unbounded_send(());
                    return Err(ProcessingError::Cancelled(format!(
                        "Cancelled during processing of item: {item:?}"
                    )));
                }

                results.lock().await.push(item.clone());
                let _ = notify_tx.unbounded_send(());

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

    let task_handle = spawn({
        async move {
            stream
                .subscribe(func, error_callback, Some(cancellation_token))
                .await
                .expect("subscribe should succeed");
        }
    });

    // Act & Assert
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    notify_rx.next().await.unwrap();
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    notify_rx.next().await.unwrap();

    assert_eq!(*results.lock().await, vec![person_alice(), person_bob()]);
    assert!(errors.lock().unwrap().is_empty());

    // Send Charlie (which causes error)
    tx.unbounded_send(Sequenced::new(person_charlie()))?;
    notify_rx.next().await.unwrap();

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
    tx.unbounded_send(Sequenced::new(person_diane()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Close the stream and await task completion deterministically
    drop(tx);
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

    Ok(())
}

#[tokio::test]
async fn test_subscribe_empty_stream_completes_without_items() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = unbounded::<Sequenced<TestData>>();
    let stream = rx;
    let stream = stream.map(|timestamped| timestamped.value);
    let results = Arc::new(FutureMutex::new(Vec::new()));

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

    let task_handle = spawn({
        async move {
            stream
                .subscribe(func, error_callback, None)
                .await
                .expect("subscribe should succeed");
        }
    });

    // Act - Close stream without sending any items
    drop(tx);

    // Wait for task to complete
    task_handle.await.unwrap();

    // Assert
    assert_eq!(*results.lock().await, Vec::<TestData>::new());

    Ok(())
}

#[tokio::test]
async fn test_subscribe_high_volume_processes_all() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = unbounded::<Sequenced<TestData>>();
    let stream = rx;
    let stream = stream.map(|timestamped| timestamped.value);
    let results = Arc::new(FutureMutex::new(Vec::new()));
    let (notify_tx, mut notify_rx) = unbounded();

    let func = {
        let results = results.clone();
        let notify_tx = notify_tx.clone();
        move |item, _ctx: CancellationToken| {
            let results = results.clone();
            let notify_tx = notify_tx.clone();
            async move {
                results.lock().await.push(item);
                let _ = notify_tx.unbounded_send(());
                Ok::<(), TestError>(())
            }
        }
    };

    let error_callback = {
        move |err| {
            panic!("Unexpected error while processing: {err:?}");
        }
    };

    let task_handle = spawn({
        async move {
            stream
                .subscribe(func, error_callback, None)
                .await
                .expect("subscribe should succeed");
        }
    });

    // Act - Send 100 items
    for _ in 0..100 {
        tx.unbounded_send(Sequenced::new(person_alice()))?;
    }

    // Wait for all 100 to complete
    for _ in 0..100 {
        notify_rx.next().await.unwrap();
    }

    // Assert
    {
        let processed = results.lock().await;
        assert_eq!(processed.len(), 100, "All 100 items should be processed");
        drop(processed);
    }

    // Cleanup
    drop(tx);
    task_handle.await.unwrap();

    Ok(())
}

#[tokio::test]
async fn test_subscribe_precancelled_token_processes_nothing() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = unbounded::<Sequenced<TestData>>();
    let stream = rx;
    let stream = stream.map(|timestamped| timestamped.value);
    let results = Arc::new(FutureMutex::new(Vec::new()));
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

    spawn({
        async move {
            stream
                .subscribe(func, error_callback, Some(cancellation_token))
                .await
                .expect("subscribe should succeed");
        }
    });

    // Act - Send items (should not be processed due to pre-cancelled token)
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(person_charlie()))?;
    tx.unbounded_send(Sequenced::new(person_diane()))?;
    tx.unbounded_send(Sequenced::new(person_dave()))?;
    drop(tx);

    // Assert
    assert_eq!(*results.lock().await, Vec::<TestData>::new());

    Ok(())
}

#[tokio::test]
async fn test_subscribe_error_aggregation_without_callback() -> anyhow::Result<()> {
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

    let task_handle = spawn({
        async move {
            stream
                .subscribe(
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
