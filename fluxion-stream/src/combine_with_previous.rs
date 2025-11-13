use crate::sequenced::Sequenced;
use crate::sequenced_stream::SequencedStreamExt;
use futures::{Stream, StreamExt, future};

pub trait CombineWithPreviousExt<T>: SequencedStreamExt<T> + Sized
where
    T: Clone + Send + Sync + 'static,
{
    fn combine_with_previous(self) -> impl Stream<Item = (Option<Sequenced<T>>, Sequenced<T>)>;
}

impl<T, S> CombineWithPreviousExt<T> for S
where
    S: SequencedStreamExt<T> + Send + Sized + 'static,
    T: Clone + Send + Sync + 'static,
{
    fn combine_with_previous(self) -> impl Stream<Item = (Option<Sequenced<T>>, Sequenced<T>)> {
        self.scan(
            None,
            |state: &mut Option<Sequenced<T>>, current: Sequenced<T>| {
                let previous = state.take();
                *state = Some(current.clone());
                future::ready(Some((previous, current)))
            },
        )
    }
}
