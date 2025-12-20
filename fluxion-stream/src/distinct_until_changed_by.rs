// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{Fluxion, StreamItem};
use futures::stream::StreamExt;
use futures::Stream;
use parking_lot::Mutex;
use std::fmt::Debug;
use std::sync::Arc;

/// Extension trait providing the `distinct_until_changed_by` operator for streams.
///
/// This operator filters out consecutive duplicate values using a custom comparison
/// function, emitting only when the value changes from the previous emission according
/// to the provided comparer.
pub trait DistinctUntilChangedByExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
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
    /// # Type Parameters
    ///
    /// * `F` - The comparison function type
    ///
    /// # Examples
    ///
    /// ## Custom Equality by Field
    ///
    /// ```rust
    /// use fluxion_stream::{DistinctUntilChangedByExt, IntoFluxionStream};
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
    /// let (tx, rx) = futures::channel::mpsc::unbounded();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// // Only care about changes to user ID, ignore name changes
    /// let mut distinct = stream.distinct_until_changed_by(|a: &User, b: &User| a.id == b.id);
    ///
    /// tx.unbounded_send(Sequenced::new(User { id: 1, name: "Alice".into() })).unwrap();
    /// tx.unbounded_send(Sequenced::new(User { id: 1, name: "Alice Updated".into() })).unwrap(); // Filtered
    /// tx.unbounded_send(Sequenced::new(User { id: 2, name: "Bob".into() })).unwrap(); // Emitted (ID changed)
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
    /// use fluxion_stream::{DistinctUntilChangedByExt, IntoFluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// let (tx, rx) = futures::channel::mpsc::unbounded();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// // Case-insensitive comparison
    /// let mut distinct = stream.distinct_until_changed_by(|a: &String, b: &String| {
    ///     a.to_lowercase() == b.to_lowercase()
    /// });
    ///
    /// tx.unbounded_send(Sequenced::new("hello".to_string())).unwrap();
    /// tx.unbounded_send(Sequenced::new("HELLO".to_string())).unwrap(); // Filtered (same ignoring case)
    /// tx.unbounded_send(Sequenced::new("world".to_string())).unwrap(); // Emitted
    /// tx.unbounded_send(Sequenced::new("World".to_string())).unwrap(); // Filtered
    ///
    /// assert_eq!(distinct.next().await.unwrap().unwrap().into_inner(), "hello");
    /// assert_eq!(distinct.next().await.unwrap().unwrap().into_inner(), "world");
    /// # }
    /// ```
    ///
    /// ## Approximate Numerical Comparison
    ///
    /// ```rust
    /// use fluxion_stream::{DistinctUntilChangedByExt, IntoFluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    /// use std::cmp::Ordering;
    ///
    /// // Wrapper to implement Ord for f64 (for testing purposes)
    /// #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
    /// struct OrderedF64(f64);
    ///
    /// impl Eq for OrderedF64 {}
    ///
    /// impl Ord for OrderedF64 {
    ///     fn cmp(&self, other: &Self) -> Ordering {
    ///         self.0.partial_cmp(&other.0).unwrap_or(Ordering::Equal)
    ///     }
    /// }
    ///
    /// # async fn example() {
    /// let (tx, rx) = futures::channel::mpsc::unbounded();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// // Only emit if difference is >= 0.1
    /// let mut distinct = stream.distinct_until_changed_by(|a: &OrderedF64, b: &OrderedF64| {
    ///     (a.0 - b.0).abs() < 0.1
    /// });
    ///
    /// tx.unbounded_send(Sequenced::new(OrderedF64(1.0))).unwrap();
    /// tx.unbounded_send(Sequenced::new(OrderedF64(1.05))).unwrap();  // Filtered (diff < 0.1)
    /// tx.unbounded_send(Sequenced::new(OrderedF64(1.15))).unwrap();  // Emitted (diff >= 0.1)
    /// tx.unbounded_send(Sequenced::new(OrderedF64(1.18))).unwrap();  // Filtered
    /// tx.unbounded_send(Sequenced::new(OrderedF64(1.30))).unwrap();  // Emitted
    ///
    /// assert_eq!(distinct.next().await.unwrap().unwrap().into_inner().0, 1.0);
    /// assert_eq!(distinct.next().await.unwrap().unwrap().into_inner().0, 1.15);
    /// assert_eq!(distinct.next().await.unwrap().unwrap().into_inner().0, 1.30);
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
    /// # Performance
    ///
    /// - O(1) time complexity per item (plus the cost of the comparer function)
    /// - Stores only the last emitted value
    /// - No buffering or lookahead required
    ///
    /// # See Also
    ///
    /// - [`distinct_until_changed`](crate::DistinctUntilChangedExt::distinct_until_changed) - Uses `PartialEq` for comparison
    /// - [`filter_ordered`](crate::FilterOrderedExt::filter_ordered) - General filtering
    fn distinct_until_changed_by<F>(
        self,
        compare: F,
    ) -> impl Stream<Item = StreamItem<T>> + Send + Sync
    where
        F: Fn(&T::Inner, &T::Inner) -> bool + Send + Sync + 'static;
}

impl<T, S> DistinctUntilChangedByExt<T> for S
where
    S: Stream<Item = StreamItem<T>> + Send + Sync + 'static,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    fn distinct_until_changed_by<F>(
        self,
        compare: F,
    ) -> impl Stream<Item = StreamItem<T>> + Send + Sync
    where
        F: Fn(&T::Inner, &T::Inner) -> bool + Send + Sync + 'static,
    {
        let last_value: Arc<Mutex<Option<T::Inner>>> = Arc::new(Mutex::new(None));
        let compare = Arc::new(compare);

        let stream = self.filter_map(move |item| {
            let last_value = Arc::clone(&last_value);
            let compare = Arc::clone(&compare);

            async move {
                match item {
                    StreamItem::Value(value) => {
                        let current_inner = value.clone().into_inner();

                        let mut last = last_value.lock();

                        // Check if this value is different from the last emitted value
                        let should_emit = match last.as_ref() {
                            None => true, // First value, always emit
                            Some(prev) => !compare(&current_inner, prev),
                        };

                        if should_emit {
                            // Update last value
                            *last = Some(current_inner);

                            // Preserve original timestamp
                            Some(StreamItem::Value(value))
                        } else {
                            None // Filter out duplicate
                        }
                    }
                    StreamItem::Error(e) => Some(StreamItem::Error(e)), // Propagate errors
                }
            }
        });

        Box::pin(stream)
    }
}
