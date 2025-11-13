use futures::{Stream, StreamExt};
use std::fmt::Debug;

use crate::combine_latest::{CombineLatestExt, CombinedState, CompareByInner};
use crate::sequenced::Sequenced;
use crate::sequenced_stream::SequencedStreamExt;

pub trait WithLatestFromExt<T, S2>: SequencedStreamExt<T> + Sized
where
    T: Clone + Debug + Send + Sync + 'static,
    Self: SequencedStreamExt<T> + Send + 'static,
    Sequenced<T>: Clone + Debug + Ord + Send + Sync + Unpin + CompareByInner + 'static,
    S2: Stream<Item = Sequenced<T>> + Send + 'static,
{
    fn with_latest_from(
        self,
        other: S2,
        filter: impl Fn(&CombinedState<T>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = (Sequenced<T>, Sequenced<T>)> + Send;
}

impl<T, S2, P> WithLatestFromExt<T, S2> for P
where
    T: Clone + Debug + Send + Sync + 'static,
    Self: SequencedStreamExt<T> + Send + 'static,
    Sequenced<T>: Clone + Debug + Ord + Send + Sync + Unpin + CompareByInner + 'static,
    S2: Stream<Item = Sequenced<T>> + Send + 'static,
    P: SequencedStreamExt<T> + CombineLatestExt<T, S2> + Sized + Unpin + Send + 'static,
{
    fn with_latest_from(
        self,
        other: S2,
        filter: impl Fn(&CombinedState<T>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = (Sequenced<T>, Sequenced<T>)> + Send {
        self.combine_latest(vec![other], filter)
            .map(|sequenced_combined| {
                let combined_state = sequenced_combined.get();
                let state = combined_state.get_state();
                // Create new Sequenced values from the inner T values
                let seq = sequenced_combined.sequence();
                (
                    Sequenced::with_sequence(state[0].clone(), seq),
                    Sequenced::with_sequence(state[1].clone(), seq),
                )
            })
    }
}
