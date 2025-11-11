use futures::stream::{Empty, Stream, StreamExt, empty};
use futures::task::{Context, Poll};
use pin_project::pin_project;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::select_all_ordered::SelectAllExt;
use crate::timestamped::Timestamped;

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
    pub fn seed(initial_state: State) -> MergedStream<Empty<()>, State, ()> {
        MergedStream {
            inner: empty(),
            state: Arc::new(Mutex::new(initial_state)),
            _marker: PhantomData,
        }
    }
}

impl<S, State, Item> MergedStream<S, State, Item>
where
    S: Stream<Item = Item> + Send + 'static,
    State: Send + Sync + 'static,
    Item: Send + Ord + Unpin + 'static,
{
    pub fn merge_with<NewStream, F, NewItem, T>(
        self,
        new_stream: NewStream,
        process_fn: F,
    ) -> MergedStream<impl Stream<Item = Timestamped<T>>, State, Timestamped<T>>
    where
        NewStream: Stream<Item = Timestamped<NewItem>> + Send + 'static,
        F: FnMut(NewItem, &mut State) -> T + Send + Clone + 'static,
        NewItem: Send + 'static,
        T: Send + Ord + Unpin + 'static,
        Item: Into<Timestamped<T>>,
    {
        let shared_state = Arc::clone(&self.state);

        // Map the new stream, processing items while holding the lock
        // The Timestamped wrapper is preserved through the transformation
        let new_stream_mapped = new_stream.then(move |timestamped_item| {
            let shared_state = Arc::clone(&shared_state);
            let mut process_fn = process_fn.clone();
            let sequence = timestamped_item.sequence();
            async move {
                let item = timestamped_item.into_inner();
                let mut state = shared_state.lock().await;
                let result = process_fn(item, &mut *state);
                // Create a new Timestamped preserving the original sequence
                // This ensures ordering is maintained across the merge
                Timestamped::with_sequence(result, sequence)
            }
        });

        // Convert existing stream items to Timestamped<T>
        // For Empty streams (from seed), this map never executes
        // For subsequent merges, this converts Timestamped<T> -> Timestamped<T> (passthrough)
        let self_stream_mapped = self.inner.map(|item| item.into());

        // Use select_all_ordered to merge streams with guaranteed temporal ordering
        // Build a vector of boxed streams that implement the Stream trait
        let merged_stream = vec![
            Box::pin(self_stream_mapped) as Pin<Box<dyn Stream<Item = Timestamped<T>> + Send>>,
            Box::pin(new_stream_mapped) as Pin<Box<dyn Stream<Item = Timestamped<T>> + Send>>,
        ]
        .select_all_ordered();

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
