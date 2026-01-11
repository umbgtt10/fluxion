// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Single-threaded version of map_ordered without Send + Sync bounds.

use super::implementation::map_ordered_impl;
use core::fmt::Debug;
use fluxion_core::{Fluxion, StreamItem};
use futures::Stream;

/// Extension trait providing the `map_ordered` operator for streams.
///
/// This trait allows any stream of `StreamItem<T>` to transform items while
/// preserving temporal ordering semantics.
pub trait MapOrderedExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Unpin + 'static,
    T::Timestamp: Debug + Ord + Copy + 'static,
{
    /// Maps each item to a new value while preserving temporal ordering.
    ///
    /// This operator transforms each item in the stream using the provided function.
    /// Unlike the standard `map` operator from [`StreamExt`], `map_ordered` is
    /// designed for Fluxion streams and preserves the `StreamItem` wrapper semantics.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that transforms each item of type `T` into type `U`
    ///
    /// # Behavior
    ///
    /// - Values are transformed using the provided function
    /// - Errors are passed through unchanged
    /// - Timestamps are preserved/transformed according to the output type
    fn map_ordered<U, F>(self, f: F) -> impl Stream<Item = StreamItem<U>>
    where
        Self: Unpin + 'static,
        U: Fluxion,
        U::Inner: Clone + Debug + Ord + Unpin + 'static,
        U::Timestamp: Debug + Ord + Copy + 'static,
        F: FnMut(T) -> U + 'static;
}

impl<S, T> MapOrderedExt<T> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Unpin + 'static,
    T::Timestamp: Debug + Ord + Copy + 'static,
{
    fn map_ordered<U, F>(self, f: F) -> impl Stream<Item = StreamItem<U>>
    where
        Self: Unpin + 'static,
        U: Fluxion,
        U::Inner: Clone + Debug + Ord + Unpin + 'static,
        U::Timestamp: Debug + Ord + Copy + 'static,
        F: FnMut(T) -> U + 'static,
    {
        map_ordered_impl(self, f)
    }
}
