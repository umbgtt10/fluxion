// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec;
use alloc::vec::Vec;

#[derive(Clone)]
pub struct IntermediateState<T> {
    values: Vec<Option<T>>,
}

impl<T: Clone> IntermediateState<T> {
    pub fn new(size: usize) -> Self {
        Self {
            values: vec![None; size],
        }
    }

    pub fn insert(&mut self, index: usize, value: T) {
        if index < self.values.len() {
            self.values[index] = Some(value);
        }
    }

    pub fn is_complete(&self) -> bool {
        self.values.iter().all(|v| v.is_some())
    }

    pub fn get_values(&self) -> Vec<T> {
        self.values
            .iter()
            .filter_map(|v| v.as_ref())
            .cloned()
            .collect()
    }
}

macro_rules! define_with_latest_from_impl {
    ($($bounds:tt)*) => {
        use super::implementation::IntermediateState;
        use crate::ordered_merge::ordered_merge_with_index;
        use crate::types::CombinedState;
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

        type PinnedStream<T> = Pin<Box<dyn Stream<Item = StreamItem<T>> + $($bounds)* 'static>>;

        /// Extension trait providing the `with_latest_from` operator for timestamped streams.
        pub trait WithLatestFromExt<T>: Stream<Item = StreamItem<T>> + Sized
        where
            T: Fluxion,
            T::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
            T::Timestamp: Debug + Ord + Copy + $($bounds)* 'static,
        {
            /// Combines elements from the primary stream (self) with the latest element from the secondary stream (other).
            fn with_latest_from<IS, R>(
                self,
                other: IS,
                result_selector: impl Fn(&CombinedState<T::Inner, T::Timestamp>) -> R + $($bounds)* 'static,
            ) -> impl Stream<Item = StreamItem<R>>
            where
                IS: IntoStream<Item = StreamItem<T>>,
                IS::Stream: $($bounds)* 'static,
                R: Fluxion,
                R::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
                R::Timestamp: From<T::Timestamp> + Debug + Ord + Copy + $($bounds)* 'static;
        }

        impl<T, S> WithLatestFromExt<T> for S
        where
            T: Fluxion,
            T::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
            T::Timestamp: Debug + Ord + Copy + $($bounds)* 'static,
            S: Stream<Item = StreamItem<T>> + Sized + Unpin + $($bounds)* 'static,
        {
            fn with_latest_from<IS, R>(
                self,
                other: IS,
                result_selector: impl Fn(&CombinedState<T::Inner, T::Timestamp>) -> R + $($bounds)* 'static,
            ) -> impl Stream<Item = StreamItem<R>>
            where
                IS: IntoStream<Item = StreamItem<T>>,
                IS::Stream: $($bounds)* 'static,
                R: Fluxion,
                R::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
                R::Timestamp: From<T::Timestamp> + Debug + Ord + Copy + $($bounds)* 'static,
            {
                let streams: Vec<PinnedStream<T>> = vec![Box::pin(self), Box::pin(other.into_stream())];

                let num_streams = streams.len();
                let state = Arc::new(Mutex::new(IntermediateState::new(num_streams)));
                let selector = Arc::new(result_selector);

                let combined_stream = ordered_merge_with_index(streams).filter_map({
                    let state = Arc::clone(&state);
                    let selector = Arc::clone(&selector);

                    move |(item, stream_index)| {
                        let state = Arc::clone(&state);
                        let selector = Arc::clone(&selector);

                        async move {
                            match item {
                                StreamItem::Value(value) => {
                                    let timestamp = value.timestamp();
                                    // Update state with new value
                                    let mut guard = state.lock();
                                    guard.insert(stream_index, value);

                                    // Only emit if:
                                    // 1. Both streams have emitted at least once (is_complete)
                                    // 2. The PRIMARY stream (index 0) triggered this emission
                                    if guard.is_complete() && stream_index == 0 {
                                        let values = guard.get_values();

                                        // values[0] = primary, values[1] = secondary
                                        let combined_state = CombinedState::new(
                                            vec![
                                                (values[0].clone().into_inner(), values[0].timestamp()),
                                                (values[1].clone().into_inner(), values[1].timestamp()),
                                            ],
                                            timestamp,
                                        );

                                        // Apply the result selector to transform the combined state
                                        let result = selector(&combined_state);

                                        // Return result directly (R implements Timestamped)
                                        Some(StreamItem::Value(result))
                                    } else {
                                        // Secondary stream emitted, just update state but don't emit
                                        None
                                    }
                                }
                                StreamItem::Error(e) => Some(StreamItem::Error(e)),
                            }
                        }
                    }
                });

                Box::pin(combined_stream)
            }
        }
    };
}
