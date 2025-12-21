// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::ordered_merge::ordered_merge_with_index;
use alloc::sync::Arc;
use core::fmt::Debug;
use core::pin::Pin;
use fluxion_core::{Fluxion, HasTimestamp, StreamItem, Timestamped};
use futures::lock::Mutex as FutureMutex;
use futures::stream::{empty, Empty, Stream, StreamExt};
use futures::task::{Context, Poll};
use pin_project::pin_project;
use std::marker::PhantomData;

/// A stateful stream merger that combines multiple Timestamped streams while maintaining state.
///
/// Internally uses [`fluxion_ordered_merge`] to merge streams in order
/// based on their timestamps, ensuring temporal consistency across merged streams.
#[pin_project]
pub struct MergedStream<S, State, Item> {
    #[pin]
    inner: S,
    state: Arc<FutureMutex<State>>,
    _marker: PhantomData<Item>,
}

impl<State> MergedStream<Empty<StreamItem<()>>, State, ()>
where
    State: Send + 'static,
{
    /// Creates a new `MergedStream` with initial state and output wrapper type.
    ///
    /// Specify the output wrapper type once here to avoid turbofish on every `merge_with`.
    ///
    /// # Example
    /// ```no_run
    /// # use fluxion_stream::MergedStream;
    /// # use fluxion_test_utils::Sequenced;
    /// let stream = MergedStream::seed::<Sequenced<i32>>(0);
    /// ```
    pub fn seed<OutWrapper>(
        initial_state: State,
    ) -> MergedStream<Empty<StreamItem<OutWrapper>>, State, OutWrapper>
    where
        State: Send + 'static,
        OutWrapper: Send + Unpin + 'static,
    {
        MergedStream {
            inner: empty::<StreamItem<OutWrapper>>(),
            state: Arc::new(FutureMutex::new(initial_state)),
            _marker: PhantomData,
        }
    }
}

impl<S, State, Item> MergedStream<S, State, Item>
where
    S: Stream<Item = StreamItem<Item>> + Send + Sync + 'static,
    State: Send + Sync + 'static,
    Item: Fluxion,
    <Item as Timestamped>::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    <Item as HasTimestamp>::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    /// Merges a new Timestamped stream into the existing merged stream.
    ///
    /// Uses [`fluxion_ordered_merge`] to combine the streams while preserving
    /// temporal order based on timestamps.
    ///
    /// The closure receives unwrapped values and returns unwrapped values - timestamp
    /// propagation is handled automatically by the operator.
    ///
    /// # Parameters
    /// - `new_stream`: The new Timestamped stream to merge
    /// - `process_fn`: Function to process inner values with mutable access to shared state
    pub fn merge_with<NewStream, NewItem, F>(
        self,
        new_stream: NewStream,
        process_fn: F,
    ) -> MergedStream<impl Stream<Item = StreamItem<Item>>, State, Item>
    where
        NewStream: Stream<Item = StreamItem<NewItem>> + Send + Sync + 'static,
        NewItem: Fluxion,
        <NewItem as Timestamped>::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
        <NewItem as HasTimestamp>::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
        F: FnMut(<NewItem as Timestamped>::Inner, &mut State) -> <Item as Timestamped>::Inner
            + Send
            + Sync
            + Clone
            + 'static,
        Item: Fluxion,
        <Item as Timestamped>::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
        <Item as HasTimestamp>::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
        <NewItem as HasTimestamp>::Timestamp: Into<<Item as HasTimestamp>::Timestamp> + Copy,
    {
        let shared_state = Arc::clone(&self.state);
        let new_stream_mapped = new_stream.then(move |stream_item| {
            let shared_state = Arc::clone(&shared_state);
            let mut process_fn = process_fn.clone();
            async move {
                match stream_item {
                    StreamItem::Value(timestamped_item) => {
                        let timestamp = timestamped_item.timestamp();
                        let inner_value = timestamped_item.into_inner();
                        let mut state = shared_state.lock().await;
                        let result_value = process_fn(inner_value, &mut *state);
                        StreamItem::Value(Item::with_timestamp(result_value, timestamp.into()))
                    }
                    StreamItem::Error(e) => StreamItem::Error(e),
                }
            }
        });

        // self.inner already yields `StreamItem<Item>`; pass through values unchanged
        let self_stream_mapped = self.inner;

        let streams = vec![
            Box::pin(self_stream_mapped)
                as Pin<Box<dyn Stream<Item = StreamItem<Item>> + Send + Sync>>,
            Box::pin(new_stream_mapped)
                as Pin<Box<dyn Stream<Item = StreamItem<Item>> + Send + Sync>>,
        ];

        // Use ordered_merge_with_index for immediate error emission (Rx semantics)
        // Discard the index since we don't need to track which stream emitted
        let merged_stream = ordered_merge_with_index(streams).map(|(item, _index)| item);

        MergedStream {
            inner: merged_stream,
            state: self.state,
            _marker: PhantomData,
        }
    }
}

impl<S, State, Item> Stream for MergedStream<S, State, Item>
where
    S: Stream<Item = StreamItem<Item>>,
{
    type Item = StreamItem<Item>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }
}
