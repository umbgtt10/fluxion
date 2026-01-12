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

        /// Extension trait providing the `scan_ordered` operator for streams.
        ///
        /// See the [module-level documentation](crate::scan_ordered) for details and examples.
        pub trait ScanOrderedExt<T>: Stream<Item = StreamItem<T>> + Sized
        where
            T: Fluxion,
            T::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
            T::Timestamp: Debug + Ord + Copy + $($bounds)* 'static,
        {
            /// Accumulates state across stream items, emitting intermediate results.
            ///
            /// The `scan_ordered` operator maintains an accumulator value that is updated for each
            /// input item. For each input, it calls the accumulator function with a mutable
            /// reference to the current state and the input value, producing an output value.
            ///
            /// See the [module-level documentation](crate::scan_ordered) for details.
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

                            // Lock state and apply accumulator function
                            let mut guard = state.lock();
                            let (acc, ref mut f) = &mut *guard;
                            let output = f(acc, &inner);
                            StreamItem::Value(Out::with_timestamp(output, timestamp.into()))
                        }
                        StreamItem::Error(e) => {
                            // Propagate error without affecting accumulator state
                            StreamItem::Error(e)
                        }
                    })
                });

                result
            }
        }
    };
}
