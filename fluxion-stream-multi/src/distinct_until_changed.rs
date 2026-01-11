// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Distinct-until-changed operator for single-threaded runtimes.

use core::fmt::Debug;
use fluxion_core::{Fluxion, StreamItem};
use fluxion_stream_core::distinct_until_changed::distinct_until_changed_impl;
use futures::Stream;

/// Extension trait providing the `distinct_until_changed` operator for streams.
///
/// This operator filters out consecutive duplicate values, emitting only when
/// the value changes from the previous emission.
pub trait DistinctUntilChangedExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
{
    /// Emits values only when they differ from the previous emitted value.
    ///
    /// This operator maintains the last emitted inner value and compares each new
    /// value against it. Only values that differ are emitted. The comparison is
    /// performed on the inner values (`T::Inner`), and timestamps from the original
    /// values are preserved.
    ///
    /// # Behavior
    ///
    /// - First value is always emitted (no previous value to compare)
    /// - Subsequent values are compared to the last emitted value
    /// - Only values where `current != previous` are emitted
    /// - Timestamps are preserved from the original incoming values
    /// - Errors are always propagated immediately
    ///
    /// # Examples
    ///
    /// ## Basic Deduplication
    ///
    /// ```rust
    /// use fluxion_stream_multi::{DistinctUntilChangedExt, IntoFluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// let (tx, rx) = async_channel::unbounded();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// let mut distinct = stream.distinct_until_changed();
    ///
    /// // Send: 1, 1, 2, 2, 2, 3, 2
    /// tx.try_send(Sequenced::new(1)).unwrap();
    /// tx.try_send(Sequenced::new(1)).unwrap(); // Filtered
    /// tx.try_send(Sequenced::new(2)).unwrap();
    /// tx.try_send(Sequenced::new(2)).unwrap(); // Filtered
    /// tx.try_send(Sequenced::new(2)).unwrap(); // Filtered
    /// tx.try_send(Sequenced::new(3)).unwrap();
    /// tx.try_send(Sequenced::new(2)).unwrap(); // Emitted (different from 3)
    ///
    /// // Output: 1, 2, 3, 2
    /// assert_eq!(distinct.next().await.unwrap().unwrap().into_inner(), 1);
    /// assert_eq!(distinct.next().await.unwrap().unwrap().into_inner(), 2);
    /// assert_eq!(distinct.next().await.unwrap().unwrap().into_inner(), 3);
    /// assert_eq!(distinct.next().await.unwrap().unwrap().into_inner(), 2);
    /// # }
    /// ```
    ///
    /// ## Toggle/State Change Detection
    ///
    /// Detect when a boolean state changes:
    ///
    /// ```rust
    /// use fluxion_stream_multi::{DistinctUntilChangedExt, IntoFluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// let (tx, rx) = async_channel::unbounded();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// let mut changes = stream.distinct_until_changed();
    ///
    /// // Simulate toggle events
    /// tx.try_send(Sequenced::new(false)).unwrap(); // Initial state
    /// tx.try_send(Sequenced::new(false)).unwrap(); // No change
    /// tx.try_send(Sequenced::new(true)).unwrap();  // Changed!
    /// tx.try_send(Sequenced::new(true)).unwrap();  // No change
    /// tx.try_send(Sequenced::new(false)).unwrap(); // Changed!
    ///
    /// // Only state transitions are emitted
    /// assert!(!changes.next().await.unwrap().unwrap().into_inner());
    /// assert!(changes.next().await.unwrap().unwrap().into_inner());
    /// assert!(!changes.next().await.unwrap().unwrap().into_inner());
    /// # }
    /// ```
    ///
    /// # Use Cases
    ///
    /// - Debouncing repeated sensor readings
    /// - Detecting state transitions (on/off, connected/disconnected)
    /// - Filtering redundant UI updates
    /// - Change detection in data streams
    /// - Rate limiting when values don't change
    ///
    /// # See Also
    ///
    /// - [`filter_ordered`](crate::FilterOrderedExt::filter_ordered) - General filtering
    /// - [`take_while_with`](crate::TakeWhileExt::take_while_with) - Conditional stream termination
    fn distinct_until_changed(self) -> impl Stream<Item = StreamItem<T>>;
}

impl<T, S> DistinctUntilChangedExt<T> for S
where
    S: Stream<Item = StreamItem<T>> + 'static,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
{
    fn distinct_until_changed(self) -> impl Stream<Item = StreamItem<T>> {
        distinct_until_changed_impl(self)
    }
}
