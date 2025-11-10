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

type PinnedStreams<V> = Vec<Pin<Box<dyn Stream<Item = (usize, V)> + Send>>>;

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

        streams.push(Box::pin(self.map(move |value| (0, value))));
        for (index, stream) in others.into_iter().enumerate() {
            streams.push(Box::pin(stream.map(move |value| (index + 1, value))));
        }

        let num_streams = streams.len();
        let state = Arc::new(Mutex::new(IntermediateState::new(num_streams)));

        streams
            .select_all_ordered()
            .filter_map({
                let state = Arc::clone(&state);

                move |(index, value)| {
                    let state = Arc::clone(&state);
                    async move {
                        let mut state = state.lock().unwrap();
                        state.insert(index, value);

                        println!("State: {:?}", state.get_state());
                        if state.is_complete() {
                            Some(state.clone())
                        } else {
                            None
                        }
                    }
                }
            })
            .map(|state| {
                let state = state.clone();
                let combined = CombinedState::new(
                    state
                        .get_state()
                        .iter()
                        .map(|entry| entry.clone().unwrap())
                        .collect(),
                );
                println!("CombinedState emitted: {:?}", combined.get_state());
                combined
            })
            .filter(move |combined_state| {
                println!("Combined state before filtering: {:?}", combined_state);
                let filtered = filter(combined_state);
                println!("Filtering successful: {}", filtered);
                ready(filtered)
            })
    }
}

#[derive(Clone, Debug)]
pub struct CombinedState<V>
where
    V: Clone + Send + Sync,
{
    state: Vec<V>,
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
    V: Clone + Send + Sync,
{
    state: Vec<Option<V>>,
}

impl<V> IntermediateState<V>
where
    V: Clone + Send + Sync,
{
    pub fn new(num_streams: usize) -> Self {
        Self {
            state: vec![None; num_streams],
        }
    }

    pub fn get_state(&self) -> &Vec<Option<V>> {
        &self.state
    }

    pub fn is_complete(&self) -> bool {
        self.state.iter().all(|entry| entry.is_some())
    }

    pub fn insert(&mut self, index: usize, value: V) {
        self.state[index] = Some(value);
    }
}
