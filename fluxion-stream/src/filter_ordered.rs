// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::fmt::Debug;
use fluxion_core::{Fluxion, StreamItem};
use futures::future::ready;
use futures::Stream;
use futures::StreamExt;

/// Extension trait providing the `filter_ordered` operator for streams.
///
/// This trait allows any stream of `StreamItem<T>` to filter items while
/// preserving temporal ordering semantics.
pub trait FilterOrderedExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
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
    /// # Returns
    ///
    /// A stream of `StreamItem<T>` containing only items that passed the filter
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_stream::{FilterOrderedExt, IntoFluxionStream};
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
    /// # Errors
    ///
    /// This operator does not produce errors under normal operation. The predicate function receives
    /// unwrapped values, and any errors from upstream are passed through unchanged.
    ///
    /// # See Also
    ///
    /// - [`MapOrderedExt::map_ordered`](crate::MapOrderedExt::map_ordered) - Transform items
    /// - [`EmitWhenExt::emit_when`](crate::EmitWhenExt::emit_when) - Filter based on secondary stream
    /// - [`TakeWhileExt::take_while_with`](crate::TakeWhileExt::take_while_with) - Filter until condition fails
    fn filter_ordered<F>(self, predicate: F) -> impl Stream<Item = StreamItem<T>> + Send + Sync
    where
        Self: Send + Sync + Unpin + 'static,
        F: FnMut(&T::Inner) -> bool + Send + Sync + 'static;
}

impl<S, T> FilterOrderedExt<T> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    fn filter_ordered<F>(self, mut predicate: F) -> impl Stream<Item = StreamItem<T>> + Send + Sync
    where
        Self: Send + Sync + Unpin + 'static,
        F: FnMut(&T::Inner) -> bool + Send + Sync + 'static,
    {
        self.filter_map(move |item| {
            ready(match item {
                StreamItem::Value(value) if predicate(&value.clone().into_inner()) => {
                    Some(StreamItem::Value(value))
                }
                StreamItem::Value(_) => None,
                StreamItem::Error(e) => Some(StreamItem::Error(e)),
            })
        })
    }
}
