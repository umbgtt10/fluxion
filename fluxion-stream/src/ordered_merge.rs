// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use futures::Stream;
use std::fmt::Debug;
use std::pin::Pin;

use crate::Ordered;
use fluxion_core::{into_stream::IntoStream, StreamItem};

// Re-export low-level types from fluxion-ordered-merge
pub use fluxion_ordered_merge::{OrderedMerge, OrderedMergeExt};

/// Extension trait providing high-level ordered merge for `Ordered` streams.
///
/// This trait merges multiple streams of ordered items, emitting all values from
/// all streams in temporal order. Unlike `combine_latest`, this emits every value
/// from every stream (not just when all have emitted).
pub trait OrderedStreamExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Clone + Debug + Ordered + Ord + Send + Sync + Unpin + 'static,
{
    /// Merges multiple ordered streams, emitting all values in temporal order.
    ///
    /// This operator takes multiple streams and merges them into a single stream where
    /// all values are emitted in order based on their `Ordered::order()` value. Every
    /// value from every stream is emitted exactly once.
    ///
    /// # Behavior
    ///
    /// - Emits **all** values from all streams (unlike `combine_latest`)
    /// - Values are ordered by their `Ordered::order()` timestamp
    /// - Does not wait for all streams to emit before starting
    /// - Continues until all input streams are exhausted
    ///
    /// # Arguments
    ///
    /// * `others` - Vector of additional streams to merge with this stream
    ///
    /// # Returns
    ///
    /// A stream of `T` where all values from all input streams are emitted in temporal order.
    ///
    /// # Errors
    ///
    /// This operator may produce `StreamItem::Error` in the following cases:
    ///
    /// - **Internal Processing Errors**: When stream processing encounters an issue, it will emit
    ///   `FluxionError::StreamProcessingError`.
    /// - **Callback Errors**: If the `on_all_streams_closed` callback panics, the panic is caught
    ///   and wrapped in `FluxionError::UserError`.
    ///
    /// These errors typically indicate abnormal stream termination. See the [Error Handling Guide](../docs/ERROR-HANDLING.md)
    /// for patterns on handling these errors in your application.
    ///
    /// # See Also
    ///
    /// - [`combine_latest`](crate::CombineLatestExt::combine_latest) - Emits latest values when any stream emits
    /// - [`with_latest_from`](crate::WithLatestFromExt::with_latest_from) - Samples secondary on primary emission
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_stream::{OrderedStreamExt, FluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    /// use fluxion_core::Ordered;
    ///
    /// # async fn example() {
    /// // Create channels
    /// let (tx1, rx1) = tokio::sync::mpsc::unbounded_channel();
    /// let (tx2, rx2) = tokio::sync::mpsc::unbounded_channel();
    ///
    /// // Create streams
    /// let stream1 = FluxionStream::from_unbounded_receiver(rx1);
    /// let stream2 = FluxionStream::from_unbounded_receiver(rx2);
    ///
    /// // Merge streams
    /// let mut merged = stream1.ordered_merge(vec![stream2]);
    ///
    /// // Send values with explicit ordering
    /// tx1.send(Sequenced::with_sequence(1, 100)).unwrap();
    /// tx2.send(Sequenced::with_sequence(2, 200)).unwrap();
    /// tx1.send(Sequenced::with_sequence(3, 300)).unwrap();
    ///
    /// // Assert - values emitted in temporal order
    /// assert_eq!(merged.next().await.unwrap().unwrap().value, 1);
    /// assert_eq!(merged.next().await.unwrap().unwrap().value, 2);
    /// assert_eq!(merged.next().await.unwrap().unwrap().value, 3);
    /// # }
    /// ```
    ///
    /// # Use Cases
    ///
    /// - Merging event streams from multiple sources
    /// - Combining time-series data while preserving temporal order
    /// - Fan-in pattern where all events must be processed in order
    ///
    /// # Comparison with `combine_latest`
    ///
    /// - `ordered_merge`: Emits all values from all streams
    /// - `combine_latest`: Emits only when streams change, after all have initialized
    fn ordered_merge<IS>(self, others: Vec<IS>) -> impl Stream<Item = StreamItem<T>> + Send + Sync
    where
        IS: IntoStream<Item = StreamItem<T>>,
        IS::Stream: Send + Sync + 'static;
}

impl<T, S> OrderedStreamExt<T> for S
where
    T: Clone + Debug + Ordered + Ord + Send + Sync + Unpin + 'static,
    S: Stream<Item = StreamItem<T>> + Send + Sync + 'static,
{
    fn ordered_merge<IS>(self, others: Vec<IS>) -> impl Stream<Item = StreamItem<T>> + Send + Sync
    where
        IS: IntoStream<Item = StreamItem<T>>,
        IS::Stream: Send + Sync + 'static,
    {
        let mut all_streams =
            vec![Box::pin(self) as Pin<Box<dyn Stream<Item = StreamItem<T>> + Send + Sync>>];
        for into_stream in others {
            let stream = into_stream.into_stream();
            all_streams.push(Box::pin(stream));
        }

        OrderedMerge::new(all_streams)
    }
}
