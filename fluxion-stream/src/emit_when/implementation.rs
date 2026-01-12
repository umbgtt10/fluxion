// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

macro_rules! define_emit_when_impl {
    ($($bounds:tt)*) => {
        use $crate::ordered_merge::ordered_merge_with_index;
        use $crate::types::CombinedState;
        use $crate::warn;
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

        type SharedState<V, TS> = Arc<Mutex<Option<(V, TS)>>>;

        /// Extension trait providing the `emit_when` operator for timestamped streams.
        ///
        /// This operator gates a source stream based on conditions from a filter stream,
        /// emitting source values only when the combined state passes a predicate.
        pub trait EmitWhenExt<T>: Stream<Item = StreamItem<T>> + Sized
        where
            T: Fluxion,
            T::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
            T::Timestamp: Debug + Ord + Copy + $($bounds)* 'static,
        {
            /// Emits source stream values only when the filter condition is satisfied.
            ///
            /// This operator maintains the latest values from both the source and filter streams,
            /// creating a combined state. Source values are emitted only when this combined state
            /// passes the provided filter predicate.
            ///
            /// # Behavior
            ///
            /// - Maintains latest value from both source and filter streams
            /// - Evaluates predicate on `CombinedState` containing both values
            /// - Emits source value when predicate returns `true`
            /// - Both streams must emit at least once before any emission occurs
            /// - Preserves temporal ordering of source stream
            ///
            /// # Arguments
            ///
            /// * `filter_stream` - Stream providing filter values for the gate condition
            /// * `filter` - Predicate function that receives `CombinedState<T::Inner>` containing
            ///   `[source_value, filter_value]` and returns `true` to emit.
            ///
            /// # Returns
            ///
            /// A pinned stream of `T` containing source values that pass the filter condition.
            fn emit_when<IS>(
                self,
                filter_stream: IS,
                filter: impl Fn(&CombinedState<T::Inner, T::Timestamp>) -> bool + $($bounds)* 'static,
            ) -> impl Stream<Item = StreamItem<T>>
            where
                IS: IntoStream<Item = fluxion_core::StreamItem<T>>,
                IS::Stream: $($bounds)* 'static;
        }

        impl<T, S> EmitWhenExt<T> for S
        where
            S: Stream<Item = StreamItem<T>> + Unpin + $($bounds)* 'static,
            T: Fluxion,
            T::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
            T::Timestamp: Debug + Ord + Copy + $($bounds)* 'static,
        {
            fn emit_when<IS>(
                self,
                filter_stream: IS,
                filter: impl Fn(&CombinedState<T::Inner, T::Timestamp>) -> bool + $($bounds)* 'static,
            ) -> impl Stream<Item = StreamItem<T>>
            where
                IS: IntoStream<Item = fluxion_core::StreamItem<T>>,
                IS::Stream: $($bounds)* 'static,
            {
                let streams: Vec<Pin<Box<dyn Stream<Item = StreamItem<T>> + $($bounds)*>>> =
                    vec![Box::pin(self), Box::pin(filter_stream.into_stream())];

                let source_value: SharedState<T::Inner, T::Timestamp> = Arc::new(Mutex::new(None));
                let filter_value: SharedState<T::Inner, T::Timestamp> = Arc::new(Mutex::new(None));
                let filter = Arc::new(filter);

                let combined_stream = ordered_merge_with_index(streams).filter_map(move |(item, index)| {
                    let source_value = Arc::clone(&source_value);
                    let filter_value = Arc::clone(&filter_value);
                    let filter = Arc::clone(&filter);
                    async move {
                        match item {
                            StreamItem::Value(ordered_value) => match index {
                                0 => {
                                    let mut source = source_value.lock();
                                    let filter_val = filter_value.lock();

                                    let timestamp = ordered_value.timestamp();
                                    *source = Some((ordered_value.clone().into_inner(), timestamp));

                                    if let Some((src, src_ts)) = source.as_ref() {
                                        if let Some((filt, filt_ts)) = filter_val.as_ref() {
                                            let combined_state = CombinedState::new(
                                                vec![(src.clone(), *src_ts), (filt.clone(), *filt_ts)],
                                                timestamp,
                                            );
                                            if filter(&combined_state) {
                                                Some(StreamItem::Value(T::with_timestamp(
                                                    src.clone(),
                                                    *src_ts,
                                                )))
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                }
                                1 => {
                                    let mut filter_val = filter_value.lock();
                                    let source = source_value.lock();

                                    let timestamp = ordered_value.timestamp();
                                    *filter_val = Some((ordered_value.clone().into_inner(), timestamp));

                                    if let Some((src, src_ts)) = source.as_ref() {
                                        if let Some((filt, filt_ts)) = filter_val.as_ref() {
                                            let combined_state = CombinedState::new(
                                                vec![(src.clone(), *src_ts), (filt.clone(), *filt_ts)],
                                                timestamp,
                                            );
                                            if filter(&combined_state) {
                                                Some(StreamItem::Value(T::with_timestamp(
                                                    src.clone(),
                                                    *filt_ts,
                                                )))
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                }
                                _ => {
                                    warn!("emit_when: unexpected stream index {} — ignoring", index);
                                    None
                                }
                            },
                            StreamItem::Error(e) => Some(StreamItem::Error(e)),
                        }
                    }
                });

                Box::pin(combined_stream)
            }
        }
    };
}
