// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::CancellationToken;
use futures::FutureExt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[test]
fn test_new_token_not_cancelled() {
    // Arrange & Act
    let token = CancellationToken::new();

    // Assert
    assert!(!token.is_cancelled());
}

#[test]
fn test_cancel_sets_flag() {
    // Arrange
    let token = CancellationToken::new();

    // Act
    token.cancel();

    // Assert
    assert!(token.is_cancelled());
}

#[test]
fn test_cancel_is_idempotent() {
    // Arrange
    let token = CancellationToken::new();

    // Act
    token.cancel();
    token.cancel();
    token.cancel();

    // Assert
    assert!(token.is_cancelled());
}

#[test]
fn test_clone_shares_state() {
    // Arrange
    let token1 = CancellationToken::new();
    let token2 = token1.clone();

    // Act
    assert!(!token1.is_cancelled());
    assert!(!token2.is_cancelled());
    token2.cancel();

    // Assert
    assert!(token1.is_cancelled());
    assert!(token2.is_cancelled());
}

#[tokio::test]
async fn test_cancelled_resolves_immediately_if_already_cancelled() {
    // Arrange
    let token = CancellationToken::new();
    token.cancel();

    // Act & Assert
    token.cancelled().await;
}

#[tokio::test]
async fn test_cancelled_waits_until_cancel() {
    // Arrange
    let token = CancellationToken::new();
    let token_clone = token.clone();

    let handle = tokio::spawn(async move {
        token_clone.cancelled().await;
        true
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Act
    token.cancel();

    // Assert
    let result = tokio::time::timeout(tokio::time::Duration::from_millis(100), handle).await;
    assert!(result.is_ok());
    assert!(result.unwrap().unwrap());
}

#[tokio::test]
async fn test_multiple_waiters_all_notified() {
    // Arrange
    let token = CancellationToken::new();

    let mut handles = vec![];
    for _ in 0..10 {
        let token_clone = token.clone();
        let handle = tokio::spawn(async move {
            token_clone.cancelled().await;
        });
        handles.push(handle);
    }

    // Act
    token.cancel();

    // Assert
    for handle in handles {
        let result = tokio::time::timeout(tokio::time::Duration::from_millis(100), handle).await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_cancel_before_first_poll() {
    // Arrange
    let token = CancellationToken::new();
    token.cancel();

    let mut future = Box::pin(token.cancelled());

    use core::future::Future;
    use core::task::{Context, Poll};

    let waker = futures::task::noop_waker();
    let mut cx = Context::from_waker(&waker);

    // Act & Assert
    match future.as_mut().poll(&mut cx) {
        Poll::Ready(()) => {}
        Poll::Pending => panic!("Expected Ready, got Pending"),
    }
}

#[tokio::test]
async fn test_race_condition_cancel_during_listen_registration() {
    // Arrange & Act
    for _ in 0..100 {
        let token = CancellationToken::new();
        let token_cancel = token.clone();

        let handle = tokio::spawn(async move {
            token.cancelled().await;
        });

        token_cancel.cancel();

        // Assert
        let result = tokio::time::timeout(tokio::time::Duration::from_millis(100), handle).await;
        assert!(result.is_ok());
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_basic_cancellation_flow() {
    // Arrange
    let token = CancellationToken::new();
    let token_worker = token.clone();

    let worker = tokio::spawn(async move {
        futures::select! {
            _ = token_worker.cancelled().fuse() => "cancelled",
            _ = tokio::time::sleep(Duration::from_secs(10)).fuse() => "timeout",
        }
    });

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Act
    token.cancel();

    // Assert
    let result = worker.await.unwrap();
    assert_eq!(result, "cancelled");
}

#[tokio::test]
async fn test_cancellation_propagates_to_all_clones() {
    // Arrange
    let token = CancellationToken::new();
    let count = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];
    for _ in 0..20 {
        let token_clone = token.clone();
        let count_clone = count.clone();

        let handle = tokio::spawn(async move {
            token_clone.cancelled().await;
            count_clone.fetch_add(1, Ordering::SeqCst);
        });
        handles.push(handle);
    }

    tokio::time::sleep(Duration::from_millis(20)).await;

    // Act
    token.cancel();

    // Assert
    for handle in handles {
        tokio::time::timeout(Duration::from_millis(100), handle)
            .await
            .expect("Task should complete")
            .expect("Task should not panic");
    }

    assert_eq!(count.load(Ordering::SeqCst), 20);
}

#[tokio::test]
async fn test_cancel_from_multiple_threads() {
    // Arrange
    let token = CancellationToken::new();
    let waiter_count = Arc::new(AtomicUsize::new(0));

    let mut waiter_handles = vec![];
    for _ in 0..10 {
        let token_clone = token.clone();
        let count_clone = waiter_count.clone();

        let handle = tokio::spawn(async move {
            token_clone.cancelled().await;
            count_clone.fetch_add(1, Ordering::SeqCst);
        });
        waiter_handles.push(handle);
    }

    let mut cancel_handles = vec![];
    for _ in 0..5 {
        let token_clone = token.clone();
        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            token_clone.cancel();
        });
        cancel_handles.push(handle);
    }

    // Act
    for handle in cancel_handles {
        handle.await.unwrap();
    }

    // Assert
    for handle in waiter_handles {
        tokio::time::timeout(Duration::from_millis(100), handle)
            .await
            .expect("Waiter should complete")
            .expect("Waiter should not panic");
    }

    assert_eq!(waiter_count.load(Ordering::SeqCst), 10);
}

#[tokio::test]
async fn test_late_waiters_see_cancellation() {
    // Arrange
    let token = CancellationToken::new();

    token.cancel();

    // Act
    let mut handles = vec![];
    for _ in 0..10 {
        let token_clone = token.clone();
        let handle = tokio::spawn(async move {
            token_clone.cancelled().await;
        });
        handles.push(handle);
    }

    // Assert
    for handle in handles {
        tokio::time::timeout(Duration::from_millis(50), handle)
            .await
            .expect("Late waiter should complete immediately")
            .expect("Late waiter should not panic");
    }
}

#[tokio::test]
async fn test_interleaved_wait_and_cancel() {
    // Arrange
    let token = CancellationToken::new();
    let completed = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];

    for _ in 0..5 {
        let token_clone = token.clone();
        let completed_clone = completed.clone();
        handles.push(tokio::spawn(async move {
            token_clone.cancelled().await;
            completed_clone.fetch_add(1, Ordering::SeqCst);
        }));
    }

    tokio::time::sleep(Duration::from_millis(5)).await;

    let token_cancel = token.clone();
    handles.push(tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(10)).await;
        token_cancel.cancel();
    }));

    for _ in 0..5 {
        let token_clone = token.clone();
        let completed_clone = completed.clone();
        handles.push(tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(5)).await;
            token_clone.cancelled().await;
            completed_clone.fetch_add(1, Ordering::SeqCst);
        }));
    }

    // Act & Assert
    for handle in handles {
        tokio::time::timeout(Duration::from_millis(200), handle)
            .await
            .expect("Task should complete")
            .ok();
    }

    assert_eq!(completed.load(Ordering::SeqCst), 10);
}

#[tokio::test]
async fn test_cancellation_with_select_macro() {
    // Arrange
    let token = CancellationToken::new();
    let token_work = token.clone();

    #[allow(clippy::never_loop)]
    let worker = tokio::spawn(async move {
        loop {
            futures::select! {
                _ = token_work.cancelled().fuse() => {
                    return "cancelled";
                }
                _ = tokio::time::sleep(Duration::from_secs(10)).fuse() => {
                    return "completed";
                }
            }
        }
    });
    #[deny(clippy::never_loop)]
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Act
    token.cancel();

    // Assert
    let result = tokio::time::timeout(Duration::from_millis(100), worker)
        .await
        .expect("Worker should complete after cancellation")
        .expect("Worker should not panic");

    assert_eq!(result, "cancelled");
}

#[tokio::test]
async fn test_drop_token_before_cancel() {
    // Arrange
    let token = CancellationToken::new();
    let token_waiter = token.clone();

    let waiter = tokio::spawn(async move {
        tokio::time::timeout(Duration::from_millis(100), token_waiter.cancelled()).await
    });

    // Act
    drop(token);

    // Assert
    let result = waiter.await.unwrap();
    assert!(result.is_err());
}

#[tokio::test]
async fn test_nested_cancellation_hierarchies() {
    // Arrange
    let parent = CancellationToken::new();

    let child1 = CancellationToken::new();
    let child2 = CancellationToken::new();

    let parent_clone = parent.clone();
    let child1_clone = child1.clone();
    let child2_clone = child2.clone();

    tokio::spawn(async move {
        parent_clone.cancelled().await;
        child1_clone.cancel();
        child2_clone.cancel();
    });

    let completed = Arc::new(AtomicUsize::new(0));

    let completed1 = completed.clone();
    let worker1 = tokio::spawn(async move {
        child1.cancelled().await;
        completed1.fetch_add(1, Ordering::SeqCst);
    });

    let completed2 = completed.clone();
    let worker2 = tokio::spawn(async move {
        child2.cancelled().await;
        completed2.fetch_add(1, Ordering::SeqCst);
    });

    tokio::time::sleep(Duration::from_millis(10)).await;

    // Act
    parent.cancel();

    // Assert
    tokio::time::timeout(Duration::from_millis(100), worker1)
        .await
        .unwrap()
        .unwrap();
    tokio::time::timeout(Duration::from_millis(100), worker2)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(completed.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn test_stress_high_contention() {
    // Arrange
    let token = CancellationToken::new();
    let completed = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];

    for _ in 0..100 {
        let token_clone = token.clone();
        let completed_clone = completed.clone();
        handles.push(tokio::spawn(async move {
            token_clone.cancelled().await;
            completed_clone.fetch_add(1, Ordering::SeqCst);
        }));
    }

    tokio::time::sleep(Duration::from_millis(20)).await;

    // Act
    token.cancel();

    // Assert
    for handle in handles {
        tokio::time::timeout(Duration::from_millis(500), handle)
            .await
            .expect("Waiter should complete")
            .expect("Waiter should not panic");
    }

    assert_eq!(completed.load(Ordering::SeqCst), 100);
}

#[tokio::test]
async fn test_is_cancelled_consistency() {
    // Arrange
    let token = CancellationToken::new();

    assert!(!token.is_cancelled());

    let token_clone = token.clone();
    assert!(!token_clone.is_cancelled());

    // Act
    token.cancel();

    // Assert
    assert!(token.is_cancelled());
    assert!(token_clone.is_cancelled());

    let token_new = token.clone();
    assert!(token_new.is_cancelled());
}
