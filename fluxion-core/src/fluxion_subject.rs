// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Hot, multi-subscriber subject for Fluxion streams.
//!
//! A `FluxionSubject` broadcasts each `StreamItem<T>` to all active subscribers.
//! - Hot: late subscribers do not see past items.
//! - Error/close: errors are propagated to all subscribers and then terminate the subject.

use crate::{FluxionError, StreamItem};
use futures::channel::mpsc::{self, UnboundedSender};
use futures::stream::{self, BoxStream, StreamExt};
use std::sync::{Arc, Mutex};

struct SubjectState<T> {
    closed: bool,
    senders: Vec<UnboundedSender<StreamItem<T>>>,
}

/// A hot, unbounded subject that broadcasts items to all current subscribers.
///
/// - Hot: late subscribers start receiving from the moment they subscribe.
/// - Unbounded: uses unbounded mpsc channels (no backpressure).
/// - Thread-safe: cheap to clone.
pub struct FluxionSubject<T: Clone + Send + 'static> {
    state: Arc<Mutex<SubjectState<T>>>,
}

impl<T: Clone + Send + 'static> FluxionSubject<T> {
    /// Create an unbounded subject.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(SubjectState {
                closed: false,
                senders: Vec::new(),
            })),
        }
    }

    /// Subscribe to this subject and receive a stream of `StreamItem<T>`.
    /// Late subscribers do not receive previously sent items.
    pub fn subscribe(&self) -> BoxStream<'static, StreamItem<T>> {
        let mut state = self.state.lock().unwrap();
        if state.closed {
            return stream::empty().boxed();
        }

        let (tx, rx) = mpsc::unbounded();
        state.senders.push(tx);
        rx.boxed()
    }

    /// Send an item to all active subscribers.
    /// Returns an error if the subject is closed.
    pub fn send(&self, item: StreamItem<T>) -> Result<(), FluxionError> {
        let mut state = self.state.lock().unwrap();
        if state.closed {
            return Err(FluxionError::stream_error("Subject is closed"));
        }

        let mut next_senders = Vec::with_capacity(state.senders.len());

        for tx in state.senders.drain(..) {
            if tx.unbounded_send(item.clone()).is_ok() {
                next_senders.push(tx);
            }
        }

        state.senders = next_senders;
        Ok(())
    }

    /// Convenience helper to send an error to all subscribers and terminate the subject.
    pub fn error(&self, err: FluxionError) -> Result<(), FluxionError> {
        let result = self.send(StreamItem::Error(err));
        self.close();
        result
    }

    /// Close the subject, completing all subscribers.
    pub fn close(&self) {
        let mut state = self.state.lock().unwrap();
        state.closed = true;
        state.senders.clear();
    }
}

impl<T: Clone + Send + 'static> Default for FluxionSubject<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Send + 'static> Clone for FluxionSubject<T> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}
