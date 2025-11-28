// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Shared state management for `combine_latest` operator.
//!
//! This module contains generic state management logic that is shared between
//! the ordered and unordered variants. Each variant implements its own
//! stream processing logic.

use futures::StreamExt;
use std::fmt::Debug;

/// Internal state for tracking the latest value from each stream.
///
/// This state maintains:
/// - Latest value from each stream (by stream index)
/// - Ordered values (stable ordering established at initialization)
/// - Mapping from stream index to position in ordered values
/// - Initialization flag
#[derive(Clone, Debug)]
pub struct IntermediateState<V>
where
    V: Clone + Send + Sync,
{
    state: Vec<Option<V>>,
    ordered_values: Vec<V>,
    stream_index_to_position: Vec<usize>,
    is_initialized: bool,
}

impl<V> IntermediateState<V>
where
    V: Clone + Send + Sync,
{
    pub fn new(num_streams: usize) -> Self {
        Self {
            state: vec![None; num_streams],
            ordered_values: Vec::new(),
            stream_index_to_position: vec![0; num_streams],
            is_initialized: false,
        }
    }

    pub const fn get_ordered_values(&self) -> &Vec<V> {
        &self.ordered_values
    }

    pub fn is_complete(&self) -> bool {
        self.state.iter().all(Option::is_some)
    }

    pub fn insert(&mut self, index: usize, value: V) {
        self.state[index] = Some(value.clone());

        if !self.is_initialized && self.is_complete() {
            // First complete state: establish the ordering
            let mut indexed_values: Vec<(usize, V)> = self
                .state
                .iter()
                .enumerate()
                .filter_map(|(i, opt)| opt.as_ref().map(|v| (i, v.clone())))
                .collect();

            // Sort by stream index to establish stable ordering
            indexed_values.sort_by_key(|(stream_idx, _)| *stream_idx);

            self.ordered_values = indexed_values.iter().map(|(_, v)| v.clone()).collect();

            for (position, (stream_idx, _)) in indexed_values.iter().enumerate() {
                self.stream_index_to_position[*stream_idx] = position;
            }

            self.is_initialized = true;
        } else if self.is_initialized {
            let position = self.stream_index_to_position[index];
            if position < self.ordered_values.len() {
                self.ordered_values[position] = value;
            }
        }
    }
}

/// Tags streams with their index for later identification.
///
/// This function converts a collection of streams into a vector of pinned streams,
/// where each stream item is paired with its original index (0 for `self`, 1+ for `others`).
pub fn tag_streams<T, S, IS>(
    self_stream: S,
    others: Vec<IS>,
) -> Vec<std::pin::Pin<Box<dyn futures::Stream<Item = (T, usize)> + Send + Sync>>>
where
    T: Send + Sync + 'static,
    S: futures::Stream<Item = T> + Send + Sync + 'static,
    IS: futures::Stream<Item = T> + Send + Sync + 'static,
{
    let mut streams = vec![];
    streams.push(Box::pin(self_stream.map(move |item| (item, 0_usize)))
        as std::pin::Pin<
            Box<dyn futures::Stream<Item = (T, usize)> + Send + Sync>,
        >);

    for (index, stream) in others.into_iter().enumerate() {
        let idx = index + 1;
        streams.push(Box::pin(stream.map(move |item| (item, idx))));
    }

    streams
}
