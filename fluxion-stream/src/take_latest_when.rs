// Copyright 2025 Umberto Gotti
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::Ordered;
use crate::combine_latest::CombinedState;
use crate::util::safe_lock;
use fluxion_ordered_merge::OrderedMergeExt;
use futures::{Stream, StreamExt};
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

pub trait TakeLatestWhenExt<T, SF>: Stream<Item = T> + Sized
where
    T: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    SF: Stream<Item = T> + Send + Sync + 'static,
{
    fn take_latest_when(
        self,
        filter_stream: SF,
        filter: impl Fn(&CombinedState<T::Inner>) -> bool + Send + Sync + 'static,
    ) -> Pin<Box<dyn Stream<Item = T> + Send + Sync>>;
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
    const fn new() -> Self {
        Self {
            source_value: None,
            filter_value: None,
        }
    }

    const fn is_complete(&self) -> bool {
        self.source_value.is_some() && self.filter_value.is_some()
    }

    fn get_values(&self) -> Option<Vec<T>> {
        if let (Some(source), Some(filter)) = (&self.source_value, &self.filter_value) {
            Some(vec![source.clone(), filter.clone()])
        } else {
            None
        }
    }
}

type IndexedStream<T> = Pin<Box<dyn Stream<Item = (T, usize)> + Send + Sync>>;
impl<T, S, SF> TakeLatestWhenExt<T, SF> for S
where
    S: Stream<Item = T> + Send + Sync + 'static,
    SF: Stream<Item = T> + Send + Sync + 'static,
    T: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn take_latest_when(
        self,
        filter_stream: SF,
        filter: impl Fn(&CombinedState<T::Inner>) -> bool + Send + Sync + 'static,
    ) -> Pin<Box<dyn Stream<Item = T> + Send + Sync>> {
        let source_stream = Box::pin(self.map(|value| (value, 0)));
        let filter_stream = Box::pin(filter_stream.map(|value| (value, 1)));

        let streams: Vec<IndexedStream<T>> = vec![source_stream, filter_stream];

        let state = Arc::new(Mutex::new(TakeLatestState::new()));
        let filter = Arc::new(filter);

        Box::pin(
            streams
                .ordered_merge()
                .filter_map(move |(ordered_value, index)| {
                    let state = Arc::clone(&state);
                    let filter = Arc::clone(&filter);
                    let order = ordered_value.order();
                    async move {
                        let mut state = match safe_lock(&state, "take_latest_when state") {
                            Ok(lock) => lock,
                            Err(e) => {
                                error!("Failed to acquire lock in take_latest_when: {}", e);
                                return None;
                            }
                        };

                        match index {
                            0 => state.source_value = Some(ordered_value.get().clone()),
                            1 => state.filter_value = Some(ordered_value.get().clone()),
                            _ => {
                                warn!(
                                    "take_latest_when: unexpected stream index {} – ignoring",
                                    index
                                );
                            }
                        }

                        if state.is_complete() {
                            let values = state.get_values()?;
                            let combined_state = CombinedState::new(values);

                            if filter(&combined_state) {
                                state
                                    .source_value
                                    .clone()
                                    .map(|source| T::with_order(source, order))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                }),
        )
    }
}
