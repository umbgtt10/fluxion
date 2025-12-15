// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Shared stream subscription factory for Fluxion streams.
//!
//! A [`FluxionShared`] converts a cold stream into a hot, multi-subscriber source.
//! It consumes the original stream and broadcasts each item to all active subscribers.
//!
//! ## Characteristics
//!
//! - **Hot**: Late subscribers do not receive past items—only items emitted after subscribing.
//! - **Shared execution**: The source stream is consumed once; results are broadcast to all.
//! - **Subscription factory**: Call `subscribe()` to create independent subscriber streams.
//! - **Owned lifecycle**: The forwarding task is owned and cancelled when dropped.
//!
//! ## Example
//!
//! ```rust
//! use fluxion_stream::{IntoFluxionStream, ShareExt, MapOrderedExt, FilterOrderedExt};
//! use fluxion_test_utils::Sequenced;
//!
//! # async fn example() {
//! let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
//!
//! // Create a source stream
//! let source = rx.into_fluxion_stream()
//!     .map_ordered(|x: Sequenced<i32>| Sequenced::new(x.into_inner() * 2));
//!
//! // Share it among multiple subscribers
//! let shared = source.share();
//!
//! // Each subscriber gets broadcast values, can chain independently
//! let _sub1 = shared.subscribe().unwrap()
//!     .filter_ordered(|x| *x > 10);
//!
//! let _sub2 = shared.subscribe().unwrap()
//!     .map_ordered(|x: Sequenced<i32>| Sequenced::new(x.into_inner().to_string()));
//! # }
//! ```
//!
//! ## Comparison with FluxionSubject
//!
//! | Type | Source | Push API |
//! |------|--------|----------|
//! | [`FluxionSubject`] | External (you call `next()`) | Yes |
//! | [`FluxionShared`] | Existing stream | No |
//!
//! Both are subscription factories with the same `subscribe()` pattern.

use fluxion_core::{FluxionSubject, StreamItem, SubjectError};
use futures::{Stream, StreamExt};
use std::pin::Pin;
use tokio::task::JoinHandle;

/// Type alias for the boxed stream returned by `subscribe()`.
pub type SharedBoxStream<T> = Pin<Box<dyn Stream<Item = StreamItem<T>> + Send + Sync + 'static>>;

/// A shared stream that broadcasts items from a source to multiple subscribers.
///
/// `FluxionShared` is created by calling [`share()`](ShareExt::share) on a
/// `FluxionStream`. It consumes the source stream and forwards all items to an internal
/// [`FluxionSubject`], which broadcasts to all subscribers.
///
/// This is a **subscription factory**, not a stream itself. Call `subscribe()` to obtain
/// streams that can be wrapped in `FluxionStream` for chaining operators.
///
/// See the [module documentation](self) for examples and more details.
pub struct FluxionShared<T: Clone + Send + Sync + 'static> {
    subject: FluxionSubject<T>,
    _task: JoinHandle<()>,
}

impl<T: Clone + Send + Sync + 'static> FluxionShared<T> {
    /// Creates a new `FluxionShared` from a source stream.
    ///
    /// This immediately spawns a task that consumes the source stream and forwards
    /// all items to the internal subject. The task runs until the source completes,
    /// errors, or all subscribers have been dropped.
    ///
    /// Prefer using [`ShareExt::share()`] instead
    /// of calling this directly.
    pub fn new<S>(source: S) -> Self
    where
        S: Stream<Item = StreamItem<T>> + Send + Sync + Unpin + 'static,
    {
        let subject = FluxionSubject::new();
        let subject_clone = subject.clone();

        let task = tokio::spawn(async move {
            let mut stream = source;
            while let Some(item) = stream.next().await {
                match item {
                    StreamItem::Value(v) => {
                        if subject_clone.next(v).is_err() {
                            // Subject closed (all subscribers dropped or explicit close)
                            break;
                        }
                    }
                    StreamItem::Error(e) => {
                        // Propagate error to all subscribers and terminate
                        let _ = subject_clone.error(e);
                        break;
                    }
                }
            }
            // Source completed, close the subject
            subject_clone.close();
        });

        Self {
            subject,
            _task: task,
        }
    }

    /// Subscribe to this shared source and receive a stream of items.
    ///
    /// Late subscribers do not receive previously emitted items.
    /// Wrap the result in `FluxionStream::new()` to access chaining operators.
    ///
    /// # Errors
    ///
    /// Returns `SubjectError::Closed` if the source has completed or errored.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fluxion_stream::{IntoFluxionStream, ShareExt, FilterOrderedExt, MapOrderedExt};
    /// use fluxion_test_utils::Sequenced;
    ///
    /// # async fn example() {
    /// let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Sequenced<i32>>();
    /// let source = rx.into_fluxion_stream();
    /// let shared = source.share();
    ///
    /// let _stream = shared.subscribe().unwrap();
    /// # }
    /// ```
    pub fn subscribe(&self) -> Result<SharedBoxStream<T>, SubjectError> {
        self.subject.subscribe()
    }

    /// Returns `true` if the shared source has completed or errored.
    ///
    /// A closed `FluxionShared` cannot produce new subscribers.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.subject.is_closed()
    }

    /// Returns the number of currently active subscribers.
    ///
    /// Note: This count is updated lazily—dropped subscribers are removed
    /// on the next emission, not immediately when dropped.
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.subject.subscriber_count()
    }
}

impl<T: Clone + Send + Sync + 'static> Drop for FluxionShared<T> {
    fn drop(&mut self) {
        // Close the subject to signal any remaining subscribers
        self.subject.close();
        // Task will be aborted when JoinHandle is dropped
    }
}

/// Extension trait providing the `share` operator for streams.
///
/// This trait allows any stream of `StreamItem<T>` to be converted into a
/// [`FluxionShared`] subscription factory for multi-subscriber broadcasting.
pub trait ShareExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Clone + Send + Sync + 'static,
{
    /// Converts this stream into a shared, multi-subscriber source.
    ///
    /// `share()` consumes this stream and returns a [`FluxionShared`] subscription factory.
    /// The source stream is consumed once, and all emitted items are broadcast to all
    /// active subscribers.
    ///
    /// This is useful when you have an expensive computation or external data source
    /// that you want to share among multiple consumers without re-executing the source.
    ///
    /// # Behavior
    ///
    /// - **Hot**: Late subscribers do not receive past items
    /// - **Shared execution**: Source operators run once; results are broadcast
    /// - **Owned lifecycle**: The forwarding task is cancelled when `FluxionShared` is dropped
    ///
    /// # Returns
    ///
    /// A [`FluxionShared`] subscription factory. Call `subscribe()` on it to create
    /// subscriber streams, then wrap in `FluxionStream::new()` to chain operators.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fluxion_stream::{IntoFluxionStream, ShareExt, FilterOrderedExt, MapOrderedExt};
    /// use fluxion_test_utils::Sequenced;
    ///
    /// # async fn example() {
    /// let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Sequenced<i32>>();
    ///
    /// // Source (runs ONCE)
    /// let source = rx.into_fluxion_stream()
    ///     .map_ordered(|x: Sequenced<i32>| Sequenced::new(x.into_inner() * 2));
    ///
    /// // Share among multiple subscribers
    /// let shared = source.share();
    ///
    /// // Each subscriber chains independently
    /// let _sub1 = shared.subscribe().unwrap();
    /// let _sub2 = shared.subscribe().unwrap();
    /// # }
    /// ```
    ///
    /// # See Also
    ///
    /// - [`FluxionSubject`] - For push-based multicast
    /// - [`FluxionShared`] - The returned subscription factory
    fn share(self) -> FluxionShared<T>
    where
        Self: Send + Sync + Unpin + 'static,
    {
        FluxionShared::new(self)
    }
}

impl<S, T> ShareExt<T> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Clone + Send + Sync + 'static,
{
}
