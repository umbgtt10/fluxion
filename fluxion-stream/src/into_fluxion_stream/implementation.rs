// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

macro_rules! define_into_fluxion_stream_impl {
    ($($bounds:tt)*) => {
        use alloc::boxed::Box;
        use async_channel::Receiver;
        use core::fmt::Debug;
        use core::pin::Pin;
        use fluxion_core::{StreamItem, Timestamped};
        use futures::{Stream, StreamExt};

        /// Extension trait to convert futures channels into fluxion streams.
        ///
        /// This trait provides a simple way to wrap a futures `UnboundedReceiver` into
        /// a stream that emits `StreamItem::Value` for each received item.
        pub trait IntoFluxionStream<T> {
            /// Converts this receiver into a fluxion stream.
            ///
            /// Each item received from the channel is wrapped in `StreamItem::Value`.
            ///
            /// # Example
            ///
            /// ```rust
            /// use fluxion_stream::IntoFluxionStream;
            /// use async_channel::unbounded;
            ///
            /// let (tx, rx) = unbounded::<i32>();
            /// let stream = rx.into_fluxion_stream();
            /// ```
            fn into_fluxion_stream(self) -> impl Stream<Item = StreamItem<T>> + $($bounds)*;

            fn into_fluxion_stream_map<U, F>(
                self,
                mapper: F,
            ) -> Pin<Box<dyn Stream<Item = StreamItem<U>> + $($bounds)*>>
            where
                F: FnMut(T) -> U + 'static + $($bounds)*,
                U: Timestamped + Clone + Debug + Ord + Unpin + 'static + $($bounds)*;
        }

        impl<T: 'static + $($bounds)*> IntoFluxionStream<T> for Receiver<T> {
            fn into_fluxion_stream(self) -> impl Stream<Item = StreamItem<T>> + $($bounds)* {
                Box::pin(self.map(StreamItem::Value))
            }

            fn into_fluxion_stream_map<U, F>(
                self,
                mut mapper: F,
            ) -> Pin<Box<dyn Stream<Item = StreamItem<U>> + $($bounds)*>>
            where
                F: FnMut(T) -> U + 'static + $($bounds)*,
                U: Timestamped + Clone + Debug + Ord + Unpin + 'static + $($bounds)*,
            {
                Box::pin(self.map(move |value| StreamItem::Value(mapper(value))))
            }
        }
    };
}
