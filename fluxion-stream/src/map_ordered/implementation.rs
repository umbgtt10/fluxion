// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Common implementation for map_ordered operator.
//!
//! This module contains the shared logic used by both multi-threaded and single-threaded
//! versions of the map_ordered operator.

use fluxion_core::StreamItem;
use futures::{Stream, StreamExt};

/// Maps each item in a stream using the provided function.
///
/// This is the core implementation that transforms `StreamItem<T>` to `StreamItem<U>`
/// by applying the function to the contained value while preserving the stream structure.
#[inline]
pub(super) fn map_ordered_impl<S, T, U, F>(stream: S, mut f: F) -> impl Stream<Item = StreamItem<U>>
where
    S: Stream<Item = StreamItem<T>>,
    F: FnMut(T) -> U,
{
    stream.map(move |item| item.map(&mut f))
}

macro_rules! define_map_ordered_impl {
    ($($bounds:tt)*) => {
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
            T::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
            T::Timestamp: Debug + Ord + Copy + $($bounds)* 'static,
        {
            /// Maps each item to a new value while preserving temporal ordering.
            ///
            /// See the [module-level documentation](crate::map_ordered) for detailed examples and usage patterns.
            fn map_ordered<U, F>(self, f: F) -> impl Stream<Item = StreamItem<U>> + $($bounds)*
            where
                Self: Unpin + $($bounds)* 'static,
                U: Fluxion,
                U::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
                U::Timestamp: Debug + Ord + Copy + $($bounds)* 'static,
                F: FnMut(T) -> U + $($bounds)* 'static;
        }

        impl<S, T> MapOrderedExt<T> for S
        where
            S: Stream<Item = StreamItem<T>>,
            T: Fluxion,
            T::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
            T::Timestamp: Debug + Ord + Copy + $($bounds)* 'static,
        {
            fn map_ordered<U, F>(self, f: F) -> impl Stream<Item = StreamItem<U>> + $($bounds)*
            where
                Self: Unpin + $($bounds)* 'static,
                U: Fluxion,
                U::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
                U::Timestamp: Debug + Ord + Copy + $($bounds)* 'static,
                F: FnMut(T) -> U + $($bounds)* 'static,
            {
                map_ordered_impl(self, f)
            }
        }
    };
}
