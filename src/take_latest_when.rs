use futures::{Stream, StreamExt, future::ready};
use std::fmt::Debug;

use crate::combine_latest::{CombineLatestExt, CombinedState};

pub trait TakeLatestWhenExt<T, S>: Stream<Item = T> + Sized
where
    T: Clone + Send + Sync + 'static,
    S: Stream<Item = T> + Send + Sync + 'static,
{
    fn take_latest_when(
        self,
        filter_stream: S,
        filter: impl Fn(&CombinedState<T>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = T>;
}

impl<T, S> TakeLatestWhenExt<T, S> for S
where
    S: Stream<Item = T> + CombineLatestExt<T, S> + Send + Sync + 'static,
    T: Clone + Debug + Send + Sync + 'static,
{
    fn take_latest_when(
        self,
        filter_stream: S,
        filter: impl Fn(&CombinedState<T>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = T> {
        self.combine_latest(vec![filter_stream], |_| true)
            .filter_map(move |combined_state| {
                ready(if filter(&combined_state) {
                    Some(combined_state.get_state()[0].clone())
                } else {
                    None
                })
            })
    }
}
