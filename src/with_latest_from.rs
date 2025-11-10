use futures::{Stream, StreamExt};
use std::fmt::Debug;

use crate::combine_latest::{CombineLatestExt, CombinedState, CompareByInner};

pub trait WithLatestFromExt<V, S>: Stream<Item = V> + Sized
where
    Self: Stream<Item = V> + Send + 'static,
    V: Clone + Debug + Ord + Send + Sync + Unpin + CompareByInner + 'static,
    S: Stream<Item = V> + Send + 'static,
{
    fn with_latest_from(
        self,
        other: S,
        filter: impl Fn(&CombinedState<V>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = (V, V)> + Send;
}

impl<V, S, P> WithLatestFromExt<V, S> for P
where
    Self: Stream<Item = V> + Send + 'static,
    V: Clone + Debug + Ord + Send + Sync + Unpin + CompareByInner + 'static,
    S: Stream<Item = V> + Send + 'static,
    P: Stream<Item = V> + CombineLatestExt<V, S> + Sized + Unpin + Send + 'static,
{
    fn with_latest_from(
        self,
        other: S,
        filter: impl Fn(&CombinedState<V>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = (V, V)> + Send {
        self.combine_latest(vec![other], filter)
            .map(|combined_state| {
                let state = combined_state.get_state();
                (state[0].clone(), state[1].clone())
            })
    }
}
