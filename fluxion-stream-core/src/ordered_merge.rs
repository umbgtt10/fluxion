// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Generic implementation of ordered_merge operator - runtime-agnostic

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Debug;
use core::pin::Pin;
use fluxion_core::into_stream::IntoStream;
use fluxion_core::{Fluxion, StreamItem};
use futures::task::{Context, Poll};
use futures::{Stream, StreamExt};

/// Generic implementation of ordered_merge - works with any runtime
///
/// This implementation is completely runtime-agnostic. Runtime-specific wrappers
/// (in fluxion-stream-multi and fluxion-stream-single) provide the trait definitions
/// with appropriate bounds.
pub fn ordered_merge_impl<S, T, IS>(
    primary_stream: S,
    others: Vec<IS>,
) -> impl Stream<Item = StreamItem<T>> + Send + Sync + Unpin
where
    T: Fluxion + Unpin,
    T::Inner: Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
    S: Stream<Item = StreamItem<T>> + Send + Sync + 'static,
    IS: IntoStream<Item = StreamItem<T>>,
    IS::Stream: Send + Sync + 'static,
{
    let mut all_streams =
        vec![Box::pin(primary_stream) as Pin<Box<dyn Stream<Item = StreamItem<T>> + Send + Sync>>];
    for into_stream in others {
        let stream = into_stream.into_stream();
        all_streams.push(Box::pin(stream));
    }

    // Use the indexed version internally and discard the index
    Box::pin(StreamExt::map(
        OrderedMergeWithIndex::new(all_streams),
        |(item, _index)| item,
    ))
}

/// Helper function to create an ordered merge stream with immediate error emission
/// and stream indexing for operators that need to track which stream emitted each item.
///
/// This is used internally by operators like `combine_latest`, `with_latest_from`, etc.
/// that need both temporal ordering of values AND immediate error propagation.
pub(crate) fn ordered_merge_with_index<T>(
    streams: Vec<Pin<Box<dyn Stream<Item = StreamItem<T>> + Send + Sync>>>,
) -> impl Stream<Item = (StreamItem<T>, usize)> + Send + Sync + Unpin
where
    T: Fluxion + Unpin,
    T::Inner: Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    Box::pin(OrderedMergeWithIndex::new(streams))
}

/// Stream that merges multiple timestamped streams in temporal order with stream indices.
///
/// This merges streams while tracking which stream each item came from.
/// Errors are emitted immediately without buffering.
struct OrderedMergeWithIndex<T>
where
    T: Fluxion,
    T::Inner: Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    streams: Vec<Pin<Box<dyn Stream<Item = StreamItem<T>> + Send + Sync>>>,
    buffered: Vec<Option<T>>,
}

impl<T> OrderedMergeWithIndex<T>
where
    T: Fluxion,
    T::Inner: Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    fn new(streams: Vec<Pin<Box<dyn Stream<Item = StreamItem<T>> + Send + Sync>>>) -> Self {
        let count = streams.len();
        let buffered = (0..count).map(|_| None).collect();
        Self { streams, buffered }
    }
}

impl<T> Stream for OrderedMergeWithIndex<T>
where
    T: Fluxion + Unpin,
    T::Inner: Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    type Item = (StreamItem<T>, usize);

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Poll streams to fill empty buffer slots - emit errors immediately with their index
        let mut any_pending = false;

        for i in 0..self.streams.len() {
            if self.buffered[i].is_none() {
                match self.streams[i].as_mut().poll_next(cx) {
                    Poll::Ready(Some(StreamItem::Error(e))) => {
                        // Fail-fast: emit error immediately with stream index
                        return Poll::Ready(Some((StreamItem::Error(e), i)));
                    }
                    Poll::Ready(Some(StreamItem::Value(item))) => {
                        self.buffered[i] = Some(item);
                    }
                    Poll::Ready(None) => {
                        // Stream is done, leave as None
                    }
                    Poll::Pending => {
                        any_pending = true;
                    }
                }
            }
        }

        // Find the minimum timestamped value among all buffered values
        let mut min_idx = None;
        let mut min_val: Option<&T> = None;

        for (i, item) in self.buffered.iter().enumerate() {
            if let Some(val) = item {
                let should_update = min_val.is_none_or(|curr_val| val < curr_val);

                if should_update {
                    min_idx = Some(i);
                    min_val = Some(val);
                }
            }
        }

        // Return the minimum value if found, with its stream index
        if let Some(idx) = min_idx {
            if let Some(item) = self.buffered[idx].take() {
                Poll::Ready(Some((StreamItem::Value(item), idx)))
            } else {
                unreachable!("min_idx is only Some when buffered[idx] is Some")
            }
        } else if any_pending {
            Poll::Pending
        } else {
            Poll::Ready(None)
        }
    }
}
