use futures::stream::{Empty, Stream, StreamExt, empty};
use futures::task::{Context, Poll};
use pin_project::pin_project;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::ordered_merge::OrderedMergeExt;
use crate::sequenced::Sequenced;
use crate::sequenced_stream::SequencedStreamExt;

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
    ) -> MergedStream<impl Stream<Item = Sequenced<T>>, State, Sequenced<T>>
    where
        NewStream: SequencedStreamExt<NewItem> + Send + 'static,
        F: FnMut(Sequenced<NewItem>, &mut State) -> Sequenced<T> + Send + Sync + Clone + 'static,
        NewItem: Send + 'static,
        T: Send + Ord + Unpin + 'static,
        Item: Into<Sequenced<T>>,
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

        let self_stream_mapped = self.inner.map(|item| item.into());

        let merged_stream = vec![
            Box::pin(self_stream_mapped) as Pin<Box<dyn Stream<Item = Sequenced<T>> + Send>>,
            Box::pin(new_stream_mapped) as Pin<Box<dyn Stream<Item = Sequenced<T>> + Send>>,
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
