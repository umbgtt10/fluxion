// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::StreamItem;
use futures::{Stream, StreamExt};

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

        pub trait MapOrderedExt<T>: Stream<Item = StreamItem<T>> + Sized
        where
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
