// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::fmt::Debug;
use fluxion_core::into_stream::IntoStream;
use fluxion_core::{Fluxion, StreamItem};
use fluxion_stream_core::CombinedState;
use futures::Stream;

/// Extension trait providing the `emit_when` operator for timestamped streams.
///
/// This operator gates a source stream based on conditions from a filter stream,
/// emitting source values only when the combined state passes a predicate.
///
/// This trait requires only `Send` (not `Sync`) on input streams, making it suitable for
/// single-threaded runtimes (Embassy, WASM).
pub trait EmitWhenExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
{
    /// Emits source stream values only when the filter condition is satisfied.
    ///
    /// This operator maintains the latest values from both the source and filter streams,
    /// creating a combined state. Source values are emitted only when this combined state
    /// passes the provided filter predicate.
    ///
    /// # Behavior
    ///
    /// - Maintains latest value from both source and filter streams
    /// - Evaluates predicate on `CombinedState` containing both values
    /// - Emits source value when predicate returns `true`
    /// - Both streams must emit at least once before any emission occurs
    /// - Preserves temporal ordering of source stream
    ///
    /// # Arguments
    ///
    /// * `filter_stream` - Stream providing filter values for the gate condition
    /// * `filter` - Predicate function that receives `CombinedState<T::Inner>` containing
    ///   `[source_value, filter_value]` and returns `true` to emit.
    fn emit_when<IS>(
        self,
        filter_stream: IS,
        filter: impl Fn(&CombinedState<T::Inner, T::Timestamp>) -> bool + Send + Sync + Unpin + 'static,
    ) -> impl Stream<Item = StreamItem<T>> + Send + Sync + Unpin
    where
        IS: IntoStream<Item = StreamItem<T>>,
        IS::Stream: Send + Sync + 'static;
}

impl<T, S> EmitWhenExt<T> for S
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
    S: Stream<Item = StreamItem<T>> + Send + Sync + Unpin + 'static,
{
    fn emit_when<IS>(
        self,
        filter_stream: IS,
        filter: impl Fn(&CombinedState<T::Inner, T::Timestamp>) -> bool + Send + Sync + Unpin + 'static,
    ) -> impl Stream<Item = StreamItem<T>> + Send + Sync + Unpin
    where
        IS: IntoStream<Item = StreamItem<T>>,
        IS::Stream: Send + Sync + 'static,
    {
        fluxion_stream_core::emit_when_impl(self, filter_stream, filter)
    }
}
