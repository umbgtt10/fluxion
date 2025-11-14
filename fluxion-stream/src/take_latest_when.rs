// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::Ordered;
use fluxion_core::into_stream::IntoStream;
use fluxion_core::lock_utilities::safe_lock;
use fluxion_ordered_merge::OrderedMergeExt;
use futures::{Stream, StreamExt};
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

pub trait TakeLatestWhenExt<T>: Stream<Item = T> + Sized
where
    T: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn take_latest_when<IS>(
        self,
        filter_stream: IS,
        filter: impl Fn(&T::Inner) -> bool + Send + Sync + 'static,
    ) -> Pin<Box<dyn Stream<Item = T> + Send + Sync>>
    where
        IS: IntoStream<Item = T>,
        IS::Stream: Send + Sync + 'static;
}

type IndexedStream<T> = Pin<Box<dyn Stream<Item = (T, usize)> + Send + Sync>>;
impl<T, S> TakeLatestWhenExt<T> for S
where
    S: Stream<Item = T> + Send + Sync + 'static,
    T: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn take_latest_when<IS>(
        self,
        filter_stream: IS,
        filter: impl Fn(&T::Inner) -> bool + Send + Sync + 'static,
    ) -> Pin<Box<dyn Stream<Item = T> + Send + Sync>>
    where
        IS: IntoStream<Item = T>,
        IS::Stream: Send + Sync + 'static,
    {
        let source_stream = Box::pin(self.map(|value| (value, 0)));
        let filter_stream = Box::pin(filter_stream.into_stream().map(|value| (value, 1)));

        let streams: Vec<IndexedStream<T>> = vec![source_stream, filter_stream];

        let source_value = Arc::new(Mutex::new(None));
        let filter_value = Arc::new(Mutex::new(None));
        let filter = Arc::new(filter);

        Box::pin(
            streams
                .ordered_merge()
                .filter_map(move |(ordered_value, index)| {
                    let source_value = Arc::clone(&source_value);
                    let filter_value = Arc::clone(&filter_value);
                    let filter = Arc::clone(&filter);
                    let order = ordered_value.order();
                    async move {
                        match index {
                            0 => {
                                let mut source = match safe_lock(&source_value, "take_latest_when source") {
                                    Ok(lock) => lock,
                                    Err(e) => {
                                        error!("Failed to acquire lock in take_latest_when: {}", e);
                                        return None;
                                    }
                                };
                                *source = Some(ordered_value.get().clone());
                            }
                            1 => {
                                let mut filter_val = match safe_lock(&filter_value, "take_latest_when filter") {
                                    Ok(lock) => lock,
                                    Err(e) => {
                                        error!("Failed to acquire lock in take_latest_when: {}", e);
                                        return None;
                                    }
                                };
                                *filter_val = Some(ordered_value.get().clone());
                            }
                            _ => {
                                warn!(
                                    "take_latest_when: unexpected stream index {} – ignoring",
                                    index
                                );
                            }
                        }

                        let source = match safe_lock(&source_value, "take_latest_when source") {
                            Ok(lock) => lock,
                            Err(e) => {
                                error!("Failed to acquire lock in take_latest_when: {}", e);
                                return None;
                            }
                        };
                        let filter_val = match safe_lock(&filter_value, "take_latest_when filter") {
                            Ok(lock) => lock,
                            Err(e) => {
                                error!("Failed to acquire lock in take_latest_when: {}", e);
                                return None;
                            }
                        };

                        if let (Some(src), Some(filt)) = (source.as_ref(), filter_val.as_ref()) {
                            if filter(filt) {
                                Some(T::with_order(src.clone(), order))
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
