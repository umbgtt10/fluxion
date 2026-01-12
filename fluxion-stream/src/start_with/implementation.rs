// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

macro_rules! define_start_with_impl {
    ($($bounds:tt)*) => {
        use alloc::vec::Vec;
        use fluxion_core::StreamItem;
        use futures::{stream::iter, Stream, StreamExt};

        /// Extension trait providing the `start_with` operator for streams.
        ///
        /// This trait allows any stream of `StreamItem<T>` to have initial values
        /// prepended before the source stream items.
        pub trait StartWithExt<T>: Stream<Item = StreamItem<T>> + Sized {
            /// Prepends initial values to the stream.
            ///
            /// The initial values will be emitted first, in the order provided, followed by
            /// all values from the source stream. The initial values must have timestamps
            /// that respect the temporal ordering constraints.
            ///
            /// See the [module-level documentation](crate::start_with) for detailed examples and usage patterns.
            fn start_with(self, initial_values: Vec<StreamItem<T>>) -> impl Stream<Item = StreamItem<T>> + $($bounds)*;
        }

        impl<S, T> StartWithExt<T> for S
        where
            S: Stream<Item = StreamItem<T>> + $($bounds)*,
            T: $($bounds)*,
        {
            fn start_with(self, initial_values: Vec<StreamItem<T>>) -> impl Stream<Item = StreamItem<T>> + $($bounds)* {
                let initial_stream = iter(initial_values);
                initial_stream.chain(self)
            }
        }
    };
}
