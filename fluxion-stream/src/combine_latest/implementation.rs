// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

/// Macro that generates the complete combine_latest implementation.
///
/// This macro eliminates duplication between multi-threaded and single-threaded
/// implementations, which differ only in trait bounds (Send + Sync vs not).
macro_rules! define_combine_latest_impl {
    (
        inner_bounds: [$($inner_bounds:tt)*],
        timestamp_bounds: [$($timestamp_bounds:tt)*],
        stream_bounds: [$($stream_bounds:tt)*],
        state_bounds: [$($state_bounds:tt)*],
        boxed_stream: [$($boxed_stream:tt)*]
    ) => {
        use $crate::ordered_merge::ordered_merge_with_index;
        use $crate::types::CombinedState;
        use alloc::boxed::Box;
        use alloc::sync::Arc;
        use alloc::vec;
        use alloc::vec::Vec;
        use core::fmt::Debug;
        use core::pin::Pin;
        use fluxion_core::fluxion_mutex::Mutex;
        use fluxion_core::into_stream::IntoStream;
        use fluxion_core::{Fluxion, StreamItem, Timestamped};
        use futures::future::ready;
        use futures::{Stream, StreamExt};

        type PinnedStreams<T> = Vec<$($boxed_stream)*>;

        pub trait CombineLatestExt<T>: Stream<Item = StreamItem<T>> + Sized
        where
            T: Fluxion,
            T::Inner: Clone + Debug + Ord $($inner_bounds)* + 'static,
            T::Timestamp: Clone + Debug + Ord $($timestamp_bounds)*,
        {
            fn combine_latest<IS>(
                self,
                others: Vec<IS>,
                filter: impl Fn(&CombinedState<T::Inner, T::Timestamp>) -> bool + $($stream_bounds)* 'static,
            ) -> impl Stream<Item = StreamItem<CombinedState<T::Inner, T::Timestamp>>> + Unpin
            where
                IS: IntoStream<Item = StreamItem<T>>,
                IS::Stream: Stream<Item = StreamItem<T>> + $($stream_bounds)* 'static,
                CombinedState<T::Inner, T::Timestamp>:
                    Timestamped<Inner = CombinedState<T::Inner, T::Timestamp>, Timestamp = T::Timestamp>
                    $($state_bounds)*;
        }

        impl<T, S> CombineLatestExt<T> for S
        where
            T: Fluxion,
            T::Inner: Clone + Debug + Ord $($inner_bounds)* + 'static,
            T::Timestamp: Clone + Debug + Ord $($timestamp_bounds)*,
            S: Stream<Item = StreamItem<T>> + $($stream_bounds)* 'static,
            CombinedState<T::Inner, T::Timestamp>: Timestamped<Inner = CombinedState<T::Inner, T::Timestamp>, Timestamp = T::Timestamp>
                $($state_bounds)*,
        {
            fn combine_latest<IS>(
                self,
                others: Vec<IS>,
                filter: impl Fn(&CombinedState<T::Inner, T::Timestamp>) -> bool + $($stream_bounds)* 'static,
            ) -> impl Stream<Item = StreamItem<CombinedState<T::Inner, T::Timestamp>>> + Unpin
            where
                IS: IntoStream<Item = StreamItem<T>>,
                IS::Stream: Stream<Item = StreamItem<T>> + $($stream_bounds)* 'static,
                CombinedState<T::Inner, T::Timestamp>:
                    Timestamped<Inner = CombinedState<T::Inner, T::Timestamp>, Timestamp = T::Timestamp>,
            {
                use StreamItem;
                let mut streams: PinnedStreams<T> = vec![];

                streams.push(Box::pin(self));
                for into_stream in others {
                    let stream = into_stream.into_stream();
                    streams.push(Box::pin(stream));
                }

                let num_streams = streams.len();
                let state = Arc::new(Mutex::new(IntermediateState::new(num_streams)));

                let combined_stream = ordered_merge_with_index(streams)
                    .filter_map({
                        let state = Arc::clone(&state);

                        move |(item, index)| {
                            let state = Arc::clone(&state);
                            async move {
                                match item {
                                    StreamItem::Value(value) => {
                                        let mut guard = state.lock();
                                        guard.insert(index, value);

                                        if guard.is_complete() {
                                            Some(StreamItem::Value(guard.clone()))
                                        } else {
                                            None
                                        }
                                    }
                                    StreamItem::Error(e) => {
                                        // Propagate upstream errors immediately without state update
                                        Some(StreamItem::Error(e))
                                    }
                                }
                            }
                        }
                    })
                    .map(|item| {
                        item.map(|state| {
                            // Extract the inner values with their timestamps to create CombinedState
                            let value_timestamp_pairs: Vec<(T::Inner, T::Timestamp)> = state
                                .get_ordered_values()
                                .iter()
                                .map(|ordered_val| {
                                    (ordered_val.clone().into_inner(), ordered_val.timestamp())
                                })
                                .collect();
                            let timestamp = state.last_timestamp().expect("State must have timestamp");
                            CombinedState::new(value_timestamp_pairs, timestamp)
                        })
                    })
                    .filter(move |item| {
                        match item {
                            StreamItem::Value(combined_state) => ready(filter(combined_state)),
                            StreamItem::Error(_) => ready(true), // Always emit errors
                        }
                    });

                Box::pin(combined_stream)
            }
        }

        #[derive(Clone, Debug)]
        struct IntermediateState<V>
        where
            V: Clone + Ord + Timestamped $($state_bounds)*,
        {
            state: Vec<Option<V>>,
            ordered_values: Vec<V>,
            stream_index_to_position: Vec<usize>,
            is_initialized: bool,
            last_timestamp: Option<V::Timestamp>,
        }

        impl<V> IntermediateState<V>
        where
            V: Clone + Ord + Timestamped $($state_bounds)*,
        {
            pub fn new(num_streams: usize) -> Self {
                Self {
                    state: vec![None; num_streams],
                    ordered_values: Vec::new(),
                    stream_index_to_position: vec![0; num_streams],
                    is_initialized: false,
                    last_timestamp: None,
                }
            }

            pub const fn get_ordered_values(&self) -> &Vec<V> {
                &self.ordered_values
            }

            pub fn last_timestamp(&self) -> Option<V::Timestamp> {
                self.last_timestamp
            }

            pub fn is_complete(&self) -> bool {
                self.state.iter().all(Option::is_some)
            }

            pub fn insert(&mut self, index: usize, value: V) {
                self.last_timestamp = Some(value.timestamp());
                self.state[index] = Some(value.clone());

                if !self.is_initialized && self.is_complete() {
                    // First complete state: establish the ordering
                    // Collect all values with their stream indices
                    let mut indexed_values: Vec<(usize, V)> = self
                        .state
                        .iter()
                        .enumerate()
                        .filter_map(|(i, opt)| opt.as_ref().map(|v| (i, v.clone())))
                        .collect();

                    // Sort by stream index to establish stable ordering
                    indexed_values.sort_by_key(|(stream_idx, _)| *stream_idx);

                    // Build the ordered_values and the mapping
                    self.ordered_values = indexed_values.iter().map(|(_, v)| v.clone()).collect();

                    // Build stream_index_to_position: for each stream index, record its position
                    for (position, (stream_idx, _)) in indexed_values.iter().enumerate() {
                        self.stream_index_to_position[*stream_idx] = position;
                    }

                    self.is_initialized = true;
                } else if self.is_initialized {
                    // After initialization: update the value at its established position
                    let position = self.stream_index_to_position[index];
                    if position < self.ordered_values.len() {
                        self.ordered_values[position] = value;
                    }
                }
            }
        }
    }
}
