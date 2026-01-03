// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use fluxion_core::CancellationToken;
use futures::channel::mpsc::unbounded;
use futures::{Stream, StreamExt};
use tokio::spawn;
use tokio::task::JoinHandle;

use crate::domain::{events::UnifiedEvent, TimestampedEvent};
use crate::legacy::file_watcher::LegacyFileWatcher;

/// Inventory adapter that manages the legacy file watcher and timestamp wrapping
pub struct InventoryAdapter {
    task_handle: Option<JoinHandle<()>>,
}

impl InventoryAdapter {
    /// Create a new InventoryAdapter
    pub fn new() -> Self {
        Self { task_handle: None }
    }

    /// Start watching legacy inventory files and return a stream of timestamped events
    pub fn start(
        &mut self,
        cancel_token: CancellationToken,
    ) -> impl Stream<Item = TimestampedEvent> + Send + Unpin {
        let (inventory_tx, inventory_rx) = unbounded();

        // Spawn legacy file watcher
        let fw = LegacyFileWatcher::new();
        let task_handle = spawn(async move {
            fw.watch_inventory(inventory_tx, cancel_token).await;
        });

        self.task_handle = Some(task_handle);

        // Create stream that wraps with timestamps
        inventory_rx
            .map(|inventory| TimestampedEvent::new(UnifiedEvent::InventoryUpdated(inventory)))
    }

    /// Shutdown and wait for task completion
    pub async fn shutdown(mut self) {
        if let Some(handle) = self.task_handle.take() {
            let _ = handle.await;
        }
    }
}
