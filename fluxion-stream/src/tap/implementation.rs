// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

macro_rules! define_tap_impl {
    ($($bounds:tt)*) => {
        use fluxion_core::{Fluxion, StreamItem};
        use futures::{Stream, StreamExt};
        use core::fmt::Debug;

        pub trait TapExt<T>: Stream<Item = StreamItem<T>> + Sized
        where
            T: Fluxion,
            T::Inner: Clone + Debug + Ord + Unpin + 'static + $($bounds)*,
            T::Timestamp: Debug + Ord + Copy + 'static + $($bounds)*,
        {
            fn tap<F>(self, mut f: F) -> impl Stream<Item = StreamItem<T>> + $($bounds)*
            where
                Self: Unpin + 'static + $($bounds)*,
                F: FnMut(&T::Inner) + 'static + $($bounds)*,
            {
                self.map(move |item| {
                    if let StreamItem::Value(value) = &item {
                        f(&value.clone().into_inner());
                    }
                    item
                })
            }
        }

        impl<S, T> TapExt<T> for S
        where
            S: Stream<Item = StreamItem<T>> + Unpin + 'static + $($bounds)*,
            T: Fluxion,
            T::Inner: Clone + Debug + Ord + Unpin + 'static + $($bounds)*,
            T::Timestamp: Debug + Ord + Copy + 'static + $($bounds)*,
        {
        }
    };
}
