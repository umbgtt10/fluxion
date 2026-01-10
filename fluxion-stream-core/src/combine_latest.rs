// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Generic implementation of combine_latest operator - runtime-agnostic

use crate::ordered_merge::ordered_merge_with_index;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Debug;
use core::pin::Pin;
use fluxion_core::fluxion_mutex::Mutex;
use fluxion_core::into_stream::IntoStream;
use fluxion_core::{Fluxion, HasTimestamp, StreamItem, Timestamped};
use futures::future::ready;
use futures::{Stream, StreamExt};

/// State container holding the latest values from multiple combined streams.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CombinedState<V, TS = u64>
where
    V: Clone + Debug + Ord,
    TS: Clone + Debug + Ord,
{
    state: Vec<(V, TS)>,
    timestamp: TS,
}

impl<V, TS> CombinedState<V, TS>
where
    V: Clone + Debug + Ord,
    TS: Clone + Debug + Ord,
{
    pub fn new(state: Vec<(V, TS)>, timestamp: TS) -> Self {
        Self { state, timestamp }
    }

    pub fn values(&self) -> Vec<V> {
        self.state.iter().map(|(v, _)| v.clone()).collect()
    }

    pub fn timestamps(&self) -> Vec<TS> {
        self.state.iter().map(|(_, ts)| ts.clone()).collect()
    }

    pub fn pairs(&self) -> &[(V, TS)] {
        &self.state
    }

    pub fn len(&self) -> usize {
        self.state.len()
    }

    pub fn is_empty(&self) -> bool {
        self.state.is_empty()
    }
}

impl<V, TS> HasTimestamp for CombinedState<V, TS>
where
    V: Clone + Debug + Ord,
    TS: Clone + Debug + Ord + Copy + Send + Sync,
{
    type Timestamp = TS;

    fn timestamp(&self) -> Self::Timestamp {
        self.timestamp
    }
}

impl<V, TS> Timestamped for CombinedState<V, TS>
where
    V: Clone + Debug + Ord,
    TS: Clone + Debug + Ord + Copy + Send + Sync,
{
    type Inner = Self;

    fn with_timestamp(value: Self::Inner, timestamp: Self::Timestamp) -> Self {
        Self {
            state: value.state,
            timestamp,
        }
    }

    fn into_inner(self) -> Self::Inner {
        self
    }
}

/// Generic implementation of combine_latest - works with any runtime
///
/// This implementation is completely runtime-agnostic. Runtime-specific wrappers
/// (in fluxion-stream-multi and fluxion-stream-single) provide the trait definitions
/// with appropriate bounds.
///
/// Note: While this requires Send+Sync internally, single-threaded runtimes can still
/// use this because their streams ARE Send (just not used across threads).
pub fn combine_latest_impl<S, T, IS, F>(
    primary_stream: S,
    others: Vec<IS>,
    filter: F,
) -> impl Stream<Item = StreamItem<CombinedState<T::Inner, T::Timestamp>>> + Send + Sync + Unpin
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Clone + Debug + Ord + Send + Sync,
    S: Stream<Item = StreamItem<T>> + Send + Sync + 'static,
    IS: IntoStream<Item = StreamItem<T>>,
    IS::Stream: Send + Sync + 'static,
    F: Fn(&CombinedState<T::Inner, T::Timestamp>) -> bool + Send + Sync + 'static,
    CombinedState<T::Inner, T::Timestamp>:
        Timestamped<Inner = CombinedState<T::Inner, T::Timestamp>, Timestamp = T::Timestamp>,
{
    type PinnedStreams<T> = Vec<Pin<Box<dyn Stream<Item = StreamItem<T>> + Send + Sync>>>;

    let mut streams: PinnedStreams<T> = vec![];

    streams.push(Box::pin(primary_stream));
    for into_stream in others {
        let stream = into_stream.into_stream();
        streams.push(Box::pin(stream));
    }

    let num_streams = streams.len();
    let state = Arc::new(Mutex::new(IntermediateState::<T>::new(num_streams)));

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
                    .map(|ordered_val| (ordered_val.clone().into_inner(), ordered_val.timestamp()))
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

#[derive(Clone, Debug)]
struct IntermediateState<V>
where
    V: Clone + Send + Sync + Ord + Timestamped,
{
    state: Vec<Option<V>>,
    ordered_values: Vec<V>,
    stream_index_to_position: Vec<usize>,
    is_initialized: bool,
    last_timestamp: Option<V::Timestamp>,
}

impl<V> IntermediateState<V>
where
    V: Clone + Send + Sync + Ord + Timestamped,
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
        self.state.iter().all(|opt| opt.is_some())
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
