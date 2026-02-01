// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

macro_rules! define_scan_ordered_impl {
    ($($bounds:tt)*) => {
        use alloc::sync::Arc;
        use core::fmt::Debug;
        use fluxion_core::fluxion_mutex::Mutex;
        use fluxion_core::{Fluxion, StreamItem};
        use futures::{future::ready, Stream, StreamExt};

        pub trait ScanOrderedExt<T>: Stream<Item = StreamItem<T>> + Sized
        where
            T: Fluxion,
            T::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
            T::Timestamp: Debug + Ord + Copy + $($bounds)* 'static,
        {
            fn scan_ordered<Out, Acc, F>(
                self,
                initial: Acc,
                accumulator: F,
            ) -> impl Stream<Item = StreamItem<Out>>
            where
                Acc: $($bounds)* 'static,
                Out: Fluxion,
                Out::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
                Out::Timestamp: From<T::Timestamp> + Debug + Ord + Copy + $($bounds)* 'static,
                F: FnMut(&mut Acc, &T::Inner) -> Out::Inner + $($bounds)* 'static;
        }

        impl<T, S> ScanOrderedExt<T> for S
        where
            S: Stream<Item = StreamItem<T>> + $($bounds)* Sized + 'static,
            T: Fluxion,
            T::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
            T::Timestamp: Debug + Ord + Copy + $($bounds)* 'static,
        {
            fn scan_ordered<Out, Acc, F>(
                self,
                initial: Acc,
                accumulator: F,
            ) -> impl Stream<Item = StreamItem<Out>>
            where
                Acc: $($bounds)* 'static,
                Out: Fluxion,
                Out::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
                Out::Timestamp: From<T::Timestamp> + Debug + Ord + Copy + $($bounds)* 'static,
                F: FnMut(&mut Acc, &T::Inner) -> Out::Inner + $($bounds)* 'static,
            {
                let state = Arc::new(Mutex::new((initial, accumulator)));

                let result = self.then(move |item| {
                    let state = Arc::clone(&state);
                    ready(match item {
                        StreamItem::Value(value) => {
                            let timestamp = value.timestamp();
                            let inner = value.into_inner();

                            let mut guard = state.lock();
                            let (acc, ref mut f) = &mut *guard;
                            let output = f(acc, &inner);
                            StreamItem::Value(Out::with_timestamp(output, timestamp.into()))
                        }
                        StreamItem::Error(e) => {
                            StreamItem::Error(e)
                        }
                    })
                });

                result
            }
        }
    };
}
