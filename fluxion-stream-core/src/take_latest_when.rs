// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Generic implementation of `take_latest_when` operator.
//!
//! Samples the latest value from a source stream whenever a filter stream emits
//! a value that passes a predicate.

use crate::ordered_merge::ordered_merge_with_index;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Debug;
use core::pin::Pin;
use fluxion_core::fluxion_mutex::Mutex;
use fluxion_core::{Fluxion, StreamItem};
use futures::{Stream, StreamExt};

/// Generic implementation of take_latest_when for multi-threaded runtimes (Send + Sync).
///
/// This operator maintains the latest value from the source stream and the latest value
/// from the filter stream. When the filter stream emits a value that passes the predicate,
/// the operator emits the most recent source value (if one exists).
///
/// # Behavior
///
/// - Source stream values are cached but don't trigger emissions
/// - Filter stream values are evaluated with the predicate
/// - Emission occurs when: filter predicate returns `true` AND a source value exists
/// - Emitted values preserve the temporal order of the triggering filter value
///
/// # Arguments
///
/// * `source_stream` - Stream whose values are sampled
/// * `filter_stream` - Stream whose values control when to sample the source
/// * `filter` - Predicate function applied to filter stream values. Returns `true` to emit.
pub fn take_latest_when_impl<T, S1, S2, F>(
    source_stream: S1,
    filter_stream: S2,
    filter: F,
) -> impl Stream<Item = StreamItem<T>> + Send + Sync + Unpin
where
    S1: Stream<Item = StreamItem<T>> + Send + Sync + 'static,
    S2: Stream<Item = StreamItem<T>> + Send + Sync + 'static,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
    F: Fn(&T::Inner) -> bool + Send + Sync + 'static,
{
    let streams: Vec<Pin<Box<dyn Stream<Item = StreamItem<T>> + Send + Sync>>> =
        vec![Box::pin(source_stream), Box::pin(filter_stream)];

    let source_value = Arc::new(Mutex::new(None));
    let filter = Arc::new(filter);

    let combined_stream = ordered_merge_with_index(streams).filter_map(move |(item, index)| {
        let source_value = Arc::clone(&source_value);
        let filter = Arc::clone(&filter);
        async move {
            match item {
                StreamItem::Value(ordered_value) => {
                    match index {
                        0 => {
                            // Source stream update - just cache the value, don't emit
                            let mut source = source_value.lock();
                            *source = Some(ordered_value);
                            None
                        }
                        1 => {
                            // Filter stream update - check if we should sample the source
                            let source = source_value.lock();

                            // Update filter value
                            let filter_inner = ordered_value.clone().into_inner();

                            // Now check the condition and potentially emit
                            if filter(&filter_inner) {
                                source.as_ref().map(|src| {
                                    StreamItem::Value(T::with_timestamp(
                                        src.clone().into_inner(),
                                        ordered_value.timestamp(),
                                    ))
                                })
                            } else {
                                None
                            }
                        }
                        _ => {
                            // Unexpected stream index - this shouldn't happen
                            None
                        }
                    }
                }
                StreamItem::Error(e) => Some(StreamItem::Error(e)),
            }
        }
    });

    Box::pin(combined_stream)
}

/// Generic implementation of take_latest_when for single-threaded runtimes (Send only).
///
/// This operator maintains the latest value from the source stream and the latest value
/// from the filter stream. When the filter stream emits a value that passes the predicate,
/// the operator emits the most recent source value (if one exists).
///
/// # Behavior
///
/// - Source stream values are cached but don't trigger emissions
/// - Filter stream values are evaluated with the predicate
/// - Emission occurs when: filter predicate returns `true` AND a source value exists
/// - Emitted values preserve the temporal order of the triggering filter value
///
/// # Arguments
///
/// * `source_stream` - Stream whose values are sampled
/// * `filter_stream` - Stream whose values control when to sample the source
/// * `filter` - Predicate function applied to filter stream values. Returns `true` to emit.
pub fn take_latest_when_impl_single<T, S1, S2, F>(
    source_stream: S1,
    filter_stream: S2,
    filter: F,
) -> impl Stream<Item = StreamItem<T>> + Send + Sync + Unpin
where
    S1: Stream<Item = StreamItem<T>> + Send + Sync + 'static,
    S2: Stream<Item = StreamItem<T>> + Send + Sync + 'static,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
    F: Fn(&T::Inner) -> bool + Send + Sync + 'static,
{
    let streams: Vec<Pin<Box<dyn Stream<Item = StreamItem<T>> + Send + Sync>>> =
        vec![Box::pin(source_stream), Box::pin(filter_stream)];

    let source_value = Arc::new(Mutex::new(None));
    let filter = Arc::new(filter);

    let combined_stream = ordered_merge_with_index(streams).filter_map(move |(item, index)| {
        let source_value = Arc::clone(&source_value);
        let filter = Arc::clone(&filter);
        async move {
            match item {
                StreamItem::Value(ordered_value) => {
                    match index {
                        0 => {
                            // Source stream update - just cache the value, don't emit
                            let mut source = source_value.lock();
                            *source = Some(ordered_value);
                            None
                        }
                        1 => {
                            // Filter stream update - check if we should sample the source
                            let source = source_value.lock();

                            // Update filter value
                            let filter_inner = ordered_value.clone().into_inner();

                            // Now check the condition and potentially emit
                            if filter(&filter_inner) {
                                source.as_ref().map(|src| {
                                    StreamItem::Value(T::with_timestamp(
                                        src.clone().into_inner(),
                                        ordered_value.timestamp(),
                                    ))
                                })
                            } else {
                                None
                            }
                        }
                        _ => {
                            // Unexpected stream index - this shouldn't happen
                            None
                        }
                    }
                }
                StreamItem::Error(e) => Some(StreamItem::Error(e)),
            }
        }
    });

    Box::pin(combined_stream)
}
