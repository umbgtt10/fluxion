// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

macro_rules! define_skip_items_impl {
    ($($bounds:tt)*) => {
        use fluxion_core::StreamItem;
        use futures::{Stream, StreamExt};

        /// Extension trait providing the `skip_items` operator for streams.
        ///
        /// This trait allows any stream of `StreamItem<T>` to skip its first n items.
        pub trait SkipItemsExt<T>: Stream<Item = StreamItem<T>> + Sized {
            /// Skips the first `n` items from the stream.
            ///
            /// The first `n` items will be discarded, and only items after that will be emitted.
            /// If the stream has fewer than `n` items, no items will be emitted.
            ///
            /// See the [module-level documentation](crate::skip_items) for detailed examples and usage patterns.
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
