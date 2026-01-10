// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Filter-ordered operator for single-threaded runtimes.

use core::fmt::Debug;
use fluxion_core::{Fluxion, StreamItem};
use fluxion_stream_core::filter_ordered::filter_ordered_impl;
use futures::Stream;

/// Extension trait providing the `filter_ordered` operator for streams.
///
/// This trait allows any stream of `StreamItem<T>` to filter items while
/// preserving temporal ordering semantics.
pub trait FilterOrderedExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
{
    /// Filters items based on a predicate while preserving temporal ordering.
    ///
    /// This operator selectively emits items from the stream based on a predicate function
    /// that evaluates the inner value. Items that satisfy the predicate are emitted, while
    /// others are filtered out. Temporal ordering is maintained across filtered items.
    ///
    /// # Behavior
    ///
    /// - Predicate receives a reference to the **inner value** (`&T::Inner`), not the full wrapper
    /// - Only items where predicate returns `true` are emitted
    /// - Filtered items preserve their original temporal order
    /// - Errors are passed through unchanged
    ///
    /// # Arguments
    ///
    /// * `predicate` - A function that takes `&T::Inner` and returns `true` to keep the item
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_stream_single::{FilterOrderedExt, IntoFluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// let (tx, rx) = async_channel::unbounded();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// // Filter for even numbers
    /// let mut evens = stream.filter_ordered(|&n| n % 2 == 0);
    ///
    /// tx.try_send(Sequenced::new(1)).unwrap();
    /// tx.try_send(Sequenced::new(2)).unwrap();
    /// tx.try_send(Sequenced::new(3)).unwrap();
    /// tx.try_send(Sequenced::new(4)).unwrap();
    ///
    /// assert_eq!(evens.next().await.unwrap().unwrap().into_inner(), 2);
    /// assert_eq!(evens.next().await.unwrap().unwrap().into_inner(), 4);
    /// # }
    /// ```
    ///
    /// # Use Cases
    ///
    /// - **Threshold Filtering**: Remove values outside acceptable ranges
    /// - **Type-Based Filtering**: Select specific enum variants
    /// - **Conditional Processing**: Process only relevant events
    /// - **Noise Reduction**: Remove unwanted data from streams
    ///
    /// # See Also
    ///
    /// - [`map_ordered`] - Transform items (if available)
    /// - [`EmitWhenExt::emit_when`](crate::EmitWhenExt::emit_when) - Filter based on secondary stream
    /// - [`TakeWhileExt::take_while_with`](crate::TakeWhileExt::take_while_with) - Filter until condition fails
    fn filter_ordered<F>(self, predicate: F) -> impl Stream<Item = StreamItem<T>>
    where
        Self: Unpin + 'static,
        F: FnMut(&T::Inner) -> bool + 'static;
}

impl<S, T> FilterOrderedExt<T> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
{
    fn filter_ordered<F>(self, predicate: F) -> impl Stream<Item = StreamItem<T>>
    where
        Self: Unpin + 'static,
        F: FnMut(&T::Inner) -> bool + 'static,
    {
        filter_ordered_impl(self, predicate)
    }
}
