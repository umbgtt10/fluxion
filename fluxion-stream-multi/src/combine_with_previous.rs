// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Combine-with-previous operator for single-threaded runtimes.

use core::fmt::Debug;
use fluxion_core::{Fluxion, StreamItem};
use fluxion_stream_core::combine_with_previous::{combine_with_previous_impl, WithPrevious};
use futures::Stream;

/// Extension trait providing the `combine_with_previous` operator for timestamped streams.
///
/// This operator pairs each stream element with its predecessor, enabling
/// stateful processing and change detection.
pub trait CombineWithPreviousExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
{
    /// Pairs each stream element with its previous element.
    ///
    /// This operator transforms a stream of `T` into a stream of `WithPrevious<T>`,
    /// where each item contains both the current value and the previous value (if any).
    /// The first element will have `previous = None`.
    ///
    /// # Behavior
    ///
    /// - First element: `WithPrevious { previous: None, current: first_value }`
    /// - Subsequent elements: `WithPrevious { previous: Some(prev), current: curr }`
    /// - Maintains state to track the previous value
    /// - Preserves temporal ordering from the source stream
    ///
    /// # Returns
    ///
    /// A stream of `WithPrevious<T>` where each item contains the current
    /// and previous values.
    ///
    /// # Error Handling
    ///
    /// Errors from the source stream are propagated unchanged.
    ///
    /// # See Also
    ///
    /// - [`combine_latest`](crate::CombineLatestExt::combine_latest) - Combines multiple streams
    /// - Useful for change detection and delta calculations
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_stream_single::CombineWithPreviousExt;
    /// use fluxion_test_utils::Sequenced;
    /// use fluxion_stream_single::IntoFluxionStream;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// let (tx, rx) = async_channel::unbounded();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// let mut paired = stream.combine_with_previous();
    ///
    /// tx.try_send(Sequenced::new(1)).unwrap();
    /// tx.try_send(Sequenced::new(2)).unwrap();
    ///
    /// // First has no previous
    /// let first = paired.next().await.unwrap().unwrap();
    /// assert_eq!(first.previous, None);
    /// assert_eq!(first.current.into_inner(), 1);
    ///
    /// // Second has previous
    /// let second = paired.next().await.unwrap().unwrap();
    /// assert_eq!(second.previous.as_ref().unwrap().into_inner(), 1);
    /// assert_eq!(second.current.into_inner(), 2);
    /// # }
    /// ```
    ///
    /// # Use Cases
    ///
    /// - Change detection (comparing consecutive values)
    /// - Delta calculation (computing differences)
    /// - State transitions (analyzing previous → current)
    /// - Duplicate filtering (skip if same as previous)
    fn combine_with_previous(self) -> impl Stream<Item = StreamItem<WithPrevious<T>>>;
}

impl<T, S> CombineWithPreviousExt<T> for S
where
    S: Stream<Item = StreamItem<T>> + Sized + 'static,
    T: Fluxion,
    T::Inner: Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
{
    fn combine_with_previous(self) -> impl Stream<Item = StreamItem<WithPrevious<T>>> {
        combine_with_previous_impl(self)
    }
}
