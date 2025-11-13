use futures::{Stream, StreamExt};
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use crate::combine_latest::CombinedState;
use crate::ordered_merge::OrderedMergeExt;
use crate::sequenced::Sequenced;
use crate::sequenced_stream::SequencedStreamExt;

pub trait TakeLatestWhenExt<T, SF>: SequencedStreamExt<T> + Sized
where
    T: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    SF: Stream<Item = Sequenced<T>> + Send + Sync + 'static,
{
    fn take_latest_when(
        self,
        filter_stream: SF,
        filter: impl Fn(&CombinedState<T>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = T>;
}

#[derive(Clone, Debug)]
struct TakeLatestState<T>
where
    T: Clone,
{
    source_value: Option<T>,
    filter_value: Option<T>,
}

impl<T> TakeLatestState<T>
where
    T: Clone,
{
    fn new() -> Self {
        Self {
            source_value: None,
            filter_value: None,
        }
    }

    fn is_complete(&self) -> bool {
        self.source_value.is_some() && self.filter_value.is_some()
    }

    fn get_values(&self) -> Vec<T> {
        vec![
            self.source_value
                .clone()
                .expect("source_value should be set when state is complete"),
            self.filter_value
                .clone()
                .expect("filter_value should be set when state is complete"),
        ]
    }
}

type IndexedStream<T> = Pin<Box<dyn Stream<Item = (Sequenced<T>, usize)> + Send>>;

impl<T, S, SF> TakeLatestWhenExt<T, SF> for S
where
    S: SequencedStreamExt<T> + Send + Sync + 'static,
    SF: Stream<Item = Sequenced<T>> + Send + Sync + 'static,
    T: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn take_latest_when(
        self,
        filter_stream: SF,
        filter: impl Fn(&CombinedState<T>) -> bool + Send + Sync + 'static,
    ) -> impl futures::Stream<Item = T> {
        let source_stream = Box::pin(self.map(|value| (value, 0)));
        let filter_stream = Box::pin(filter_stream.map(|value| (value, 1)));

        let streams: Vec<IndexedStream<T>> = vec![source_stream, filter_stream];

        let state = Arc::new(Mutex::new(TakeLatestState::new()));
        let filter = Arc::new(filter);

        streams
            .ordered_merge()
            .filter_map(move |(timestamped_value, index)| {
                let state = Arc::clone(&state);
                let filter = Arc::clone(&filter);
                async move {
                    let mut state = state
                        .lock()
                        .expect("Failed to acquire lock on take_latest_when state");

                    match index {
                        0 => state.source_value = Some(timestamped_value.value.clone()),
                        1 => state.filter_value = Some(timestamped_value.value.clone()),
                        _ => unreachable!(),
                    }

                    if state.is_complete() {
                        let values = state.get_values();
                        let combined_state = CombinedState::new(values);

                        if filter(&combined_state) {
                            Some(
                                state
                                    .source_value
                                    .clone()
                                    .expect("source_value should be set when state is complete"),
                            )
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            })
    }
}
