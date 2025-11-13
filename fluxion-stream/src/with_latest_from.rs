use futures::{Stream, StreamExt};
use std::fmt::Debug;

use crate::Ordered;
use crate::combine_latest::{CombineLatestExt, CombinedState, CompareByInner};

pub trait WithLatestFromExt<T, S2>: Stream<Item = T> + Sized
where
    T: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + CompareByInner + 'static,
    T::Inner: Clone + Debug + Ord + Send + Sync + 'static,
    S2: Stream<Item = T> + Send + Sync + 'static,
{
    fn with_latest_from(
        self,
        other: S2,
        filter: impl Fn(&CombinedState<T::Inner>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = (T, T)> + Send;
}

impl<T, S2, P> WithLatestFromExt<T, S2> for P
where
    T: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + CompareByInner + 'static,
    T::Inner: Clone + Debug + Ord + Send + Sync + 'static,
    S2: Stream<Item = T> + Send + Sync + 'static,
    P: Stream<Item = T> + CombineLatestExt<T> + Sized + Unpin + Send + Sync + 'static,
{
    fn with_latest_from(
        self,
        other: S2,
        filter: impl Fn(&CombinedState<T::Inner>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = (T, T)> + Send {
        self.combine_latest(vec![other], filter)
            .map(|ordered_combined| {
                let combined_state = ordered_combined.get();
                let state = combined_state.get_state();
                // Create new Ordered values from the inner values
                let order = ordered_combined.order();
                (
                    T::with_order(state[0].clone(), order),
                    T::with_order(state[1].clone(), order),
                )
            })
    }
}
