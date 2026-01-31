// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::FluxionTask;
use futures::channel::oneshot;

#[tokio::test]
async fn test_task_cancellation_on_drop() {
    // Arrange
    let (tx, rx) = oneshot::channel();
    let task = FluxionTask::spawn(|cancel| async move {
        cancel.cancelled().await;
        let _ = tx.send(());
    });

    // Act
    drop(task);

    // Assert
    assert!(rx.await.is_ok());
}

#[tokio::test]
async fn test_task_manual_cancel() {
    // Arrange
    let (tx, rx) = oneshot::channel();
    let task = FluxionTask::spawn(|cancel| async move {
        cancel.cancelled().await;
        let _ = tx.send(());
    });

    // Act
    assert!(!task.is_cancelled());
    task.cancel();

    // Assert
    assert!(task.is_cancelled());
    assert!(rx.await.is_ok());
}
