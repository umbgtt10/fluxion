// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

macro_rules! define_window_by_count_impl {
    ($($bounds:tt)*) => {
        use alloc::boxed::Box;
        use alloc::sync::Arc;
        use alloc::vec::Vec;
        use core::fmt::Debug;
        use core::mem::take;
        use fluxion_core::fluxion_mutex::Mutex;
        use fluxion_core::{Fluxion, StreamItem};
        use futures::{future::ready, Stream, StreamExt};

        /// Extension trait providing the [`window_by_count`](WindowByCountExt::window_by_count) operator.
        ///
        /// This trait is implemented for all streams of [`StreamItem<T>`] where `T` implements [`Fluxion`].
        pub trait WindowByCountExt<T>: Stream<Item = StreamItem<T>> + Sized
        where
            T: Fluxion,
            T::Inner: Clone + Debug + Ord + Unpin + 'static + $($bounds)*,
            T::Timestamp: Debug + Ord + Copy + 'static + $($bounds)*,
        {
            /// Groups consecutive items into fixed-size windows (batches).
            ///
            /// Collects items into vectors of size `n`. When `n` items have been collected,
            /// emits a `Vec<T::Inner>` with the timestamp of the last item in the window.
            /// On stream completion, any remaining items are emitted as a partial window.
            ///
            /// # Type Parameters
            ///
            /// - `Out`: The output wrapper type (must implement `Fluxion` with `Inner = Vec<T::Inner>`)
            ///
            /// # Arguments
            ///
            /// * `n` - The window size. Must be at least 1.
            fn window_by_count<Out>(self, n: usize) -> impl Stream<Item = StreamItem<Out>> + $($bounds)*
            where
                Out: Fluxion<Inner = Vec<T::Inner>>,
                Out::Inner: Clone + Debug + Ord + Unpin + 'static + $($bounds)*,
                Out::Timestamp: From<T::Timestamp> + Debug + Ord + Copy + 'static + $($bounds)*;
        }

        impl<S, T> WindowByCountExt<T> for S
        where
            S: Stream<Item = StreamItem<T>> + 'static + $($bounds)*,
            T: Fluxion,
            T::Inner: Clone + Debug + Ord + Unpin + 'static + $($bounds)*,
            T::Timestamp: Debug + Ord + Copy + 'static + $($bounds)*,
        {
            fn window_by_count<Out>(self, n: usize) -> impl Stream<Item = StreamItem<Out>> + $($bounds)*
            where
                Out: Fluxion<Inner = Vec<T::Inner>>,
                Out::Inner: Clone + Debug + Ord + Unpin + 'static + $($bounds)*,
                Out::Timestamp: From<T::Timestamp> + Debug + Ord + Copy + 'static + $($bounds)*,
            {
                assert!(n >= 1, "window_by_count: window size must be at least 1");

                // State: (buffer, last_timestamp)
                let state = Arc::new(Mutex::new((Vec::with_capacity(n), None::<T::Timestamp>)));

                // Use filter_map to accumulate and emit when window is full
                // We need to handle the completion case separately using chain
                let window_size = n;
                let state_clone = Arc::clone(&state);

                let main_stream = self.filter_map(move |item| {
                    let state = Arc::clone(&state_clone);
                    let window_size = window_size;

                    ready(match item {
                        StreamItem::Value(value) => {
                            let timestamp = value.timestamp();
                            let inner = value.into_inner();

                            let mut guard = state.lock();
                            let (buffer, last_ts) = &mut *guard;

                            buffer.push(inner);
                            *last_ts = Some(timestamp);

                            if buffer.len() >= window_size {
                                let window = take(buffer);
                                *buffer = Vec::with_capacity(window_size);
                                let ts = last_ts.take().expect("timestamp must exist");
                                Some(StreamItem::Value(Out::with_timestamp(window, ts.into())))
                            } else {
                                None
                            }
                        }
                        StreamItem::Error(e) => {
                            // Clear buffer and propagate error
                            let mut guard = state.lock();
                            let (buffer, last_ts) = &mut *guard;
                            buffer.clear();
                            *last_ts = None;
                            Some(StreamItem::Error(e))
                        }
                    })
                });

                // Chain with a stream that emits partial window on completion
                let final_state = state;
                let flush_stream = futures::stream::once(async move {
                    let mut guard = final_state.lock();
                    let (buffer, last_ts) = &mut *guard;

                    if !buffer.is_empty() {
                        let window = take(buffer);
                        let ts = last_ts
                            .take()
                            .expect("timestamp must exist for partial window");
                        Some(StreamItem::Value(Out::with_timestamp(window, ts.into())))
                    } else {
                        None
                    }
                })
                .filter_map(ready);

                Box::pin(main_stream.chain(flush_stream))
            }
        }
    };
}
