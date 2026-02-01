// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

macro_rules! define_take_items_impl {
    ($($bounds:tt)*) => {
        use fluxion_core::StreamItem;
        use futures::{Stream, StreamExt};

        pub trait TakeItemsExt<T>: Stream<Item = StreamItem<T>> + Sized {
            fn take_items(self, n: usize) -> impl Stream<Item = StreamItem<T>> + $($bounds)*;
        }

        impl<S, T> TakeItemsExt<T> for S
        where
            S: Stream<Item = StreamItem<T>> + $($bounds)*,
        {
            fn take_items(self, n: usize) -> impl Stream<Item = StreamItem<T>> + $($bounds)* {
                StreamExt::take(self, n)
            }
        }
    };
}
