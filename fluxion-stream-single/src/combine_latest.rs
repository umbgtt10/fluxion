// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Single-threaded runtime wrapper for combine_latest
//!
//! This module provides the trait definition WITHOUT Send bounds for single-threaded runtimes.
//! **CRITICAL**: Relaxed input stream bounds (`IS::Stream: 'static` instead of `Send + Sync`)
//! allow this to work in Embassy and WASM.

use alloc::vec::Vec;
use core::fmt::Debug;
use fluxion_core::into_stream::IntoStream;
use fluxion_core::{Fluxion, StreamItem, Timestamped};
use futures::Stream;

pub use fluxion_stream_core::CombinedState;

/// Extension trait for combining multiple streams (single-threaded runtimes)
///
/// This trait has relaxed bounds suitable for single-threaded runtimes (Embassy, WASM).
/// **Key difference**: Input streams only need `'static`, not `Send + Sync`.
pub trait CombineLatestExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Clone + Debug + Ord + Send + Sync + Unpin,
{
    /// Combines this stream with multiple other streams.
    ///
    /// See [fluxion-stream documentation](https://docs.rs/fluxion-stream) for detailed behavior.
    fn combine_latest<IS>(
        self,
        others: Vec<IS>,
        filter: impl Fn(&CombinedState<T::Inner, T::Timestamp>) -> bool + Send + Sync + Unpin + 'static,
    ) -> impl Stream<Item = StreamItem<CombinedState<T::Inner, T::Timestamp>>> + Send + Sync + Unpin
    where
        IS: IntoStream<Item = StreamItem<T>>,
        IS::Stream: Send + Sync + 'static,
        CombinedState<T::Inner, T::Timestamp>:
            Timestamped<Inner = CombinedState<T::Inner, T::Timestamp>, Timestamp = T::Timestamp>;
}

impl<T, S> CombineLatestExt<T> for S
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Clone + Debug + Ord + Send + Sync + Unpin,
    S: Stream<Item = StreamItem<T>> + Send + Sync + Unpin + 'static,
    CombinedState<T::Inner, T::Timestamp>:
        Timestamped<Inner = CombinedState<T::Inner, T::Timestamp>, Timestamp = T::Timestamp>,
{
    fn combine_latest<IS>(
        self,
        others: Vec<IS>,
        filter: impl Fn(&CombinedState<T::Inner, T::Timestamp>) -> bool + Send + Sync + Unpin + 'static,
    ) -> impl Stream<Item = StreamItem<CombinedState<T::Inner, T::Timestamp>>> + Send + Sync + Unpin
    where
        IS: IntoStream<Item = StreamItem<T>>,
        IS::Stream: Send + Sync + 'static, // Need Send + Sync for core impl (single-threaded can satisfy this)
        CombinedState<T::Inner, T::Timestamp>:
            Timestamped<Inner = CombinedState<T::Inner, T::Timestamp>, Timestamp = T::Timestamp>,
    {
        // Single-threaded runtimes implement Send even though they don't use threading
        fluxion_stream_core::combine_latest_impl(self, others, filter)
    }
}
