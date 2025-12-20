// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Adapter that wraps legacy User records with timestamps
//! This is Pattern 3: Wrapper Ordering from the Integration Guide

use fluxion_core::CancellationToken;
use futures::{Stream, StreamExt};
use tokio::spawn;
use tokio::sync::mpsc::unbounded_channel;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::domain::{events::UnifiedEvent, TimestampedEvent};
use crate::legacy::database::LegacyDatabase;

/// User adapter that manages the legacy database polling and timestamp wrapping
pub struct UserAdapter {
    task_handle: Option<JoinHandle<()>>,
}

impl UserAdapter {
    /// Create a new UserAdapter
    pub fn new() -> Self {
        Self { task_handle: None }
    }

    /// Start polling the legacy database and return a stream of timestamped events
    pub fn start(
        &mut self,
        cancel_token: CancellationToken,
    ) -> impl Stream<Item = TimestampedEvent> + Send + Unpin {
        let (user_tx, user_rx) = unbounded_channel();

        // Spawn legacy database poller
        let db = LegacyDatabase::new();
        let task_handle = spawn(db.poll_users(user_tx, cancel_token));

        self.task_handle = Some(task_handle);

        // Create stream that wraps with timestamps
        UnboundedReceiverStream::new(user_rx)
            .map(|user| TimestampedEvent::new(UnifiedEvent::UserAdded(user)))
    }

    /// Shutdown and wait for task completion
    pub async fn shutdown(mut self) {
        if let Some(handle) = self.task_handle.take() {
            let _ = handle.await;
        }
    }
}
