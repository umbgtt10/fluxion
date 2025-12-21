// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::fmt::Debug;
use fluxion_core::{Fluxion, StreamItem};
use futures::stream::StreamExt;
use futures::Stream;
use parking_lot::Mutex;
use std::sync::Arc;

/// Extension trait providing the `distinct_until_changed` operator for streams.
///
/// This operator filters out consecutive duplicate values, emitting only when
/// the value changes from the previous emission.
pub trait DistinctUntilChangedExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
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
    /// # Type Parameters
    ///
    /// * `IS` - Type that can be converted into a stream compatible with this stream
    ///
    /// # Examples
    ///
    /// ## Basic Deduplication
    ///
    /// ```rust
    /// use fluxion_stream::{DistinctUntilChangedExt, IntoFluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// let (tx, rx) = futures::channel::mpsc::unbounded();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// let mut distinct = stream.distinct_until_changed();
    ///
    /// // Send: 1, 1, 2, 2, 2, 3, 2
    /// tx.unbounded_send(Sequenced::new(1)).unwrap();
    /// tx.unbounded_send(Sequenced::new(1)).unwrap(); // Filtered
    /// tx.unbounded_send(Sequenced::new(2)).unwrap();
    /// tx.unbounded_send(Sequenced::new(2)).unwrap(); // Filtered
    /// tx.unbounded_send(Sequenced::new(2)).unwrap(); // Filtered
    /// tx.unbounded_send(Sequenced::new(3)).unwrap();
    /// tx.unbounded_send(Sequenced::new(2)).unwrap(); // Emitted (different from 3)
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
    /// use fluxion_stream::{DistinctUntilChangedExt, IntoFluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// let (tx, rx) = futures::channel::mpsc::unbounded();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// let mut changes = stream.distinct_until_changed();
    ///
    /// // Simulate toggle events
    /// tx.unbounded_send(Sequenced::new(false)).unwrap(); // Initial state
    /// tx.unbounded_send(Sequenced::new(false)).unwrap(); // No change
    /// tx.unbounded_send(Sequenced::new(true)).unwrap();  // Changed!
    /// tx.unbounded_send(Sequenced::new(true)).unwrap();  // No change
    /// tx.unbounded_send(Sequenced::new(false)).unwrap(); // Changed!
    ///
    /// // Only state transitions are emitted
    /// assert!(!changes.next().await.unwrap().unwrap().into_inner());
    /// assert!(changes.next().await.unwrap().unwrap().into_inner());
    /// assert!(!changes.next().await.unwrap().unwrap().into_inner());
    /// # }
    /// ```
    ///
    /// ## Timestamp Preservation
    ///
    /// Timestamps are preserved from the original incoming values:
    ///
    /// ```rust
    /// use fluxion_stream::{DistinctUntilChangedExt, IntoFluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use fluxion_core::HasTimestamp;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// let (tx, rx) = futures::channel::mpsc::unbounded();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// let mut distinct = stream.distinct_until_changed();
    ///
    /// tx.unbounded_send(Sequenced::new(1)).unwrap();
    /// let first = distinct.next().await.unwrap().unwrap();
    /// let ts1 = first.timestamp();
    ///
    /// tx.unbounded_send(Sequenced::new(1)).unwrap(); // Filtered (duplicate)
    /// tx.unbounded_send(Sequenced::new(2)).unwrap(); // Emitted with its original timestamp
    ///
    /// let second = distinct.next().await.unwrap().unwrap();
    /// let ts2 = second.timestamp();
    ///
    /// // Timestamps come from the original Sequenced values
    /// assert!(ts2 > ts1);
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
    /// # Performance
    ///
    /// - O(1) time complexity per item
    /// - Stores only the last emitted value
    /// - No buffering or lookahead required
    ///
    /// # See Also
    ///
    /// - [`filter_ordered`](crate::FilterOrderedExt::filter_ordered) - General filtering
    /// - [`take_while_with`](crate::TakeWhileExt::take_while_with) - Conditional stream termination
    fn distinct_until_changed(self) -> impl Stream<Item = StreamItem<T>> + Send + Sync;
}

impl<T, S> DistinctUntilChangedExt<T> for S
where
    S: Stream<Item = StreamItem<T>> + Send + Sync + 'static,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    fn distinct_until_changed(self) -> impl Stream<Item = StreamItem<T>> + Send + Sync {
        let last_value: Arc<Mutex<Option<T::Inner>>> = Arc::new(Mutex::new(None));

        let stream = self.filter_map(move |item| {
            let last_value = Arc::clone(&last_value);
            async move {
                match item {
                    StreamItem::Value(value) => {
                        let current_inner = value.clone().into_inner();

                        let mut last = last_value.lock();

                        // Check if this value is different from the last emitted value
                        let should_emit = match last.as_ref() {
                            None => true, // First value, always emit
                            Some(prev) => current_inner != *prev,
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
