// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Adapter that wraps legacy User records with timestamps
//! This is Pattern 3: Wrapper Ordering from the Integration Guide

use fluxion_core::CancellationToken;
use futures::channel::mpsc::unbounded;
use futures::{Stream, StreamExt};
use tokio::spawn;
use tokio::task::JoinHandle;

use crate::domain::{events::UnifiedEvent, TimestampedEvent};
use crate::legacy::database::LegacyDatabase;

pub struct UserAdapter {
    task_handle: Option<JoinHandle<()>>,
}

impl UserAdapter {
    pub fn new() -> Self {
        Self { task_handle: None }
    }

    pub fn start(
        &mut self,
        cancel_token: CancellationToken,
    ) -> impl Stream<Item = TimestampedEvent> + Send + Unpin {
        let (user_tx, user_rx) = unbounded();

        let db = LegacyDatabase::new();
        let task_handle = spawn(db.poll_users(user_tx, cancel_token));

        self.task_handle = Some(task_handle);

        user_rx.map(|user| TimestampedEvent::new(UnifiedEvent::UserAdded(user)))
    }

    pub async fn shutdown(mut self) {
        if let Some(handle) = self.task_handle.take() {
            let _ = handle.await;
        }
    }
}
