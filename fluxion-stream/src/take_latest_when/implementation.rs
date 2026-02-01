// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

macro_rules! define_take_latest_when_impl {
    ($($bounds:tt)*) => {
        use crate::ordered_merge::ordered_merge_with_index;
        use alloc::boxed::Box;
        use alloc::sync::Arc;
        use alloc::vec;
        use alloc::vec::Vec;
        use core::fmt::Debug;
        use core::pin::Pin;
        use fluxion_core::fluxion_mutex::Mutex;
        use fluxion_core::into_stream::IntoStream;
        use fluxion_core::{Fluxion, StreamItem};
        use futures::{Stream, StreamExt};

        pub trait TakeLatestWhenExt<T>: Stream<Item = StreamItem<T>> + Sized
        where
            T: Fluxion,
            T::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
            T::Timestamp: Debug + Ord + Copy + $($bounds)* 'static,
        {
            fn take_latest_when<IS>(
                self,
                filter_stream: IS,
                filter: impl Fn(&T::Inner) -> bool + $($bounds)* 'static,
            ) -> impl Stream<Item = StreamItem<T>> + $($bounds)*
            where
                IS: IntoStream<Item = StreamItem<T>>,
                IS::Stream: $($bounds)* 'static;
        }

        impl<T, S> TakeLatestWhenExt<T> for S
        where
            S: Stream<Item = StreamItem<T>> + Unpin + $($bounds)* 'static,
            T: Fluxion,
            T::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
            T::Timestamp: Debug + Ord + Copy + $($bounds)* 'static,
        {
            fn take_latest_when<IS>(
                self,
                filter_stream: IS,
                filter: impl Fn(&T::Inner) -> bool + $($bounds)* 'static,
            ) -> impl Stream<Item = StreamItem<T>> + $($bounds)*
            where
                IS: IntoStream<Item = StreamItem<T>>,
                IS::Stream: $($bounds)* 'static,
            {
                let streams: Vec<Pin<Box<dyn Stream<Item = StreamItem<T>> + $($bounds)*>>> =
                    vec![Box::pin(self), Box::pin(filter_stream.into_stream())];

                let source_value = Arc::new(Mutex::new(None));
                let filter = Arc::new(filter);

                let combined_stream = ordered_merge_with_index(streams).filter_map(move |(item, index)| {
                    let source_value = Arc::clone(&source_value);
                    let filter = Arc::clone(&filter);
                    async move {
                        match item {
                            StreamItem::Value(ordered_value) => {
                                match index {
                                    0 => {
                                        let mut source = source_value.lock();
                                        *source = Some(ordered_value);
                                        None
                                    }
                                    1 => {
                                        let source = source_value.lock();

                                        let filter_inner = ordered_value.clone().into_inner();

                                        if filter(&filter_inner) {
                                            source.as_ref().map(|src| {
                                                StreamItem::Value(T::with_timestamp(
                                                    src.clone().into_inner(),
                                                    ordered_value.timestamp(),
                                                ))
                                            })
                                        } else {
                                            None
                                        }
                                    }
                                    _ => {
                                        None
                                    }
                                }
                            }
                            StreamItem::Error(e) => Some(StreamItem::Error(e)),
                        }
                    }
                });

                Box::pin(combined_stream)
            }
        }
    }
}
