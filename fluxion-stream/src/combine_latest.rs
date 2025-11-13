use futures::future::ready;
use futures::{Stream, StreamExt};
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use crate::ordered::{Ordered, OrderedWrapper};
use fluxion_ordered_merge::OrderedMergeExt;

/// Trait for comparing ordered types by their inner values.
/// This is used by combine_latest to establish a stable ordering of streams.
pub trait CompareByInner {
    fn cmp_inner(&self, other: &Self) -> std::cmp::Ordering;
}

pub trait CombineLatestExt<T>: Stream<Item = T> + Sized
where
    T: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + CompareByInner + 'static,
    T::Inner: Clone + Debug + Ord + Send + Sync + 'static,
{
    fn combine_latest<S2>(
        self,
        others: Vec<S2>,
        filter: impl Fn(&CombinedState<T::Inner>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = OrderedWrapper<CombinedState<T::Inner>>> + Send + Unpin
    where
        S2: Stream<Item = T> + Send + Sync + 'static;
}

type PinnedStreams<T> = Vec<Pin<Box<dyn Stream<Item = (T, usize)> + Send>>>;

impl<T, S> CombineLatestExt<T> for S
where
    T: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + CompareByInner + 'static,
    T::Inner: Clone + Debug + Ord + Send + Sync + 'static,
    S: Stream<Item = T> + Send + Sync + 'static,
{
    fn combine_latest<S2>(
        self,
        others: Vec<S2>,
        filter: impl Fn(&CombinedState<T::Inner>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = OrderedWrapper<CombinedState<T::Inner>>> + Send + Unpin
    where
        S2: Stream<Item = T> + Send + Sync + 'static,
    {
        let mut streams: PinnedStreams<T> = vec![];

        streams.push(Box::pin(self.map(move |value| (value, 0))));
        for (index, stream) in others.into_iter().enumerate() {
            let idx = index + 1;
            streams.push(Box::pin(stream.map(move |value| (value, idx))));
        }

        let num_streams = streams.len();
        let state = Arc::new(Mutex::new(IntermediateState::new(num_streams)));

        Box::pin(
            streams
                .ordered_merge()
                .filter_map({
                    let state = Arc::clone(&state);

                    move |(value, index)| {
                        let state = Arc::clone(&state);
                        let order = value.order(); // Capture order of triggering value
                        async move {
                            let mut state = state
                                .lock()
                                .expect("Failed to acquire lock on combine_latest state");
                            state.insert(index, value);

                            if state.is_complete() {
                                Some((state.clone(), order))
                            } else {
                                None
                            }
                        }
                    }
                })
                .map(|(state, order)| {
                    // Extract the inner values to create CombinedState
                    let inner_values: Vec<T::Inner> = state
                        .get_ordered_values()
                        .iter()
                        .map(|ordered_val| ordered_val.get().clone())
                        .collect();
                    let combined = CombinedState::new(inner_values);
                    crate::ordered::OrderedWrapper::with_order(combined, order)
                })
                .filter(move |ordered_combined| {
                    let combined_state = ordered_combined.get();
                    ready(filter(combined_state))
                }),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CombinedState<V>
where
    V: Clone + Send + Sync,
{
    state: Vec<V>,
}

impl<V> PartialOrd for CombinedState<V>
where
    V: Clone + Send + Sync + Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<V> Ord for CombinedState<V>
where
    V: Clone + Send + Sync + Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.state.cmp(&other.state)
    }
}

impl<V> CombinedState<V>
where
    V: Clone + Send + Sync,
{
    pub fn new(state: Vec<V>) -> Self {
        Self { state }
    }

    pub fn get_state(&self) -> &Vec<V> {
        &self.state
    }
}

impl<V> Ordered for CombinedState<V>
where
    V: Clone + Send + Sync,
{
    type Inner = CombinedState<V>;

    fn order(&self) -> u64 {
        // CombinedState doesn't have its own order - it's always wrapped by an Ordered type
        // This should never be called directly
        0
    }

    fn get(&self) -> &Self::Inner {
        self
    }

    fn with_order(value: Self::Inner, _order: u64) -> Self {
        value
    }

    fn into_inner(self) -> Self::Inner {
        self
    }
}

#[derive(Clone, Debug)]
struct IntermediateState<V>
where
    V: Clone + Send + Sync + Ord + CompareByInner,
{
    state: Vec<Option<V>>,
    ordered_values: Vec<V>,
    stream_index_to_position: Vec<usize>,
    is_initialized: bool,
}

impl<V> IntermediateState<V>
where
    V: Clone + Send + Sync + Ord + CompareByInner,
{
    pub fn new(num_streams: usize) -> Self {
        Self {
            state: vec![None; num_streams],
            ordered_values: Vec::new(),
            stream_index_to_position: vec![0; num_streams],
            is_initialized: false,
        }
    }

    pub fn get_ordered_values(&self) -> &Vec<V> {
        &self.ordered_values
    }

    pub fn is_complete(&self) -> bool {
        self.state.iter().all(|entry| entry.is_some())
    }

    pub fn insert(&mut self, index: usize, value: V) {
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

            // Sort by inner value to establish order by enum variant type
            indexed_values.sort_by(|a, b| a.1.cmp_inner(&b.1));

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
