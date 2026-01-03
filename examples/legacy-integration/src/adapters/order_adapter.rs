// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::domain::{events::UnifiedEvent, TimestampedEvent};
use crate::legacy::message_queue::LegacyMessageQueue;
use fluxion_core::CancellationToken;
use futures::channel::mpsc::unbounded;
use futures::{Stream, StreamExt};
use tokio::spawn;
use tokio::task::JoinHandle;

/// Order adapter that manages the legacy message queue consumer and timestamp wrapping
pub struct OrderAdapter {
    task_handle: Option<JoinHandle<()>>,
}

impl OrderAdapter {
    /// Create a new OrderAdapter
    pub fn new() -> Self {
        Self { task_handle: None }
    }

    /// Start consuming from the legacy message queue and return a stream of timestamped events
    pub fn start(
        &mut self,
        cancel_token: CancellationToken,
    ) -> impl Stream<Item = TimestampedEvent> + Send + Unpin {
        let (order_tx, order_rx) = unbounded();

        // Spawn legacy message queue consumer
        let mq = LegacyMessageQueue::new();
        let task_handle = spawn(async move {
            mq.consume_orders(order_tx, cancel_token).await;
        });

        self.task_handle = Some(task_handle);

        // Create stream that wraps with timestamps
        order_rx.map(|order| TimestampedEvent::new(UnifiedEvent::OrderReceived(order)))
    }

    /// Shutdown and wait for task completion
    pub async fn shutdown(mut self) {
        if let Some(handle) = self.task_handle.take() {
            let _ = handle.await;
        }
    }
}
