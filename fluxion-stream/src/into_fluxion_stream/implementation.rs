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

        pub trait IntoFluxionStream<T> {
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
