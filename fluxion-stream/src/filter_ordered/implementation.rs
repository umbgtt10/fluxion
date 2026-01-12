// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

macro_rules! define_filter_ordered_impl {
    ($($bounds:tt)*) => {
        use core::fmt::Debug;
        use fluxion_core::{Fluxion, StreamItem};
        use futures::future::ready;
        use futures::Stream;
        use futures::StreamExt;

        pub trait FilterOrderedExt<T>: Stream<Item = StreamItem<T>> + Sized
        where
            T: Fluxion,
            T::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
            T::Timestamp: Debug + Ord + Copy + $($bounds)* 'static,
        {
            fn filter_ordered<F>(self, predicate: F) -> impl Stream<Item = StreamItem<T>> + $($bounds)*
            where
                Self: Unpin + $($bounds)* 'static,
                F: FnMut(&T::Inner) -> bool + $($bounds)* 'static;
        }

        impl<S, T> FilterOrderedExt<T> for S
        where
            S: Stream<Item = StreamItem<T>>,
            T: Fluxion,
            T::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
            T::Timestamp: Debug + Ord + Copy + $($bounds)* 'static,
        {
            fn filter_ordered<F>(self, mut predicate: F) -> impl Stream<Item = StreamItem<T>> + $($bounds)*
            where
                Self: Unpin + $($bounds)* 'static,
                F: FnMut(&T::Inner) -> bool + $($bounds)* 'static,
            {
                self.filter_map(move |item| {
                    ready(match item {
                        StreamItem::Value(value) if predicate(&value.clone().into_inner()) => {
                            Some(StreamItem::Value(value))
                        }
                        StreamItem::Value(_) => None,
                        StreamItem::Error(e) => Some(StreamItem::Error(e)),
                    })
                })
            }
        }
    };
}
