// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::fmt::Debug;
use fluxion_core::into_stream::IntoStream;
use fluxion_core::{Fluxion, StreamItem};
use futures::Stream;

/// Extension trait providing high-level ordered merge for `Timestamped` streams.
///
/// This trait merges multiple streams of timestamped items, emitting all values from
/// all streams in temporal order. Unlike `combine_latest`, this emits every value
/// from every stream (not just when all have emitted).
///
/// This trait requires only `Send` (not `Sync`) on input streams, making it suitable for
/// single-threaded runtimes (Embassy, WASM).
pub trait OrderedStreamExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
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
    fn ordered_merge<IS>(
        self,
        others: Vec<IS>,
    ) -> impl Stream<Item = StreamItem<T>> + Send + Sync + Unpin
    where
        IS: IntoStream<Item = StreamItem<T>>,
        IS::Stream: Send + Sync + 'static;
}

impl<T, S> OrderedStreamExt<T> for S
where
    T: Fluxion + Unpin,
    T::Inner: Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
    S: Stream<Item = StreamItem<T>> + Send + Sync + Unpin + 'static,
{
    fn ordered_merge<IS>(
        self,
        others: Vec<IS>,
    ) -> impl Stream<Item = StreamItem<T>> + Send + Sync + Unpin
    where
        IS: IntoStream<Item = StreamItem<T>>,
        IS::Stream: Send + Sync + 'static,
    {
        fluxion_stream_core::ordered_merge_impl(self, others)
    }
}
