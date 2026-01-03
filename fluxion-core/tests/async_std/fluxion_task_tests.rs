// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use fluxion_core::FluxionTask;
use futures::channel::oneshot;

#[async_std::test]
async fn test_task_cancellation_on_drop() {
    let (tx, rx) = oneshot::channel();

    let task = FluxionTask::spawn(|cancel| async move {
        cancel.cancelled().await;
        let _ = tx.send(());
    });

    drop(task);

    // Wait for task to signal completion
    assert!(rx.await.is_ok(), "Task should complete after cancellation");
}

#[async_std::test]
async fn test_task_manual_cancel() {
    let (tx, rx) = oneshot::channel();

    let task = FluxionTask::spawn(|cancel| async move {
        cancel.cancelled().await;
        let _ = tx.send(());
    });

    assert!(!task.is_cancelled());
    task.cancel();
    assert!(task.is_cancelled());

    // Wait for task to signal completion
    assert!(rx.await.is_ok(), "Task should complete after cancellation");
}
