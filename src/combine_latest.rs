use futures::future::ready;
use futures::{Stream, StreamExt};
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use crate::select_all_ordered::SelectAllExt;

pub trait CombineLatestExt<V, S>: Stream<Item = V> + Sized
where
    V: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    S: Stream<Item = V> + Send + 'static,
{
    fn combine_latest(
        self,
        others: Vec<S>,
        filter: impl Fn(&CombinedState<V>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = CombinedState<V>> + Send;
}

type PinnedStreams<V> = Vec<Pin<Box<dyn Stream<Item = (V, usize)> + Send>>>;

impl<V, S> CombineLatestExt<V, S> for S
where
    V: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    S: Stream<Item = V> + Send + 'static,
{
    fn combine_latest(
        self,
        others: Vec<S>,
        filter: impl Fn(&CombinedState<V>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = CombinedState<V>> + Send {
        let mut streams: PinnedStreams<V> = vec![];

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
                        let mut state = state.lock().unwrap();
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
    V: Clone + Send + Sync + Ord,
{
    state: Vec<Option<V>>,
    ordered_values: Vec<V>,
}

impl<V> IntermediateState<V>
where
    V: Clone + Send + Sync + Ord,
{
    pub fn new(num_streams: usize) -> Self {
        Self {
            state: vec![None; num_streams],
            ordered_values: Vec::new(),
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

        // Rebuild ordered_values from current state, maintaining sort order
        self.ordered_values = self.state.iter().filter_map(|opt| opt.clone()).collect();
        self.ordered_values.sort();
    }
}
