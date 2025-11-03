use futures::{Stream, StreamExt, future};

pub trait CombineWithPreviousExt<V>: Stream<Item = V> + Sized
where
    V: Clone + Send + Sync + 'static,
{
    fn combine_with_previous(self) -> impl Stream<Item = (Option<V>, V)>;
}

impl<V, S> CombineWithPreviousExt<V> for S
where
    S: Stream<Item = V> + Send + Sized + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn combine_with_previous(self) -> impl Stream<Item = (Option<V>, V)> {
        self.scan(None, |state: &mut Option<V>, current: V| {
            let previous = state.take();
            *state = Some(current.clone());
            future::ready(Some((previous, current)))
        })
    }
}
