// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

macro_rules! define_combine_with_previous_impl {
    ($($bounds:tt)*) => {
        use crate::types::WithPrevious;
        use alloc::boxed::Box;
        use core::fmt::Debug;
        use fluxion_core::{Fluxion, StreamItem};
        use futures::{future::ready, Stream, StreamExt};

        pub trait CombineWithPreviousExt<T>: Stream<Item = StreamItem<T>> + Sized
        where
            T: Fluxion,
            T::Inner: Debug + Ord + Unpin + $($bounds)* 'static,
            T::Timestamp: Debug + Ord + Copy + $($bounds)* 'static,
        {
            fn combine_with_previous(self) -> impl Stream<Item = StreamItem<WithPrevious<T>>> + $($bounds)*;
        }

        impl<T, S> CombineWithPreviousExt<T> for S
        where
            S: Stream<Item = StreamItem<T>> + Sized + $($bounds)* 'static,
            T: Fluxion,
            T::Inner: Debug + Ord + Unpin + $($bounds)* 'static,
            T::Timestamp: Debug + Ord + Copy + $($bounds)* 'static,
        {
            fn combine_with_previous(self) -> impl Stream<Item = StreamItem<WithPrevious<T>>> + $($bounds)* {
                Box::pin(
                    self.scan(None, |state: &mut Option<T>, item: StreamItem<T>| {
                        ready(Some(match item {
                            StreamItem::Value(current) => {
                                let previous = state.take();
                                *state = Some(current.clone());
                                StreamItem::Value(WithPrevious::new(previous, current))
                            }
                            StreamItem::Error(e) => StreamItem::Error(e),
                        }))
                    }),
                )
            }
        }
    };
}
