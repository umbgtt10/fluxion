// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Multi-threaded version of map_ordered with Send + Sync bounds.

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
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    /// Maps each item to a new value while preserving temporal ordering.
    ///
    /// See the [module-level documentation](crate::map_ordered) for detailed examples and usage patterns.
    fn map_ordered<U, F>(self, f: F) -> impl Stream<Item = StreamItem<U>> + Send + Sync
    where
        Self: Send + Sync + Unpin + 'static,
        U: Fluxion,
        U::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
        U::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
        F: FnMut(T) -> U + Send + Sync + 'static;
}

impl<S, T> MapOrderedExt<T> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    fn map_ordered<U, F>(self, f: F) -> impl Stream<Item = StreamItem<U>> + Send + Sync
    where
        Self: Send + Sync + Unpin + 'static,
        U: Fluxion,
        U::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
        U::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
        F: FnMut(T) -> U + Send + Sync + 'static,
    {
        map_ordered_impl(self, f)
    }
}
