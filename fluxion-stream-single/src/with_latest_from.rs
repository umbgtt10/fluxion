// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::fmt::Debug;
use fluxion_core::into_stream::IntoStream;
use fluxion_core::{Fluxion, StreamItem};
use fluxion_stream_core::CombinedState;
use futures::Stream;

/// Extension trait providing the `with_latest_from` operator for timestamped streams.
///
/// This operator combines a primary stream with a secondary stream, emitting only
/// when the primary stream emits, using the latest value from the secondary stream.
///
/// This trait requires only `Send` (not `Sync`) on input streams, making it suitable for
/// single-threaded runtimes (Embassy, WASM).
pub trait WithLatestFromExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
{
    /// Combines elements from the primary stream (self) with the latest element from the secondary stream (other).
    ///
    /// This operator only emits when the primary stream emits. It waits until both streams
    /// have emitted at least once, then for each primary emission, it combines the primary
    /// value with the most recent secondary value.
    ///
    /// # Behavior
    ///
    /// - Emissions are triggered **only** by the primary stream (self)
    /// - Secondary stream updates are stored but don't trigger emissions
    /// - Waits until both streams have emitted at least once
    /// - Preserves temporal ordering from the primary stream
    ///
    /// # Arguments
    ///
    /// * `other` - Secondary stream whose latest value will be combined with primary emissions
    /// * `result_selector` - Function that transforms the `CombinedState` into the output type `R`.
    ///   The combined state contains `[primary_value, secondary_value]`.
    ///
    /// # Returns
    ///
    /// A stream of `R` where each emission contains the result of applying
    /// `result_selector` to the combined state, ordered by the primary stream's order.
    fn with_latest_from<IS, R>(
        self,
        other: IS,
        result_selector: impl Fn(&CombinedState<T::Inner, T::Timestamp>) -> R
            + Send
            + Sync
            + Unpin
            + 'static,
    ) -> impl Stream<Item = StreamItem<R>> + Send + Sync + Unpin
    where
        IS: IntoStream<Item = StreamItem<T>>,
        IS::Stream: Send + Sync + 'static,
        R: Fluxion,
        R::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
        R::Timestamp: From<T::Timestamp> + Debug + Ord + Send + Sync + Unpin + Copy + 'static;
}

impl<T, S> WithLatestFromExt<T> for S
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
    S: Stream<Item = StreamItem<T>> + Send + Sync + Unpin + 'static,
{
    fn with_latest_from<IS, R>(
        self,
        other: IS,
        result_selector: impl Fn(&CombinedState<T::Inner, T::Timestamp>) -> R
            + Send
            + Sync
            + Unpin
            + 'static,
    ) -> impl Stream<Item = StreamItem<R>> + Send + Sync + Unpin
    where
        IS: IntoStream<Item = StreamItem<T>>,
        IS::Stream: Send + Sync + 'static,
        R: Fluxion,
        R::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
        R::Timestamp: From<T::Timestamp> + Debug + Ord + Send + Sync + Unpin + Copy + 'static,
    {
        fluxion_stream_core::with_latest_from_impl(self, other, result_selector)
    }
}
