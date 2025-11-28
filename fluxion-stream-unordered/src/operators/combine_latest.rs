// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Unordered variant of `combine_latest` using `select_all`.

use fluxion_core::lock_utilities::lock_or_recover;
use fluxion_core::{into_stream::IntoStream, StreamItem};
use fluxion_stream_common::operators::combine_latest::{tag_streams, IntermediateState};
use futures::{future::ready, Stream, StreamExt};
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

/// Extension trait providing the unordered `combine_latest` operator.
///
/// This variant uses `select_all` for arrival-order processing, providing 2-3x
/// speedup compared to the ordered variant. Events are processed as they arrive,
/// without timestamp-based ordering.
pub trait CombineLatestExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Clone + Debug + Send + Sync + 'static,
{
    /// Combines this stream with multiple other streams, emitting when any stream emits.
    ///
    /// This unordered variant uses `select_all` for arrival-order processing. Events are
    /// processed as they arrive, prioritizing performance over temporal causality.
    ///
    /// # Behavior
    ///
    /// - Waits until all streams have emitted at least one value
    /// - After initialization, emits whenever any stream produces a value
    /// - **Processes events in arrival order** (no timestamp ordering)
    /// - 2-3x faster than ordered variant
    /// - Allows filtering emissions based on the combined state
    ///
    /// # Arguments
    ///
    /// * `others` - Vector of streams to combine with this stream
    /// * `filter` - Predicate function that determines whether to emit a combined state
    ///
    /// # Returns
    ///
    /// A stream of combined values as `Vec<T>` processed in arrival order.
    fn combine_latest<IS>(
        self,
        others: Vec<IS>,
        filter: impl Fn(&Vec<T>) -> bool + Send + Sync + 'static,
    ) -> Pin<Box<dyn Stream<Item = StreamItem<Vec<T>>> + Send>>
    where
        IS: IntoStream<Item = StreamItem<T>>,
        IS::Stream: Send + Sync + 'static;
}

type PinnedStreams<T> = Vec<Pin<Box<dyn Stream<Item = (StreamItem<T>, usize)> + Send + Sync>>>;

impl<T, S> CombineLatestExt<T> for S
where
    T: Clone + Debug + Send + Sync + 'static,
    S: Stream<Item = StreamItem<T>> + Send + Sync + 'static,
{
    fn combine_latest<IS>(
        self,
        others: Vec<IS>,
        filter: impl Fn(&Vec<T>) -> bool + Send + Sync + 'static,
    ) -> Pin<Box<dyn Stream<Item = StreamItem<Vec<T>>> + Send>>
    where
        IS: IntoStream<Item = StreamItem<T>>,
        IS::Stream: Send + Sync + 'static,
    {
        // Tag each stream with its index using shared helper
        let others_streams: Vec<_> = others.into_iter().map(|s| s.into_stream()).collect();
        let streams: PinnedStreams<T> = tag_streams(self, others_streams);
        let num_streams = streams.len();

        // UNORDERED VARIANT: Use select_all for arrival-order processing
        let merged = futures::stream::select_all(streams);

        // Apply combine_latest state management using shared helper
        let state = Arc::new(Mutex::new(IntermediateState::new(num_streams)));

        let state_stream = merged.filter_map({
            let state = Arc::clone(&state);
            move |(item, index)| {
                let state = Arc::clone(&state);
                async move {
                    match item {
                        StreamItem::Value(item) => {
                            let mut guard = lock_or_recover(&state, "combine_latest state");
                            guard.insert(index, item);

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
        });

        // Convert to Vec<T> and filter
        let combined_stream = state_stream
            .map(|item| {
                item.map(|state| {
                    // Extract the ordered values
                    state.get_ordered_values().clone()
                })
            })
            .filter(move |item| {
                match item {
                    StreamItem::Value(combined_values) => ready(filter(combined_values)),
                    StreamItem::Error(_) => ready(true), // Always emit errors
                }
            });

        Box::pin(combined_stream)
    }
}
