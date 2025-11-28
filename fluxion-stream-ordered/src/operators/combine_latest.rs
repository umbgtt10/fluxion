// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Ordered variant of `combine_latest` using `ordered_merge`.

use fluxion_core::{
    into_stream::IntoStream, lock_utilities::lock_or_recover, ComparableTimestamped, StreamItem,
    Timestamped,
};
use fluxion_ordered_merge::OrderedMergeExt;
use fluxion_stream::types::CombinedState;
use fluxion_stream_common::operators::combine_latest::{tag_streams, IntermediateState};
use futures::{future::ready, Stream, StreamExt};
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

/// Extension trait providing the ordered `combine_latest` operator.
///
/// This variant uses `ordered_merge` to preserve temporal ordering based on timestamps.
pub trait CombineLatestExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: ComparableTimestamped,
    T::Inner: Clone + Debug + Ord + Send + Sync + 'static,
    T::Timestamp: Clone + Debug + Ord + Send + Sync,
{
    /// Combines this stream with multiple other streams, emitting when any stream emits.
    ///
    /// This ordered variant uses `ordered_merge` to maintain temporal ordering based on
    /// timestamps. Events are processed in timestamp order to preserve causality.
    ///
    /// # Behavior
    ///
    /// - Waits until all streams have emitted at least one value
    /// - After initialization, emits whenever any stream produces a value
    /// - **Maintains temporal ordering** using `ordered_merge`
    /// - Allows filtering emissions based on the combined state
    ///
    /// # Arguments
    ///
    /// * `others` - Vector of streams to combine with this stream
    /// * `filter` - Predicate function that determines whether to emit a combined state
    ///
    /// # Returns
    ///
    /// A stream of timestamped `CombinedState<T::Inner, T::Timestamp>` preserving temporal order.
    fn combine_latest<IS>(
        self,
        others: Vec<IS>,
        filter: impl Fn(&CombinedState<T::Inner, T::Timestamp>) -> bool + Send + Sync + 'static,
    ) -> Pin<Box<dyn Stream<Item = StreamItem<CombinedState<T::Inner, T::Timestamp>>> + Send>>
    where
        IS: IntoStream<Item = StreamItem<T>>,
        IS::Stream: Send + Sync + 'static,
        CombinedState<T::Inner, T::Timestamp>:
            Timestamped<Inner = CombinedState<T::Inner, T::Timestamp>, Timestamp = T::Timestamp>;
}

type PinnedStreams<T> = Vec<Pin<Box<dyn Stream<Item = (StreamItem<T>, usize)> + Send + Sync>>>;

impl<T, S> CombineLatestExt<T> for S
where
    T: ComparableTimestamped,
    T::Inner: Clone + Debug + Ord + Send + Sync + 'static,
    T::Timestamp: Debug,
    S: Stream<Item = StreamItem<T>> + Send + Sync + 'static,
    CombinedState<T::Inner, T::Timestamp>:
        Timestamped<Inner = CombinedState<T::Inner, T::Timestamp>, Timestamp = T::Timestamp>,
{
    fn combine_latest<IS>(
        self,
        others: Vec<IS>,
        filter: impl Fn(&CombinedState<T::Inner, T::Timestamp>) -> bool + Send + Sync + 'static,
    ) -> Pin<Box<dyn Stream<Item = StreamItem<CombinedState<T::Inner, T::Timestamp>>> + Send>>
    where
        IS: IntoStream<Item = StreamItem<T>>,
        IS::Stream: Send + Sync + 'static,
        CombinedState<T::Inner, T::Timestamp>:
            Timestamped<Inner = CombinedState<T::Inner, T::Timestamp>, Timestamp = T::Timestamp>,
    {
        // Tag each stream with its index using shared helper
        let others_streams: Vec<_> = others.into_iter().map(|s| s.into_stream()).collect();
        let streams: PinnedStreams<T> = tag_streams(self, others_streams);
        let num_streams = streams.len();

        // ORDERED VARIANT: Use ordered_merge to preserve temporal ordering
        let merged = streams.ordered_merge();

        // Apply combine_latest state management using shared helper
        let state = Arc::new(Mutex::new((
            IntermediateState::new(num_streams),
            None::<T::Timestamp>,
        )));

        let state_stream = merged
            .filter_map({
                let state = Arc::clone(&state);
                move |(item, index)| {
                    let state = Arc::clone(&state);
                    async move {
                        match item {
                            StreamItem::Value(item) => {
                                let timestamp = item.timestamp();
                                let mut guard = lock_or_recover(&state, "combine_latest state");
                                guard.0.insert(index, item);
                                guard.1 = Some(timestamp);

                                if guard.0.is_complete() {
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
                item.map(|(state, timestamp)| {
                    // Extract the inner values to create CombinedState
                    let inner_values: Vec<T::Inner> = state
                        .get_ordered_values()
                        .iter()
                        .map(|ordered_val| ordered_val.clone().into_inner())
                        .collect();
                    let timestamp = timestamp.expect("State must have timestamp");
                    CombinedState::new(inner_values, timestamp)
                })
            })
            .filter(move |item| {
                match item {
                    StreamItem::Value(combined_state) => ready(filter(combined_state)),
                    StreamItem::Error(_) => ready(true), // Always emit errors
                }
            });

        Box::pin(state_stream)
    }
}
