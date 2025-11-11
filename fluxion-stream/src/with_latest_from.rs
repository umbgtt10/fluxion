use futures::{Stream, StreamExt};
use std::fmt::Debug;

use crate::combine_latest::{CombineLatestExt, CombinedState, CompareByInner};
use crate::timestamped::Timestamped;

pub trait WithLatestFromExt<T, S>: Stream<Item = Timestamped<T>> + Sized
where
    Self: Stream<Item = Timestamped<T>> + Send + 'static,
    Timestamped<T>: Clone + Debug + Ord + Send + Sync + Unpin + CompareByInner + 'static,
    S: Stream<Item = Timestamped<T>> + Send + 'static,
{
    fn with_latest_from(
        self,
        other: S,
        filter: impl Fn(&CombinedState<Timestamped<T>>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = (Timestamped<T>, Timestamped<T>)> + Send;
}

impl<T, S, P> WithLatestFromExt<T, S> for P
where
    Self: Stream<Item = Timestamped<T>> + Send + 'static,
    Timestamped<T>: Clone + Debug + Ord + Send + Sync + Unpin + CompareByInner + 'static,
    S: Stream<Item = Timestamped<T>> + Send + 'static,
    P: Stream<Item = Timestamped<T>> + CombineLatestExt<T, S> + Sized + Unpin + Send + 'static,
{
    fn with_latest_from(
        self,
        other: S,
        filter: impl Fn(&CombinedState<Timestamped<T>>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = (Timestamped<T>, Timestamped<T>)> + Send {
        self.combine_latest(vec![other], filter)
            .map(|combined_state| {
                let state = combined_state.get_state();
                (state[0].clone(), state[1].clone())
            })
    }
}
