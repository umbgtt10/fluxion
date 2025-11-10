use futures::{Stream, StreamExt, future::ready};
use std::fmt::Debug;

use crate::combine_latest::{CombineLatestExt, CombinedState};
use crate::sequenced::Sequenced;

pub trait TakeLatestWhenExt<T, S>: Stream<Item = Sequenced<T>> + Sized
where
    T: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    S: Stream<Item = Sequenced<T>> + Send + Sync + 'static,
{
    fn take_latest_when(
        self,
        filter_stream: S,
        filter: impl Fn(&CombinedState<T>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = T>;
}

impl<T, S> TakeLatestWhenExt<T, S> for S
where
    S: Stream<Item = Sequenced<T>> + Send + Sync + 'static,
    T: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn take_latest_when(
        self,
        filter_stream: S,
        filter: impl Fn(&CombinedState<T>) -> bool + Send + Sync + 'static,
    ) -> impl futures::Stream<Item = T> {
        self.combine_latest(vec![filter_stream], |_| true)
            .filter_map(move |combined_state| {
                let values: Vec<T> = combined_state
                    .get_state()
                    .iter()
                    .map(|s| s.value.clone())
                    .collect();
                let value_state = CombinedState::new(values);
                ready(if filter(&value_state) {
                    Some(combined_state.get_state()[0].value.clone())
                } else {
                    None
                })
            })
    }
}
