// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Hot, multi-subscriber subject for Fluxion streams.
//!
//! A [`FluxionSubject`] broadcasts each [`StreamItem<T>`] to all active subscribers.
//!
//! ## Characteristics
//!
//! - **Hot**: Late subscribers do not receive past items—only items sent after subscribing.
//! - **Unbounded**: Uses unbounded mpsc channels internally (no backpressure).
//! - **Thread-safe**: Cheap to clone; all clones share the same internal state.
//! - **Error/close**: Errors are propagated to all subscribers and terminate the subject.
//!
//! ## Example
//!
//! ```
//! use fluxion_core::{FluxionSubject, StreamItem};
//! use futures::StreamExt;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let subject = FluxionSubject::<i32>::new();
//!
//! // Subscribe before sending
//! let mut stream = subject.subscribe().unwrap();
//!
//! // Send values to all subscribers
//! subject.send(StreamItem::Value(1)).unwrap();
//! subject.send(StreamItem::Value(2)).unwrap();
//! subject.close();
//!
//! // Receive values
//! assert_eq!(stream.next().await, Some(StreamItem::Value(1)));
//! assert_eq!(stream.next().await, Some(StreamItem::Value(2)));
//! assert_eq!(stream.next().await, None); // Subject closed
//! # }

use crate::{FluxionError, StreamItem, SubjectError};
use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use futures::Stream;
use parking_lot::Mutex;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

type SubjectBoxStream<T> = Pin<Box<dyn Stream<Item = StreamItem<T>> + Send + Sync + 'static>>;

struct SubjectState<T> {
    closed: bool,
    senders: Vec<UnboundedSender<StreamItem<T>>>,
}

// A Sync-capable wrapper around the unbounded receiver used by FluxionSubject subscriptions.
struct SubjectStream<T> {
    inner: Arc<Mutex<UnboundedReceiver<StreamItem<T>>>>,
}

impl<T: Clone + Send + Sync + 'static> SubjectStream<T> {
    fn into_boxed_stream(rx: UnboundedReceiver<StreamItem<T>>) -> SubjectBoxStream<T> {
        Box::pin(Self {
            inner: Arc::new(Mutex::new(rx)),
        })
    }
}

impl<T: Clone + Send + Sync + 'static> Stream for SubjectStream<T> {
    type Item = StreamItem<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut guard = self.inner.lock();
        Pin::new(&mut *guard).poll_next(cx)
    }
}

/// A hot, unbounded subject that broadcasts items to all current subscribers.
///
/// `FluxionSubject` is the entry point for pushing values into a Fluxion stream pipeline.
/// It implements a publish-subscribe pattern where multiple subscribers can receive
/// the same items.
///
/// See the [module documentation](self) for examples and more details.
pub struct FluxionSubject<T: Clone + Send + Sync + 'static> {
    state: Arc<Mutex<SubjectState<T>>>,
}

impl<T: Clone + Send + Sync + 'static> FluxionSubject<T> {
    /// Creates a new unbounded subject with no subscribers.
    ///
    /// The subject starts in an open state and can immediately accept
    /// subscriptions and items.
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
    pub fn subscribe(&self) -> Result<SubjectBoxStream<T>, SubjectError> {
        let mut state = self.state.lock();
        if state.closed {
            return Err(SubjectError::Closed);
        }

        let (tx, rx) = mpsc::unbounded();
        state.senders.push(tx);
        Ok(SubjectStream::into_boxed_stream(rx))
    }

    /// Send an item to all active subscribers.
    ///
    /// Returns a subject-specific error if the operation fails:
    /// - `SubjectError::Closed` if the subject has been closed
    /// - `SubjectError::NoSubscribers` if no subscribers remain (optional check)
    pub fn send(&self, item: StreamItem<T>) -> Result<(), SubjectError> {
        let mut state = self.state.lock();
        if state.closed {
            return Err(SubjectError::Closed);
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

    /// Send a value to all active subscribers.
    ///
    /// This is a convenience wrapper around `send(StreamItem::Value(value))`.
    ///
    /// # Errors
    ///
    /// Returns `SubjectError::Closed` if the subject has been closed.
    pub fn next(&self, value: T) -> Result<(), SubjectError> {
        self.send(StreamItem::Value(value))
    }

    /// Convenience helper to send a stream error to all subscribers and terminate the subject.
    ///
    /// This bridges stream errors (FluxionError) with subject operations (SubjectError).
    pub fn error(&self, err: FluxionError) -> Result<(), SubjectError> {
        let result = self.send(StreamItem::Error(err));
        self.close();
        result
    }

    /// Closes the subject, completing all subscriber streams.
    ///
    /// After closing:
    /// - All existing subscribers will receive `None` on their next poll (stream ends).
    /// - `send()` and `error()` will return `SubjectError::Closed`.
    /// - `subscribe()` will return `SubjectError::Closed`.
    ///
    /// Closing is idempotent—calling it multiple times has no additional effect.
    pub fn close(&self) {
        let mut state = self.state.lock();
        state.closed = true;
        state.senders.clear();
    }

    /// Returns `true` if the subject has been closed.
    ///
    /// A closed subject cannot accept new items or subscribers.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.state.lock().closed
    }

    /// Returns the number of currently active subscribers.
    ///
    /// Note: This count is updated lazily—dropped subscribers are removed
    /// on the next `send()` call, not immediately when dropped.
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.state.lock().senders.len()
    }
}

impl<T: Clone + Send + Sync + 'static> Default for FluxionSubject<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Send + Sync + 'static> Clone for FluxionSubject<T> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}
