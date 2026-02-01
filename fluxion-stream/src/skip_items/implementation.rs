// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

macro_rules! define_skip_items_impl {
    ($($bounds:tt)*) => {
        use fluxion_core::StreamItem;
        use futures::{Stream, StreamExt};

        pub trait SkipItemsExt<T>: Stream<Item = StreamItem<T>> + Sized {
            fn skip_items(self, n: usize) -> impl Stream<Item = StreamItem<T>> + $($bounds)*;
        }

        impl<S, T> SkipItemsExt<T> for S
        where
            S: Stream<Item = StreamItem<T>> + $($bounds)*,
        {
            fn skip_items(self, n: usize) -> impl Stream<Item = StreamItem<T>> + $($bounds)* {
                StreamExt::skip(self, n)
            }
        }
    };
}
