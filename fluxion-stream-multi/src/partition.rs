// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Partition operator - WASM compatible
//!
//! This operator requires dynamic task spawning via FluxionTask and is available for:
//! - WASM (wasm32 target)
//!
//! **Not available in Embassy** - Embassy cannot dynamically spawn tasks.

use core::fmt::Debug;
use fluxion_core::{Fluxion, StreamItem};
use futures::Stream;

pub use fluxion_stream_core::PartitionedStream;

/// Extension trait providing the `partition` operator for streams.
///
/// Available for WASM. Requires FluxionTask for dynamic task spawning.
/// Not available in Embassy which cannot dynamically spawn tasks.
pub trait PartitionExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
{
    /// Partitions the stream into two based on a predicate.
    ///
    /// This operator routes each item to one of two output streams:
    /// - Items where `predicate(&inner)` returns `true` go to the first stream
    /// - Items where `predicate(&inner)` returns `false` go to the second stream
    ///
    /// # Behavior
    ///
    /// - Each item is sent to exactly one output stream
    /// - Errors are propagated to both streams
    /// - When the source completes, both output streams complete
    /// - Spawns a background task to perform routing
    ///
    /// # Arguments
    ///
    /// * `predicate` - A function that takes `&T::Inner` and returns `true` or `false`
    ///
    /// # Returns
    ///
    /// A tuple of two streams: `(true_stream, false_stream)`
    fn partition<F>(self, predicate: F) -> (PartitionedStream<T>, PartitionedStream<T>)
    where
        Self: Send + Unpin + 'static,
        F: Fn(&T::Inner) -> bool + Send + Sync + Unpin + 'static;
}

impl<S, T> PartitionExt<T> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
{
    fn partition<F>(self, predicate: F) -> (PartitionedStream<T>, PartitionedStream<T>)
    where
        Self: Send + Unpin + 'static,
        F: Fn(&T::Inner) -> bool + Send + Sync + Unpin + 'static,
    {
        fluxion_stream_core::partition_impl(self, predicate)
    }
}
