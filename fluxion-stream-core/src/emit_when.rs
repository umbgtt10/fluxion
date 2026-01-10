// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::combine_latest::CombinedState;
use crate::ordered_merge::ordered_merge_with_index;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Debug;
use core::pin::Pin;
use fluxion_core::fluxion_mutex::Mutex;
use fluxion_core::into_stream::IntoStream;
use fluxion_core::{Fluxion, StreamItem};
use futures::{Stream, StreamExt};

type SharedState<V, TS> = Arc<Mutex<Option<(V, TS)>>>;

/// Generic implementation of emit_when operator for timestamped streams.
///
/// This function gates a source stream based on conditions from a filter stream,
/// emitting source values only when the combined state passes a predicate.
///
/// # Arguments
///
/// * `source` - The source stream to gate
/// * `filter_stream` - The filter stream that controls gating
/// * `filter` - Predicate function that determines whether to emit
///
/// # Returns
///
/// A stream that emits source values only when the filter condition is satisfied.
pub fn emit_when_impl<S, T, IS, F>(
    source: S,
    filter_stream: IS,
    filter: F,
) -> impl Stream<Item = StreamItem<T>> + Send + Sync + Unpin
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
    S: Stream<Item = StreamItem<T>> + Send + Sync + 'static,
    IS: IntoStream<Item = StreamItem<T>>,
    IS::Stream: Send + Sync + 'static,
    F: Fn(&CombinedState<T::Inner, T::Timestamp>) -> bool + Send + Sync + 'static,
{
    let streams: Vec<Pin<Box<dyn Stream<Item = StreamItem<T>> + Send + Sync>>> =
        vec![Box::pin(source), Box::pin(filter_stream.into_stream())];

    let source_value: SharedState<T::Inner, T::Timestamp> = Arc::new(Mutex::new(None));
    let filter_value: SharedState<T::Inner, T::Timestamp> = Arc::new(Mutex::new(None));
    let filter = Arc::new(filter);

    Box::pin(
        ordered_merge_with_index(streams).filter_map(move |(item, index)| {
            let source_value = Arc::clone(&source_value);
            let filter_value = Arc::clone(&filter_value);
            let filter = Arc::clone(&filter);
            async move {
                match item {
                    StreamItem::Value(ordered_value) => {
                        match index {
                            0 => {
                                // Source stream update
                                let mut source = source_value.lock();
                                let filter_val = filter_value.lock();

                                let timestamp = ordered_value.timestamp();
                                *source = Some((ordered_value.clone().into_inner(), timestamp));

                                if let Some((src, src_ts)) = source.as_ref() {
                                    if let Some((filt, filt_ts)) = filter_val.as_ref() {
                                        let combined_state = CombinedState::new(
                                            vec![(src.clone(), *src_ts), (filt.clone(), *filt_ts)],
                                            timestamp,
                                        );
                                        if filter(&combined_state) {
                                            Some(StreamItem::Value(T::with_timestamp(
                                                src.clone(),
                                                *src_ts,
                                            )))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            }
                            1 => {
                                // Filter stream update
                                let mut filter_val = filter_value.lock();
                                let source = source_value.lock();

                                let timestamp = ordered_value.timestamp();
                                *filter_val = Some((ordered_value.clone().into_inner(), timestamp));

                                if let Some((src, src_ts)) = source.as_ref() {
                                    if let Some((filt, filt_ts)) = filter_val.as_ref() {
                                        let combined_state = CombinedState::new(
                                            vec![(src.clone(), *src_ts), (filt.clone(), *filt_ts)],
                                            timestamp,
                                        );
                                        if filter(&combined_state) {
                                            Some(StreamItem::Value(T::with_timestamp(
                                                src.clone(),
                                                *filt_ts,
                                            )))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        }
                    }
                    StreamItem::Error(e) => Some(StreamItem::Error(e)),
                }
            }
        }),
    )
}
