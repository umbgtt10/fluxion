// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Debug;
use core::pin::Pin;
use fluxion_core::{into_stream::IntoStream, Fluxion, StreamItem};
use futures::task::{Context, Poll};
use futures::{Stream, StreamExt};

/// Extension trait providing high-level ordered merge for `Timestamped` streams.
///
/// This trait merges multiple streams of timestamped items, emitting all values from
/// all streams in temporal order. Unlike `combine_latest`, this emits every value
/// from every stream (not just when all have emitted).
#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
))]
pub trait OrderedStreamExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    /// Merges multiple timestamped streams, emitting all values in temporal order.
    ///
    /// This operator takes multiple streams and merges them into a single stream where
    /// all values are emitted in order based on their `Timestamped::timestamp()` value. Every
    /// value from every stream is emitted exactly once.
    ///
    /// # Behavior
    ///
    /// - Emits **all** values from all streams (unlike `combine_latest`)
    /// - Values are ordered by their `Timestamped::timestamp()` timestamp
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
    /// use fluxion_stream::OrderedStreamExt;
    /// use fluxion_test_utils::{Sequenced, helpers::unwrap_stream, unwrap_value, test_channel};
    /// use fluxion_core::Timestamped as TimestampedTrait;
    ///
    /// # async fn example() {
    /// // Create channels
    /// let (tx1, stream1) = test_channel::<Sequenced<i32>>();
    /// let (tx2, stream2) = test_channel::<Sequenced<i32>>();
    ///
    /// // Merge streams
    /// let mut merged = stream1.ordered_merge(vec![stream2]);
    ///
    /// // Send values with explicit ordering
    /// tx1.unbounded_send((1, 100).into()).unwrap();
    /// tx2.unbounded_send((2, 200).into()).unwrap();
    /// tx1.unbounded_send((3, 300).into()).unwrap();
    ///
    /// // Assert - values emitted in temporal order
    /// assert_eq!(unwrap_value(Some(unwrap_stream(&mut merged, 500).await)).value, 1);
    /// assert_eq!(unwrap_value(Some(unwrap_stream(&mut merged, 500).await)).value, 2);
    /// assert_eq!(unwrap_value(Some(unwrap_stream(&mut merged, 500).await)).value, 3);
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
    fn ordered_merge<IS>(self, others: Vec<IS>) -> impl Stream<Item = StreamItem<T>>
    where
        IS: IntoStream<Item = StreamItem<T>>,
        IS::Stream: Send + Sync + 'static;
}

// Single-threaded runtime version (WASM/Embassy)
#[cfg(not(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
)))]
pub trait OrderedStreamExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Debug + Ord + Unpin + 'static,
    T::Timestamp: Debug + Ord + Copy + 'static,
{
    /// Merges multiple timestamped streams, emitting all values in temporal order.
    ///
    /// (Documentation same as multi-threaded version)
    fn ordered_merge<IS>(self, others: Vec<IS>) -> impl Stream<Item = StreamItem<T>>
    where
        IS: IntoStream<Item = StreamItem<T>>,
        IS::Stream: 'static;
}

// Multi-threaded implementation
#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
))]
impl<T, S> OrderedStreamExt<T> for S
where
    T: Fluxion,
    T::Inner: Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
    S: Stream<Item = StreamItem<T>> + Send + Sync + 'static,
{
    fn ordered_merge<IS>(self, others: Vec<IS>) -> impl Stream<Item = StreamItem<T>>
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

        StreamExt::map(
            OrderedMergeWithImmediateErrorsIndexed::new(all_streams),
            |(item, _index)| item,
        )
    }
}

// Single-threaded implementation
#[cfg(not(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
)))]
impl<T, S> OrderedStreamExt<T> for S
where
    T: Fluxion,
    T::Inner: Debug + Ord + Unpin + 'static,
    T::Timestamp: Debug + Ord + Copy + 'static,
    S: Stream<Item = StreamItem<T>> + 'static,
{
    fn ordered_merge<IS>(self, others: Vec<IS>) -> impl Stream<Item = StreamItem<T>>
    where
        IS: IntoStream<Item = StreamItem<T>>,
        IS::Stream: 'static,
    {
        let mut all_streams = vec![Box::pin(self) as Pin<Box<dyn Stream<Item = StreamItem<T>>>>];
        for into_stream in others {
            let stream = into_stream.into_stream();
            all_streams.push(Box::pin(stream));
        }

        // Use the indexed version internally and discard the index
        StreamExt::map(
            OrderedMergeWithImmediateErrorsIndexed::new(all_streams),
            |(item, _index)| item,
        )
    }
}

/// Helper function to create an ordered merge stream with immediate error emission
/// and stream indexing for operators that need to track which stream emitted each item.
///
/// This is used internally by operators like `combine_latest`, `with_latest_from`, etc.
/// that need both temporal ordering of values AND immediate error propagation.
#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
))]
pub(crate) fn ordered_merge_with_index<T>(
    streams: Vec<Pin<Box<dyn Stream<Item = StreamItem<T>> + Send + Sync>>>,
) -> impl Stream<Item = (StreamItem<T>, usize)>
where
    T: Fluxion + Unpin,
    T::Inner: Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    OrderedMergeWithImmediateErrorsIndexed::new(streams)
}

#[cfg(not(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
)))]
pub(crate) fn ordered_merge_with_index<T>(
    streams: Vec<Pin<Box<dyn Stream<Item = StreamItem<T>>>>>,
) -> impl Stream<Item = (StreamItem<T>, usize)>
where
    T: Fluxion + Unpin,
    T::Inner: Debug + Ord + Unpin + 'static,
    T::Timestamp: Debug + Ord + Copy + 'static,
{
    OrderedMergeWithImmediateErrorsIndexed::new(streams)
}

/// Stream that merges multiple timestamped streams in temporal order with stream indices.
///
/// Like OrderedMergeWithImmediateErrors but also tracks which stream each item came from.
#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
))]
struct OrderedMergeWithImmediateErrorsIndexed<T>
where
    T: Fluxion,
    T::Inner: Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    streams: Vec<Pin<Box<dyn Stream<Item = StreamItem<T>> + Send + Sync>>>,
    buffered: Vec<Option<T>>,
}

#[cfg(not(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
)))]
struct OrderedMergeWithImmediateErrorsIndexed<T>
where
    T: Fluxion,
    T::Inner: Debug + Ord + Unpin + 'static,
    T::Timestamp: Debug + Ord + Copy + 'static,
{
    streams: Vec<Pin<Box<dyn Stream<Item = StreamItem<T>>>>>,
    buffered: Vec<Option<T>>,
}

#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
))]
impl<T> OrderedMergeWithImmediateErrorsIndexed<T>
where
    T: Fluxion,
    T::Inner: Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    fn new(streams: Vec<Pin<Box<dyn Stream<Item = StreamItem<T>> + Send + Sync>>>) -> Self {
        let count = streams.len();
        let buffered = (0..count).map(|_| None).collect();
        Self { streams, buffered }
    }
}

#[cfg(not(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
)))]
impl<T> OrderedMergeWithImmediateErrorsIndexed<T>
where
    T: Fluxion,
    T::Inner: Debug + Ord + Unpin + 'static,
    T::Timestamp: Debug + Ord + Copy + 'static,
{
    fn new(streams: Vec<Pin<Box<dyn Stream<Item = StreamItem<T>>>>>) -> Self {
        let count = streams.len();
        let buffered = (0..count).map(|_| None).collect();
        Self { streams, buffered }
    }
}

#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
))]
impl<T> Stream for OrderedMergeWithImmediateErrorsIndexed<T>
where
    T: Fluxion + Unpin,
    T::Inner: Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    type Item = (StreamItem<T>, usize);

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Poll streams to fill empty buffer slots - emit errors immediately with their index
        let mut any_pending = false;

        for i in 0..self.streams.len() {
            if self.buffered[i].is_none() {
                match self.streams[i].as_mut().poll_next(cx) {
                    Poll::Ready(Some(StreamItem::Error(e))) => {
                        // Rx fail-fast: emit error immediately with stream index
                        return Poll::Ready(Some((StreamItem::Error(e), i)));
                    }
                    Poll::Ready(Some(StreamItem::Value(item))) => {
                        self.buffered[i] = Some(item);
                    }
                    Poll::Ready(None) => {
                        // Stream is done, leave as None
                    }
                    Poll::Pending => {
                        any_pending = true;
                    }
                }
            }
        }

        // Find the minimum timestamped value among all buffered values
        let mut min_idx = None;
        let mut min_val: Option<&T> = None;

        for (i, item) in self.buffered.iter().enumerate() {
            if let Some(val) = item {
                let should_update = min_val.is_none_or(|curr_val| val < curr_val);

                if should_update {
                    min_idx = Some(i);
                    min_val = Some(val);
                }
            }
        }

        // Return the minimum value if found, with its stream index
        if let Some(idx) = min_idx {
            if let Some(item) = self.buffered[idx].take() {
                Poll::Ready(Some((StreamItem::Value(item), idx)))
            } else {
                unreachable!("min_idx is only Some when buffered[idx] is Some")
            }
        } else if any_pending {
            Poll::Pending
        } else {
            Poll::Ready(None)
        }
    }
}

#[cfg(not(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
)))]
impl<T> Stream for OrderedMergeWithImmediateErrorsIndexed<T>
where
    T: Fluxion + Unpin,
    T::Inner: Debug + Ord + Unpin + 'static,
    T::Timestamp: Debug + Ord + Copy + 'static,
{
    type Item = (StreamItem<T>, usize);

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Poll streams to fill empty buffer slots - emit errors immediately with their index
        let mut any_pending = false;

        for i in 0..self.streams.len() {
            if self.buffered[i].is_none() {
                match self.streams[i].as_mut().poll_next(cx) {
                    Poll::Ready(Some(StreamItem::Error(e))) => {
                        // Rx fail-fast: emit error immediately with stream index
                        return Poll::Ready(Some((StreamItem::Error(e), i)));
                    }
                    Poll::Ready(Some(StreamItem::Value(item))) => {
                        self.buffered[i] = Some(item);
                    }
                    Poll::Ready(None) => {
                        // Stream is done, leave as None
                    }
                    Poll::Pending => {
                        any_pending = true;
                    }
                }
            }
        }

        // Find the minimum timestamped value among all buffered values
        let mut min_idx = None;
        let mut min_val: Option<&T> = None;

        for (i, item) in self.buffered.iter().enumerate() {
            if let Some(val) = item {
                let should_update = min_val.is_none_or(|curr_val| val < curr_val);

                if should_update {
                    min_idx = Some(i);
                    min_val = Some(val);
                }
            }
        }

        // Return the minimum value if found, with its stream index
        if let Some(idx) = min_idx {
            if let Some(item) = self.buffered[idx].take() {
                Poll::Ready(Some((StreamItem::Value(item), idx)))
            } else {
                unreachable!("min_idx is only Some when buffered[idx] is Some")
            }
        } else if any_pending {
            Poll::Pending
        } else {
            Poll::Ready(None)
        }
    }
}
