// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::combine_latest::CombinedState;
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

/// Generic implementation of with_latest_from operator for timestamped streams.
///
/// This function combines a primary stream with a secondary stream, emitting only
/// when the primary stream emits, using the latest value from the secondary stream.
///
/// # Arguments
///
/// * `primary` - The primary stream that triggers emissions
/// * `secondary` - The secondary stream whose latest value is sampled
/// * `result_selector` - Function that transforms the combined state into output type
///
/// # Returns
///
/// A stream that emits when the primary stream emits, combining with latest from secondary.
pub fn with_latest_from_impl<S, T, IS, R, F>(
    primary: S,
    secondary: IS,
    result_selector: F,
) -> impl Stream<Item = StreamItem<R>> + Send + Sync + Unpin
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
    S: Stream<Item = StreamItem<T>> + Send + Sync + 'static,
    IS: IntoStream<Item = StreamItem<T>>,
    IS::Stream: Send + Sync + 'static,
    R: Fluxion,
    R::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    R::Timestamp: From<T::Timestamp> + Debug + Ord + Send + Sync + Copy + 'static,
    F: Fn(&CombinedState<T::Inner, T::Timestamp>) -> R + Send + Sync + 'static,
{
    let streams: Vec<Pin<Box<dyn Stream<Item = StreamItem<T>> + Send + Sync>>> =
        vec![Box::pin(primary), Box::pin(secondary.into_stream())];

    let num_streams = streams.len();
    let state = Arc::new(Mutex::new(IntermediateState::new(num_streams)));
    let selector = Arc::new(result_selector);

    Box::pin(ordered_merge_with_index(streams).filter_map({
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
    }))
}

#[derive(Clone)]
struct IntermediateState<T> {
    values: Vec<Option<T>>,
}

impl<T: Clone> IntermediateState<T> {
    fn new(size: usize) -> Self {
        Self {
            values: vec![None; size],
        }
    }

    fn insert(&mut self, index: usize, value: T) {
        if index < self.values.len() {
            self.values[index] = Some(value);
        }
    }

    fn is_complete(&self) -> bool {
        self.values.iter().all(|v| v.is_some())
    }

    fn get_values(&self) -> Vec<T> {
        self.values
            .iter()
            .filter_map(|v| v.as_ref())
            .cloned()
            .collect()
    }
}
