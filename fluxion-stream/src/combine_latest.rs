use futures::future::ready;
use futures::{Stream, StreamExt};
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use crate::select_all_ordered::SelectAllExt;
use crate::timestamped::Timestamped;
use crate::timestamped_stream::TimestampedStreamExt;

pub trait CompareByInner {
    fn cmp_inner(&self, other: &Self) -> std::cmp::Ordering;
}

impl<T: Ord> CompareByInner for Timestamped<T> {
    fn cmp_inner(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

pub trait CombineLatestExt<T, S>: TimestampedStreamExt<T> + Sized
where
    Timestamped<T>: Clone + Debug + Ord + Send + Sync + Unpin + CompareByInner + 'static,
    S: Stream<Item = Timestamped<T>> + Send + 'static,
{
    fn combine_latest(
        self,
        others: Vec<S>,
        filter: impl Fn(&CombinedState<Timestamped<T>>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = CombinedState<Timestamped<T>>> + Send;
}

type PinnedStreams<T> = Vec<Pin<Box<dyn Stream<Item = (Timestamped<T>, usize)> + Send>>>;

impl<T, S> CombineLatestExt<T, S> for S
where
    Timestamped<T>: Clone + Debug + Ord + Send + Sync + Unpin + CompareByInner + 'static,
    S: TimestampedStreamExt<T> + Send + 'static,
{
    fn combine_latest(
        self,
        others: Vec<S>,
        filter: impl Fn(&CombinedState<Timestamped<T>>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = CombinedState<Timestamped<T>>> + Send {
        let mut streams: PinnedStreams<T> = vec![];

        streams.push(Box::pin(self.map(move |value| (value, 0))));
        for (index, stream) in others.into_iter().enumerate() {
            let idx = index + 1;
            streams.push(Box::pin(stream.map(move |value| (value, idx))));
        }

        let num_streams = streams.len();
        let state = Arc::new(Mutex::new(IntermediateState::new(num_streams)));

        streams
            .select_all_ordered()
            .filter_map({
                let state = Arc::clone(&state);

                move |(value, index)| {
                    let state = Arc::clone(&state);
                    async move {
                        let mut state = state
                            .lock()
                            .expect("Failed to acquire lock on combine_latest state");
                        state.insert(index, value);

                        if state.is_complete() {
                            Some(state.clone())
                        } else {
                            None
                        }
                    }
                }
            })
            .map(|state| CombinedState::new(state.get_ordered_values().clone()))
            .filter(move |combined_state| ready(filter(combined_state)))
    }
}

#[derive(Clone, Debug)]
pub struct CombinedState<V>
where
    V: Clone + Send + Sync,
{
    state: Vec<V>, // Temporal order (sorted by Ord for Sequenced<T>)
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

#[derive(Clone, Debug)]
struct IntermediateState<V>
where
    V: Clone + Send + Sync + Ord + CompareByInner,
{
    state: Vec<Option<V>>,
    ordered_values: Vec<V>,
    stream_index_to_position: Vec<usize>, // Maps stream index to position in ordered_values
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
