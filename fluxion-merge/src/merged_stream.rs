// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_ordered_merge::OrderedMergeExt;
use fluxion_test_utils::ChronoTimestamped;
use futures::stream::{empty, Empty, Stream, StreamExt};
use futures::task::{Context, Poll};
use pin_project::pin_project;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

/// A stateful stream merger that combines multiple Timestamped streams while maintaining state.
///
/// Internally uses [`fluxion_ordered_merge`] to merge streams in order
/// based on their sequence numbers, ensuring temporal consistency across merged streams.
#[pin_project]
pub struct MergedStream<S, State, Item> {
    #[pin]
    inner: S,
    state: Arc<Mutex<State>>,
    _marker: PhantomData<Item>,
}

impl<State> MergedStream<Empty<()>, State, ()>
where
    State: Send + 'static,
{
    pub fn seed(initial_state: State) -> Self {
        Self {
            inner: empty(),
            state: Arc::new(Mutex::new(initial_state)),
            _marker: PhantomData,
        }
    }
}

impl<S, State, Item> MergedStream<S, State, Item>
where
    S: Stream<Item = Item> + Send + Sync + 'static,
    State: Send + Sync + 'static,
    Item: Send + Ord + Unpin + 'static,
{
    /// Merges a new Timestamped stream into the existing merged stream.
    ///
    /// Uses [`fluxion_ordered_merge`] to combine the streams while preserving
    /// temporal order based on sequence numbers.
    ///
    /// # Parameters
    /// - `new_stream`: The new Timestamped stream to merge
    /// - `process_fn`: Function to process each item with mutable access to shared state
    pub fn merge_with<NewStream, F, NewItem, T>(
        self,
        new_stream: NewStream,
        process_fn: F,
    ) -> MergedStream<impl Stream<Item = ChronoTimestamped<T>>, State, ChronoTimestamped<T>>
    where
        NewStream: Stream<Item = ChronoTimestamped<NewItem>> + Send + Sync + 'static,
        F: FnMut(ChronoTimestamped<NewItem>, &mut State) -> ChronoTimestamped<T>
            + Send
            + Sync
            + Clone
            + 'static,
        NewItem: Send + Sync + 'static,
        T: Send + Sync + Ord + Unpin + 'static,
        Item: Into<ChronoTimestamped<T>>,
    {
        let shared_state = Arc::clone(&self.state);
        let new_stream_mapped = new_stream.then(move |timestamped_item| {
            let shared_state = Arc::clone(&shared_state);
            let mut process_fn = process_fn.clone();
            async move {
                let mut state = shared_state.lock().await;
                process_fn(timestamped_item, &mut *state)
            }
        });

        let self_stream_mapped = self.inner.map(Into::into);

        let merged_stream = vec![
            Box::pin(self_stream_mapped)
                as Pin<Box<dyn Stream<Item = ChronoTimestamped<T>> + Send + Sync>>,
            Box::pin(new_stream_mapped)
                as Pin<Box<dyn Stream<Item = ChronoTimestamped<T>> + Send + Sync>>,
        ]
        .ordered_merge();

        MergedStream {
            inner: merged_stream,
            state: self.state,
            _marker: PhantomData,
        }
    }
}

impl<S, State, Item> Stream for MergedStream<S, State, Item>
where
    S: Stream<Item = Item>,
{
    type Item = Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }
}
