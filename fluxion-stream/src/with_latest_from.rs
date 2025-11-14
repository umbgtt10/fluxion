// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use futures::{Stream, StreamExt};
use std::fmt::Debug;

use crate::Ordered;
use crate::combine_latest::{CombineLatestExt, CombinedState};
use fluxion_core::CompareByInner;
use fluxion_core::into_stream::IntoStream;

pub trait WithLatestFromExt<T>: Stream<Item = T> + Sized
where
    T: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + CompareByInner + 'static,
    T::Inner: Clone + Debug + Ord + Send + Sync + 'static,
{
    fn with_latest_from<IS>(
        self,
        other: IS,
        filter: impl Fn(&CombinedState<T::Inner>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = (T, T)> + Send
    where
        IS: IntoStream<Item = T>,
        IS::Stream: Send + Sync + 'static;
}

impl<T, P> WithLatestFromExt<T> for P
where
    T: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + CompareByInner + 'static,
    T::Inner: Clone + Debug + Ord + Send + Sync + 'static,
    P: Stream<Item = T> + CombineLatestExt<T> + Sized + Unpin + Send + Sync + 'static,
{
    fn with_latest_from<IS>(
        self,
        other: IS,
        filter: impl Fn(&CombinedState<T::Inner>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = (T, T)> + Send
    where
        IS: IntoStream<Item = T>,
        IS::Stream: Send + Sync + 'static,
    {
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
