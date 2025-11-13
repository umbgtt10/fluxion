use futures::{Stream, StreamExt};
use std::fmt::Debug;

use crate::combine_latest::{CombineLatestExt, CombinedState, CompareByInner};
use crate::sequenced::Sequenced;
use crate::sequenced_stream::SequencedStreamExt;

pub trait WithLatestFromExt<T, S2>: SequencedStreamExt<T> + Sized
where
    Self: SequencedStreamExt<T> + Send + 'static,
    Sequenced<T>: Clone + Debug + Ord + Send + Sync + Unpin + CompareByInner + 'static,
    S2: Stream<Item = Sequenced<T>> + Send + 'static,
{
    fn with_latest_from(
        self,
        other: S2,
        filter: impl Fn(&CombinedState<Sequenced<T>>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = (Sequenced<T>, Sequenced<T>)> + Send;
}

impl<T, S2, P> WithLatestFromExt<T, S2> for P
where
    Self: SequencedStreamExt<T> + Send + 'static,
    Sequenced<T>: Clone + Debug + Ord + Send + Sync + Unpin + CompareByInner + 'static,
    S2: Stream<Item = Sequenced<T>> + Send + 'static,
    P: SequencedStreamExt<T> + CombineLatestExt<T, S2> + Sized + Unpin + Send + 'static,
{
    fn with_latest_from(
        self,
        other: S2,
        filter: impl Fn(&CombinedState<Sequenced<T>>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = (Sequenced<T>, Sequenced<T>)> + Send {
        self.combine_latest(vec![other], filter)
            .map(|combined_state| {
                let state = combined_state.get_state();
                (state[0].clone(), state[1].clone())
            })
    }
}
