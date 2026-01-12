// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

macro_rules! define_take_items_impl {
    ($($bounds:tt)*) => {
        use fluxion_core::StreamItem;
        use futures::{Stream, StreamExt};

        /// Extension trait providing the `take_items` operator for streams.
        ///
        /// This trait allows any stream of `StreamItem<T>` to be limited to
        /// the first n items.
        pub trait TakeItemsExt<T>: Stream<Item = StreamItem<T>> + Sized {
            /// Emits only the first `n` items from the stream, then completes.
            ///
            /// After emitting `n` items, the stream will complete and no further items
            /// will be emitted, even if the source stream continues to produce values.
            ///
            /// See the [module-level documentation](crate::take_items) for detailed examples and usage patterns.
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
