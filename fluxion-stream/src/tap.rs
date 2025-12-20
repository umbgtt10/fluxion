// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Side-effect operator for debugging and troubleshooting streams.
//!
//! This module provides the [`tap`](TapExt::tap) operator that allows observing
//! stream items without modifying them, useful for debugging complex pipelines.
//!
//! # Overview
//!
//! The `tap` operator invokes a side-effect function for each value without
//! affecting the stream. It's the reactive equivalent of inserting `println!`
//! statements for debugging.
//!
//! # Basic Usage
//!
//! ```
//! use fluxion_stream::prelude::*;
//! use fluxion_test_utils::{Sequenced, test_channel};
//! use futures::StreamExt;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let (tx, stream) = test_channel::<Sequenced<i32>>();
//!
//! let mut tapped = stream
//!     .tap(|value| println!("Observed: {:?}", value));
//!
//! tx.unbounded_send(Sequenced::new(42)).unwrap();
//! drop(tx);
//!
//! // Value passes through unchanged
//! let item = tapped.next().await.unwrap();
//! assert_eq!(item.unwrap().into_inner(), 42);
//! # }
//! ```
//!
//! # Debugging Pipelines
//!
//! Insert `tap` at any point in a pipeline to observe intermediate values:
//!
//! ```
//! use fluxion_stream::prelude::*;
//! use fluxion_test_utils::{Sequenced, test_channel};
//! use futures::StreamExt;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let (tx, stream) = test_channel::<Sequenced<i32>>();
//!
//! let mut processed = stream
//!     .tap(|v| println!("Before filter: {:?}", v))
//!     .filter_ordered(|x| *x > 10)
//!     .tap(|v| println!("After filter: {:?}", v))
//!     .map_ordered(|x| Sequenced::new(x.into_inner() * 2))
//!     .tap(|v| println!("Final value: {:?}", v));
//!
//! tx.unbounded_send(Sequenced::new(42)).unwrap();
//! drop(tx);
//!
//! let result = processed.next().await.unwrap().unwrap();
//! assert_eq!(result.into_inner(), 84);
//! # }
//! ```
//!
//! # Error Handling
//!
//! The tap function is only called for values, not errors. Errors pass through
//! unchanged without invoking the tap function.

use fluxion_core::{Fluxion, StreamItem};
use futures::{Stream, StreamExt};
use std::fmt::Debug;

/// Extension trait providing the [`tap`](Self::tap) operator.
///
/// This trait is implemented for all streams of [`StreamItem<T>`] where `T` implements [`Fluxion`].
pub trait TapExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    /// Invokes a side-effect function for each value without modifying the stream.
    ///
    /// This operator is useful for debugging, logging, or metrics collection
    /// without affecting the stream's data flow.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that receives a reference to each value's inner type.
    ///   Called for side effects only; return value is ignored.
    ///
    /// # Behavior
    ///
    /// - **Values**: Function `f` is called with a reference to the inner value,
    ///   then the value passes through unchanged
    /// - **Errors**: Pass through without calling `f`
    /// - **Timestamps**: Preserved unchanged
    /// - **Ordering**: Maintained
    ///
    /// # Examples
    ///
    /// ## Debugging with println
    ///
    /// ```rust
    /// use fluxion_stream::{TapExt, IntoFluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// let (tx, rx) = futures::channel::mpsc::unbounded();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// let mut tapped = stream.tap(|x| println!("Value: {}", x));
    ///
    /// tx.unbounded_send(Sequenced::new(42)).unwrap();
    /// let result = tapped.next().await.unwrap().unwrap();
    /// assert_eq!(result.into_inner(), 42);
    /// # }
    /// ```
    ///
    /// ## Collecting metrics
    ///
    /// ```rust
    /// use fluxion_stream::{TapExt, IntoFluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use std::sync::atomic::{AtomicUsize, Ordering};
    /// use std::sync::Arc;
    ///
    /// # async fn example() {
    /// let counter = Arc::new(AtomicUsize::new(0));
    /// let counter_clone = counter.clone();
    ///
    /// let (tx, rx) = futures::channel::mpsc::unbounded::<Sequenced<i32>>();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// let tapped = stream.tap(move |_| {
    ///     counter_clone.fetch_add(1, Ordering::Relaxed);
    /// });
    /// # }
    /// ```
    ///
    /// # See Also
    ///
    /// - [`MapOrderedExt::map_ordered`](crate::MapOrderedExt::map_ordered) - Transform values
    /// - [`FilterOrderedExt::filter_ordered`](crate::FilterOrderedExt::filter_ordered) - Filter values
    /// - [`OnErrorExt::on_error`](crate::OnErrorExt::on_error) - Handle errors with side effects
    fn tap<F>(self, f: F) -> impl Stream<Item = StreamItem<T>> + Send + Sync
    where
        Self: Send + Sync + Unpin + 'static,
        F: FnMut(&T::Inner) + Send + Sync + 'static;
}

impl<S, T> TapExt<T> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    fn tap<F>(self, mut f: F) -> impl Stream<Item = StreamItem<T>> + Send + Sync
    where
        Self: Send + Sync + Unpin + 'static,
        F: FnMut(&T::Inner) + Send + Sync + 'static,
    {
        self.map(move |item| {
            if let StreamItem::Value(value) = &item {
                f(&value.clone().into_inner());
            }
            item
        })
    }
}
