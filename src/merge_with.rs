use futures::stream::{Empty, Stream, StreamExt, empty, select};
use futures::task::{Context, Poll};
use pin_project::pin_project;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

#[pin_project]
pub struct MergedStream<S, State, Item> {
    #[pin]
    inner: S,
    state: Arc<Mutex<State>>,
    _marker: PhantomData<Item>,
}

impl<State, Item> MergedStream<Empty<Item>, State, Item>
where
    State: Send + 'static,
    Item: Send + 'static,
{
    pub fn seed(initial_state: State) -> MergedStream<Empty<Item>, State, Item> {
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
    Item: Send + 'static,
{
    pub fn merge_with<NewStream, F, NewItem>(
        self,
        new_stream: NewStream,
        process_fn: F,
    ) -> MergedStream<impl Stream<Item = Item>, State, Item>
    where
        NewStream: Stream<Item = NewItem> + Send + 'static,
        F: FnMut(NewItem, &mut State) -> Item + Send + Clone + 'static,
        NewItem: Send + 'static,
    {
        let shared_state = Arc::clone(&self.state);

        let new_stream_mapped = new_stream.then(move |item| {
            let shared_state = Arc::clone(&shared_state);
            let mut process_fn = process_fn.clone();
            async move {
                let mut state = shared_state.lock().await;
                process_fn(item, &mut *state)
            }
        });

        let merged_stream = select(self.inner, new_stream_mapped);

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
