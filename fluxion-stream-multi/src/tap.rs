// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Tap operator for single-threaded runtimes.

use core::fmt::Debug;
use fluxion_core::{Fluxion, StreamItem};
use fluxion_stream_core::tap::tap_impl;
use futures::Stream;

/// Extension trait providing the `tap` operator.
///
/// This trait is implemented for all streams of `StreamItem<T>` where `T` implements `Fluxion`.
pub trait TapExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
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
    /// use fluxion_stream_multi::{TapExt, IntoFluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// let (tx, rx) = async_channel::unbounded();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// let mut tapped = stream.tap(|x| println!("Value: {}", x));
    ///
    /// tx.try_send(Sequenced::new(42)).unwrap();
    /// let result = tapped.next().await.unwrap().unwrap();
    /// assert_eq!(result.into_inner(), 42);
    /// # }
    /// ```
    ///
    /// ## Collecting metrics
    ///
    /// ```rust
    /// use fluxion_stream_multi::{TapExt, IntoFluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use std::sync::Arc;
    /// use core::sync::atomic::{AtomicUsize, Ordering};
    ///
    /// # async fn example() {
    /// let counter = Arc::new(AtomicUsize::new(0));
    /// let counter_clone = counter.clone();
    ///
    /// let (tx, rx) = async_channel::unbounded::<Sequenced<i32>>();
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
    fn tap<F>(self, f: F) -> impl Stream<Item = StreamItem<T>>
    where
        Self: Unpin + 'static,
        F: FnMut(&T::Inner) + 'static;
}

impl<S, T> TapExt<T> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
{
    fn tap<F>(self, f: F) -> impl Stream<Item = StreamItem<T>>
    where
        Self: Unpin + 'static,
        F: FnMut(&T::Inner) + 'static,
    {
        tap_impl(self, f)
    }
}
