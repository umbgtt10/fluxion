// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Map transformation operator that preserves temporal ordering.

use core::fmt::Debug;
use fluxion_core::{Fluxion, StreamItem};
use futures::{Stream, StreamExt};

/// Extension trait providing the `map_ordered` operator for streams.
///
/// This trait allows any stream of `StreamItem<T>` to transform items while
/// preserving temporal ordering semantics.
pub trait MapOrderedExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    /// Maps each item to a new value while preserving temporal ordering.
    ///
    /// This operator transforms each item in the stream using the provided function.
    /// Unlike the standard `map` operator from [`StreamExt`], `map_ordered` is
    /// designed for Fluxion streams and preserves the `StreamItem` wrapper semantics.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that transforms each item of type `T` into type `U`
    ///
    /// # Behavior
    ///
    /// - Values are transformed using the provided function
    /// - Errors are passed through unchanged
    /// - Timestamps are preserved/transformed according to the output type
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_stream::{MapOrderedExt, IntoFluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// let (tx, rx) = futures::channel::mpsc::unbounded();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// let mut mapped = stream.map_ordered(|x: Sequenced<i32>| Sequenced::new(x.into_inner() * 2));
    ///
    /// tx.unbounded_send(Sequenced::new(5)).unwrap();
    /// assert_eq!(mapped.next().await.unwrap().unwrap().into_inner(), 10);
    /// # }
    /// ```
    ///
    /// # See Also
    ///
    /// - [`FilterOrderedExt::filter_ordered`](crate::FilterOrderedExt::filter_ordered) - Filter items
    /// - [`ScanOrderedExt::scan_ordered`](crate::ScanOrderedExt::scan_ordered) - Stateful transformation
    ///
    /// [`StreamExt`]: futures::StreamExt
    fn map_ordered<U, F>(self, f: F) -> impl Stream<Item = StreamItem<U>> + Send + Sync
    where
        Self: Send + Sync + Unpin + 'static,
        U: Fluxion,
        U::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
        U::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
        F: FnMut(T) -> U + Send + Sync + 'static;
}

impl<S, T> MapOrderedExt<T> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    fn map_ordered<U, F>(self, mut f: F) -> impl Stream<Item = StreamItem<U>> + Send + Sync
    where
        Self: Send + Sync + Unpin + 'static,
        U: Fluxion,
        U::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
        U::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
        F: FnMut(T) -> U + Send + Sync + 'static,
    {
        self.map(move |item| item.map(&mut f))
    }
}
