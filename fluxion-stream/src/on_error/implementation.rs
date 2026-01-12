// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

macro_rules! define_on_error_impl {
    ($($bounds:tt)*) => {
        use fluxion_core::{FluxionError, StreamItem};
        use futures::future::ready;
        use futures::{Stream, StreamExt};

        /// Extension trait providing the `on_error` operator for streams.
        ///
        /// This trait allows any stream of `StreamItem<T>` to handle errors
        /// with a custom handler function.
        pub trait OnErrorExt<T>: Stream<Item = StreamItem<T>> + Sized {
            /// Handle errors in the stream with a handler function.
            ///
            /// The handler receives a reference to each error and returns:
            /// - `true` to consume the error (remove from stream)
            /// - `false` to propagate the error downstream
            ///
            /// Multiple `on_error` operators can be chained to implement the
            /// Chain of Responsibility pattern for error handling.
            ///
            /// - [Error Handling Guide](../docs/ERROR-HANDLING.md) - Comprehensive error patterns
            fn on_error<F>(self, handler: F) -> impl Stream<Item = StreamItem<T>> + $($bounds)*
            where
                F: FnMut(&FluxionError) -> bool + $($bounds)* 'static,
                Self: $($bounds)* 'static; // Ensure Self satisfies bounds for return type
        }

        impl<S, T> OnErrorExt<T> for S
        where
            S: Stream<Item = StreamItem<T>> + $($bounds)* 'static,
            T: $($bounds)* 'static,
        {
            fn on_error<F>(self, mut handler: F) -> impl Stream<Item = StreamItem<T>> + $($bounds)*
            where
                F: FnMut(&FluxionError) -> bool + $($bounds)* 'static,
            {
                self.filter_map(move |item| {
                    ready(match item {
                        StreamItem::Error(err) => {
                            if handler(&err) {
                                // Error handled, skip it
                                None
                            } else {
                                // Error not handled, propagate
                                Some(StreamItem::Error(err))
                            }
                        }
                        other => Some(other),
                    })
                })
            }
        }
    };
}
