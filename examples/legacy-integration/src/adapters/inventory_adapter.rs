// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::CancellationToken;
use futures::channel::mpsc::unbounded;
use futures::{Stream, StreamExt};
use tokio::spawn;
use tokio::task::JoinHandle;

use crate::domain::{events::UnifiedEvent, TimestampedEvent};
use crate::legacy::file_watcher::LegacyFileWatcher;

pub struct InventoryAdapter {
    task_handle: Option<JoinHandle<()>>,
}

impl InventoryAdapter {
    pub fn new() -> Self {
        Self { task_handle: None }
    }

    pub fn start(
        &mut self,
        cancel_token: CancellationToken,
    ) -> impl Stream<Item = TimestampedEvent> + Send + Unpin {
        let (inventory_tx, inventory_rx) = unbounded();

        let fw = LegacyFileWatcher::new();
        let task_handle = spawn(async move {
            fw.watch_inventory(inventory_tx, cancel_token).await;
        });

        self.task_handle = Some(task_handle);

        inventory_rx
            .map(|inventory| TimestampedEvent::new(UnifiedEvent::InventoryUpdated(inventory)))
    }

    pub async fn shutdown(mut self) {
        if let Some(handle) = self.task_handle.take() {
            let _ = handle.await;
        }
    }
}
