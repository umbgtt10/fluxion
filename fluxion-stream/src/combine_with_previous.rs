use crate::timestamped::Timestamped;
use futures::{Stream, StreamExt, future};

/// Extension trait for streams of `Timestamped<T>` that emits each item along with its previous.
///
/// This accepts any stream whose `Item` is `Timestamped<T>`, keeping the operator generic
/// and allowing callers to pass test helpers or arbitrary streams without boxing.
pub trait CombineWithPreviousExt<T>: Stream<Item = Timestamped<T>> + Sized
where
    T: Clone + Send + Sync + 'static,
{
    fn combine_with_previous(self) -> impl Stream<Item = (Option<Timestamped<T>>, Timestamped<T>)>;
}

impl<T, S> CombineWithPreviousExt<T> for S
where
    S: Stream<Item = Timestamped<T>> + Send + Sized + 'static,
    T: Clone + Send + Sync + 'static,
{
    fn combine_with_previous(self) -> impl Stream<Item = (Option<Timestamped<T>>, Timestamped<T>)> {
        self.scan(
            None,
            |state: &mut Option<Timestamped<T>>, current: Timestamped<T>| {
                let previous = state.take();
                *state = Some(current.clone());
                future::ready(Some((previous, current)))
            },
        )
    }
}
