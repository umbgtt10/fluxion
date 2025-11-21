// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::Timestamped;
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
    /// temporal order based on timestamps.
    ///
    /// The closure receives unwrapped values and returns unwrapped values - timestamp
    /// propagation is handled automatically by the operator.
    ///
    /// # Parameters
    /// - `new_stream`: The new Timestamped stream to merge
    /// - `process_fn`: Function to process inner values with mutable access to shared state
    pub fn merge_with<NewStream, F, InWrapper, InItem, OutWrapper, OutItem, TS>(
        self,
        new_stream: NewStream,
        process_fn: F,
    ) -> MergedStream<impl Stream<Item = OutWrapper>, State, OutWrapper>
    where
        NewStream: Stream<Item = InWrapper> + Send + Sync + 'static,
        F: FnMut(InItem, &mut State) -> OutItem + Send + Sync + Clone + 'static,
        InWrapper:
            Timestamped<Inner = InItem, Timestamp = TS> + Send + Sync + Ord + Unpin + 'static,
        InItem: Clone + Send + Sync + 'static,
        OutWrapper:
            Timestamped<Inner = OutItem, Timestamp = TS> + Send + Sync + Ord + Unpin + 'static,
        OutItem: Clone + Send + Sync + 'static,
        TS: Ord + Copy + Send + Sync + std::fmt::Debug,
        Item: Into<OutWrapper>,
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
                OutWrapper::with_timestamp(result_value, timestamp)
            }
        });

        let self_stream_mapped = self.inner.map(Into::into);

        let merged_stream = vec![
            Box::pin(self_stream_mapped) as Pin<Box<dyn Stream<Item = OutWrapper> + Send + Sync>>,
            Box::pin(new_stream_mapped) as Pin<Box<dyn Stream<Item = OutWrapper> + Send + Sync>>,
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
    /// let mut result = MergedStream::seed(0)
    ///     .merge_with::<_, _, _, _, Sequenced<i32>, i32, u64>(
    ///         tokio_stream::wrappers::UnboundedReceiverStream::new(rx1),
    ///         |value, state| {
    ///             *state += value;
    ///             *state
    ///         }
    ///     )
    ///     .into_fluxion_stream()
    ///     .map_ordered(|x| x * 2)
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
