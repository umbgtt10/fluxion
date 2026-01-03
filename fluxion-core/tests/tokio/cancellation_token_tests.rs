// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Tests for runtime-agnostic CancellationToken.
//!
//! These tests verify the token works correctly across different scenarios
//! and edge cases that might occur in production use.

use fluxion_core::CancellationToken;
use futures::FutureExt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

// ============================================================================
// Unit Tests
// ============================================================================

#[test]
fn test_new_token_not_cancelled() {
    let token = CancellationToken::new();
    assert!(!token.is_cancelled());
}

#[test]
fn test_cancel_sets_flag() {
    let token = CancellationToken::new();
    token.cancel();
    assert!(token.is_cancelled());
}

#[test]
fn test_cancel_is_idempotent() {
    let token = CancellationToken::new();
    token.cancel();
    token.cancel();
    token.cancel();
    assert!(token.is_cancelled());
}

#[test]
fn test_clone_shares_state() {
    let token1 = CancellationToken::new();
    let token2 = token1.clone();

    assert!(!token1.is_cancelled());
    assert!(!token2.is_cancelled());

    token2.cancel();

    assert!(token1.is_cancelled());
    assert!(token2.is_cancelled());
}

#[tokio::test]
async fn test_cancelled_resolves_immediately_if_already_cancelled() {
    let token = CancellationToken::new();
    token.cancel();

    // Should resolve immediately without blocking
    token.cancelled().await;
}

#[tokio::test]
async fn test_cancelled_waits_until_cancel() {
    let token = CancellationToken::new();
    let token_clone = token.clone();

    let handle = tokio::spawn(async move {
        token_clone.cancelled().await;
        true
    });

    // Give the spawned task time to start waiting
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Now cancel
    token.cancel();

    // Task should complete
    let result = tokio::time::timeout(tokio::time::Duration::from_millis(100), handle).await;

    assert!(result.is_ok());
    assert!(result.unwrap().unwrap());
}

#[tokio::test]
async fn test_multiple_waiters_all_notified() {
    let token = CancellationToken::new();

    let mut handles = vec![];
    for _ in 0..10 {
        let token_clone = token.clone();
        let handle = tokio::spawn(async move {
            token_clone.cancelled().await;
        });
        handles.push(handle);
    }

    // Cancel once
    token.cancel();

    // All tasks should complete
    for handle in handles {
        let result = tokio::time::timeout(tokio::time::Duration::from_millis(100), handle).await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_cancel_before_first_poll() {
    let token = CancellationToken::new();
    token.cancel();

    // Create the future but don't poll it yet
    let mut future = Box::pin(token.cancelled());

    // Now poll it - should be Ready immediately
    use core::future::Future;
    use core::task::{Context, Poll};

    let waker = futures::task::noop_waker();
    let mut cx = Context::from_waker(&waker);

    match future.as_mut().poll(&mut cx) {
        Poll::Ready(()) => {}
        Poll::Pending => panic!("Expected Ready, got Pending"),
    }
}

#[tokio::test]
async fn test_race_condition_cancel_during_listen_registration() {
    // This test tries to catch a race condition where cancel()
    // is called between the first is_cancelled() check and listen() registration

    for _ in 0..100 {
        let token = CancellationToken::new();
        let token_cancel = token.clone();

        let handle = tokio::spawn(async move {
            token.cancelled().await;
        });

        // Cancel immediately (might race with listen() registration)
        token_cancel.cancel();

        // Should still complete successfully
        let result = tokio::time::timeout(tokio::time::Duration::from_millis(100), handle).await;
        assert!(result.is_ok(), "Task didn't complete after cancel");
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_basic_cancellation_flow() {
    let token = CancellationToken::new();
    let token_worker = token.clone();

    let worker = tokio::spawn(async move {
        futures::select! {
            _ = token_worker.cancelled().fuse() => "cancelled",
            _ = tokio::time::sleep(Duration::from_secs(10)).fuse() => "timeout",
        }
    });

    // Cancel after a short delay
    tokio::time::sleep(Duration::from_millis(10)).await;
    token.cancel();

    let result = worker.await.unwrap();
    assert_eq!(result, "cancelled");
}

#[tokio::test]
async fn test_cancellation_propagates_to_all_clones() {
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

    // Let all tasks start waiting
    tokio::time::sleep(Duration::from_millis(20)).await;

    // Cancel once
    token.cancel();

    // All tasks should complete
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
    let token = CancellationToken::new();
    let waiter_count = Arc::new(AtomicUsize::new(0));

    // Spawn waiters
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

    // Spawn multiple cancellers
    let mut cancel_handles = vec![];
    for _ in 0..5 {
        let token_clone = token.clone();
        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            token_clone.cancel();
        });
        cancel_handles.push(handle);
    }

    // Wait for all cancellers
    for handle in cancel_handles {
        handle.await.unwrap();
    }

    // All waiters should complete
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
    let token = CancellationToken::new();

    // Cancel immediately
    token.cancel();

    // Create new waiters after cancellation
    let mut handles = vec![];
    for _ in 0..10 {
        let token_clone = token.clone();
        let handle = tokio::spawn(async move {
            token_clone.cancelled().await;
        });
        handles.push(handle);
    }

    // All should complete immediately
    for handle in handles {
        tokio::time::timeout(Duration::from_millis(50), handle)
            .await
            .expect("Late waiter should complete immediately")
            .expect("Late waiter should not panic");
    }
}

#[tokio::test]
async fn test_interleaved_wait_and_cancel() {
    // Test scenario: waiters and cancellers interleaved
    let token = CancellationToken::new();
    let completed = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];

    // Spawn first batch of waiters
    for _ in 0..5 {
        let token_clone = token.clone();
        let completed_clone = completed.clone();
        handles.push(tokio::spawn(async move {
            token_clone.cancelled().await;
            completed_clone.fetch_add(1, Ordering::SeqCst);
        }));
    }

    tokio::time::sleep(Duration::from_millis(5)).await;

    // Spawn canceller
    let token_cancel = token.clone();
    handles.push(tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(10)).await;
        token_cancel.cancel();
    }));

    // Spawn second batch of waiters (before cancel)
    for _ in 0..5 {
        let token_clone = token.clone();
        let completed_clone = completed.clone();
        handles.push(tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(5)).await;
            token_clone.cancelled().await;
            completed_clone.fetch_add(1, Ordering::SeqCst);
        }));
    }

    // Wait for all
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
    // This test verifies that CancellationToken works correctly with futures::select!
    // We use a long-running operation to ensure cancellation is detected reliably
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
                    // This branch should never execute in this test
                    return "completed";
                }
            }
        }
    });
    #[deny(clippy::never_loop)]
    // Give the worker task a chance to start waiting
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Cancel the token
    token.cancel();

    // Worker should complete quickly with "cancelled"
    let result = tokio::time::timeout(Duration::from_millis(100), worker)
        .await
        .expect("Worker should complete after cancellation")
        .expect("Worker should not panic");

    assert_eq!(result, "cancelled");
}

#[tokio::test]
async fn test_drop_token_before_cancel() {
    let token = CancellationToken::new();
    let token_waiter = token.clone();

    let waiter = tokio::spawn(async move {
        tokio::time::timeout(Duration::from_millis(100), token_waiter.cancelled()).await
    });

    // Drop the original token without cancelling
    drop(token);

    // Waiter should timeout (not panic)
    let result = waiter.await.unwrap();
    assert!(result.is_err()); // Timeout error
}

#[tokio::test]
async fn test_nested_cancellation_hierarchies() {
    // Parent token
    let parent = CancellationToken::new();

    // Child tokens (separate instances, manually linked)
    let child1 = CancellationToken::new();
    let child2 = CancellationToken::new();

    let parent_clone = parent.clone();
    let child1_clone = child1.clone();
    let child2_clone = child2.clone();

    // Task that cancels children when parent is cancelled
    tokio::spawn(async move {
        parent_clone.cancelled().await;
        child1_clone.cancel();
        child2_clone.cancel();
    });

    // Workers waiting on children
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

    // Cancel parent
    tokio::time::sleep(Duration::from_millis(10)).await;
    parent.cancel();

    // All workers should complete
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
    // Stress test with many waiters and cancellers
    let token = CancellationToken::new();
    let completed = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];

    // Spawn 100 waiters
    for _ in 0..100 {
        let token_clone = token.clone();
        let completed_clone = completed.clone();
        handles.push(tokio::spawn(async move {
            token_clone.cancelled().await;
            completed_clone.fetch_add(1, Ordering::SeqCst);
        }));
    }

    // Let them all start
    tokio::time::sleep(Duration::from_millis(20)).await;

    // Cancel from one thread
    token.cancel();

    // All should complete quickly
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
    let token = CancellationToken::new();

    // Before cancel
    assert!(!token.is_cancelled());

    let token_clone = token.clone();
    assert!(!token_clone.is_cancelled());

    // Cancel
    token.cancel();

    // After cancel - all clones should see it
    assert!(token.is_cancelled());
    assert!(token_clone.is_cancelled());

    // New clone should also see it
    let token_new = token.clone();
    assert!(token_new.is_cancelled());
}
