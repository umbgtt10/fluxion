// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Distinct-until-changed-by operator for single-threaded runtimes.

use core::fmt::Debug;
use fluxion_core::{Fluxion, StreamItem};
use fluxion_stream_core::distinct_until_changed_by::distinct_until_changed_by_impl;
use futures::Stream;

/// Extension trait providing the `distinct_until_changed_by` operator for streams.
///
/// This operator filters out consecutive duplicate values using a custom comparison
/// function, emitting only when the value changes from the previous emission according
/// to the provided comparer.
pub trait DistinctUntilChangedByExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
{
    /// Emits values only when they differ from the previous emitted value according
    /// to a custom comparison function.
    ///
    /// This operator maintains the last emitted inner value and compares each new
    /// value against it using the provided comparison function. Only values where
    /// the comparer returns `false` are emitted. The timestamps from the original
    /// values are preserved.
    ///
    /// # Behavior
    ///
    /// - First value is always emitted (no previous value to compare)
    /// - Subsequent values are compared to the last emitted value using `compare`
    /// - Only values where `compare(current, previous) == false` are emitted
    /// - The comparer should return `true` if values are considered equal/same
    /// - Timestamps are preserved from the original incoming values
    /// - Errors are always propagated immediately
    ///
    /// # Arguments
    ///
    /// * `compare` - A function that takes two references to `T::Inner` and returns
    ///   `true` if they should be considered equal (and thus filtered), or `false`
    ///   if they are different (and should be emitted).
    ///
    /// # Examples
    ///
    /// ## Custom Equality by Field
    ///
    /// ```rust
    /// use fluxion_stream_multi::{DistinctUntilChangedByExt, IntoFluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    ///
    /// #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    /// struct User {
    ///     id: u32,
    ///     name: String,
    /// }
    ///
    /// # async fn example() {
    /// let (tx, rx) = async_channel::unbounded();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// // Only care about changes to user ID, ignore name changes
    /// let mut distinct = stream.distinct_until_changed_by(|a: &User, b: &User| a.id == b.id);
    ///
    /// tx.try_send(Sequenced::new(User { id: 1, name: "Alice".into() })).unwrap();
    /// tx.try_send(Sequenced::new(User { id: 1, name: "Alice Updated".into() })).unwrap(); // Filtered
    /// tx.try_send(Sequenced::new(User { id: 2, name: "Bob".into() })).unwrap(); // Emitted (ID changed)
    ///
    /// let first = distinct.next().await.unwrap().unwrap();
    /// assert_eq!(first.into_inner().id, 1);
    ///
    /// let second = distinct.next().await.unwrap().unwrap();
    /// assert_eq!(second.into_inner().id, 2);
    /// # }
    /// ```
    ///
    /// ## Case-Insensitive String Comparison
    ///
    /// ```rust
    /// use fluxion_stream_multi::{DistinctUntilChangedByExt, IntoFluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// let (tx, rx) = async_channel::unbounded();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// // Case-insensitive comparison
    /// let mut distinct = stream.distinct_until_changed_by(|a: &String, b: &String| {
    ///     a.to_lowercase() == b.to_lowercase()
    /// });
    ///
    /// tx.try_send(Sequenced::new("hello".to_string())).unwrap();
    /// tx.try_send(Sequenced::new("HELLO".to_string())).unwrap(); // Filtered (same ignoring case)
    /// tx.try_send(Sequenced::new("world".to_string())).unwrap(); // Emitted
    /// tx.try_send(Sequenced::new("World".to_string())).unwrap(); // Filtered
    ///
    /// assert_eq!(distinct.next().await.unwrap().unwrap().into_inner(), "hello");
    /// assert_eq!(distinct.next().await.unwrap().unwrap().into_inner(), "world");
    /// # }
    /// ```
    ///
    /// # Use Cases
    ///
    /// - Comparing complex types by specific fields
    /// - Case-insensitive or fuzzy comparisons
    /// - Threshold-based filtering (e.g., temperature within range)
    /// - Custom domain-specific equality logic
    /// - Working with types that don't implement `PartialEq`
    ///
    /// # See Also
    ///
    /// - [`distinct_until_changed`](crate::DistinctUntilChangedExt::distinct_until_changed) - Uses `PartialEq` for comparison
    fn distinct_until_changed_by<F>(self, compare: F) -> impl Stream<Item = StreamItem<T>>
    where
        F: Fn(&T::Inner, &T::Inner) -> bool + 'static;
}

impl<T, S> DistinctUntilChangedByExt<T> for S
where
    S: Stream<Item = StreamItem<T>> + 'static,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
{
    fn distinct_until_changed_by<F>(self, compare: F) -> impl Stream<Item = StreamItem<T>>
    where
        F: Fn(&T::Inner, &T::Inner) -> bool + 'static,
    {
        distinct_until_changed_by_impl(self, compare)
    }
}
