// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{HasTimestamp, Timestamped};
use fluxion_ordered_merge::OrderedMergeExt;
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
/// based on their timestamps, ensuring temporal consistency across merged streams.
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
    pub fn seed<OutWrapper>(initial_state: State) -> MergedStream<Empty<()>, State, OutWrapper>
    where
        State: Send + 'static,
    {
        MergedStream {
            inner: empty(),
            state: Arc::new(Mutex::new(initial_state)),
            _marker: PhantomData,
        }
    }
}

impl<S, State, Item> MergedStream<S, State, Item>
where
    S: Stream + Send + Sync + 'static,
    State: Send + Sync + 'static,
    Item: Send + Ord + Unpin + 'static,
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
    pub fn merge_with<NewStream, F>(
        self,
        new_stream: NewStream,
        process_fn: F,
    ) -> MergedStream<impl Stream<Item = Item>, State, Item>
    where
        S::Item: Into<Item>,
        NewStream: Stream + Send + Sync + 'static,
        NewStream::Item: Timestamped + Send + Sync + Ord + Unpin + 'static,
        <NewStream::Item as HasTimestamp>::Inner: Clone + Send + Sync + 'static,
        <NewStream::Item as HasTimestamp>::Timestamp: Ord + Copy + Send + Sync + std::fmt::Debug,
        F: FnMut(
                <NewStream::Item as HasTimestamp>::Inner,
                &mut State,
            ) -> <Item as HasTimestamp>::Inner
            + Send
            + Sync
            + Clone
            + 'static,
        Item: Timestamped + Send + Sync + Ord + Unpin + 'static,
        Item::Timestamp: Ord + Copy + Send + Sync + std::fmt::Debug,
        Item::Inner: Clone + Send + Sync + 'static,
        <NewStream::Item as HasTimestamp>::Timestamp: Into<Item::Timestamp> + Copy,
    {
        let shared_state = Arc::clone(&self.state);
        let new_stream_mapped = new_stream.then(move |timestamped_item| {
            let shared_state = Arc::clone(&shared_state);
            let mut process_fn = process_fn.clone();
            async move {
                let timestamp = timestamped_item.timestamp();
                let inner_value = timestamped_item.into_inner();
                let mut state = shared_state.lock().await;
                let result_value = process_fn(inner_value, &mut *state);
                Item::with_timestamp(result_value, timestamp.into())
            }
        });

        let self_stream_mapped = self.inner.map(Into::into);

        let merged_stream = vec![
            Box::pin(self_stream_mapped) as Pin<Box<dyn Stream<Item = Item> + Send + Sync>>,
            Box::pin(new_stream_mapped) as Pin<Box<dyn Stream<Item = Item> + Send + Sync>>,
        ]
        .ordered_merge();

        MergedStream {
            inner: merged_stream,
            state: self.state,
            _marker: PhantomData,
        }
    }
}

impl<S, State, Item> MergedStream<S, State, Item>
where
    S: Stream<Item = Item>,
{
    /// Converts this `MergedStream` into a `FluxionStream` for operator chaining.
    ///
    /// This allows seamless chaining of stateful merging with other fluxion operators.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_stream::{MergedStream, FluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use tokio::sync::mpsc;
    /// use futures::StreamExt;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let (tx1, rx1) = mpsc::unbounded_channel::<Sequenced<i32>>();
    /// let (tx2, rx2) = mpsc::unbounded_channel::<Sequenced<i32>>();
    ///
    /// // Chain merge_with with other operators in one expression
    /// // Note: Type specified ONCE in seed, not on every merge_with!
    /// let mut result = MergedStream::seed::<Sequenced<i32>>(0)
    ///     .merge_with(
    ///         tokio_stream::wrappers::UnboundedReceiverStream::new(rx1),
    ///         |value, state| {
    ///             *state += value;
    ///             *state
    ///         }
    ///     )
    ///     .into_fluxion_stream()
    ///     .map_ordered(|seq| {
    ///         let value = seq.into_inner();
    ///         Sequenced::new(value * 2)
    ///     })
    ///     .filter_ordered(|&x| x > 10);
    /// # }
    /// ```
    pub fn into_fluxion_stream(self) -> crate::FluxionStream<Self>
    where
        Self: Sized,
    {
        crate::FluxionStream::new(self)
    }
}

impl<S, State, Item> Stream for MergedStream<S, State, Item>
where
    S: Stream<Item = Item>,
{
    type Item = fluxion_core::StreamItem<Item>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project().inner.poll_next(cx) {
            Poll::Ready(Some(item)) => Poll::Ready(Some(fluxion_core::StreamItem::Value(item))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
