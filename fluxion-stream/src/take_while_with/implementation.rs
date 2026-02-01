// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::cmp::Ordering;
use fluxion_core::HasTimestamp;

#[derive(Clone, Debug)]
pub enum Item<TItem, TFilter>
where
    TItem: HasTimestamp,
{
    Source(TItem),
    Filter(TFilter),
}

impl<TItem, TFilter> HasTimestamp for Item<TItem, TFilter>
where
    TItem: HasTimestamp,
    TFilter: HasTimestamp<Timestamp = TItem::Timestamp>,
{
    type Timestamp = TItem::Timestamp;

    fn timestamp(&self) -> Self::Timestamp {
        match self {
            Self::Source(s) => s.timestamp(),
            Self::Filter(f) => f.timestamp(),
        }
    }
}

impl<TItem, TFilter> Unpin for Item<TItem, TFilter> where TItem: HasTimestamp {}

impl<TItem, TFilter> PartialEq for Item<TItem, TFilter>
where
    TItem: HasTimestamp,
    TFilter: HasTimestamp<Timestamp = TItem::Timestamp>,
{
    fn eq(&self, other: &Self) -> bool {
        self.timestamp() == other.timestamp()
    }
}

impl<TItem, TFilter> Eq for Item<TItem, TFilter>
where
    TItem: HasTimestamp,
    TFilter: HasTimestamp<Timestamp = TItem::Timestamp>,
{
}

impl<TItem, TFilter> PartialOrd for Item<TItem, TFilter>
where
    TItem: HasTimestamp,
    TFilter: HasTimestamp<Timestamp = TItem::Timestamp>,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<TItem, TFilter> Ord for Item<TItem, TFilter>
where
    TItem: HasTimestamp,
    TFilter: HasTimestamp<Timestamp = TItem::Timestamp>,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp().cmp(&other.timestamp())
    }
}

macro_rules! define_take_while_with_impl {
    ($($stream_bounds:tt)*) => {
        use crate::ordered_merge::ordered_merge_with_index;
        use alloc::boxed::Box;
        use alloc::sync::Arc;
        use alloc::vec;
        use alloc::vec::Vec;
        use core::fmt::Debug;
        use core::pin::Pin;
        use fluxion_core::fluxion_mutex::Mutex;
        use fluxion_core::{Fluxion, HasTimestamp, StreamItem, Timestamped};
        use futures::stream::StreamExt;
        use futures::Stream;
        use super::implementation::Item;

        type PinnedStream<T> = Pin<Box<dyn Stream<Item = StreamItem<T>> + $($stream_bounds)* 'static>>;

        pub trait TakeWhileExt<TItem, TFilter, S>: Stream<Item = StreamItem<TItem>> + Sized
        where
            TItem: Fluxion,
            TItem::Inner: Clone + Debug + Ord + Unpin + $($stream_bounds)* 'static,
            TItem::Timestamp: Debug + Ord + Copy + $($stream_bounds)* 'static,
            TFilter: Fluxion<Timestamp = TItem::Timestamp>,
            TFilter::Inner: Clone + Debug + Ord + Unpin + $($stream_bounds)* 'static,
            S: Stream<Item = StreamItem<TFilter>> + $($stream_bounds)* 'static,
        {
            fn take_while_with(
                self,
                filter_stream: S,
                filter: impl Fn(&TFilter::Inner) -> bool + $($stream_bounds)* 'static,
            ) -> impl Stream<Item = StreamItem<TItem>>;
        }

        impl<TItem, TFilter, S, P> TakeWhileExt<TItem, TFilter, S> for P
        where
            P: Stream<Item = StreamItem<TItem>> + Unpin + $($stream_bounds)* 'static,
            TItem: Fluxion,
            TItem::Inner: Clone + Debug + Ord + Unpin + $($stream_bounds)* 'static,
            TItem::Timestamp: Debug + Ord + Copy + $($stream_bounds)* 'static,
            TFilter: Fluxion<Timestamp = TItem::Timestamp>,
            TFilter::Inner: Clone + Debug + Ord + Unpin + $($stream_bounds)* 'static,
            S: Stream<Item = StreamItem<TFilter>> + $($stream_bounds)* 'static,
        {
            fn take_while_with(
                self,
                filter_stream: S,
                filter: impl Fn(&TFilter::Inner) -> bool + $($stream_bounds)* 'static,
            ) -> impl Stream<Item = StreamItem<TItem>> {
                let filter = Arc::new(filter);

                let source_stream =
                    self.map(|item| item.map(|value| Item::<TItem, TFilter>::Source(value)));

                let filter_stream =
                    filter_stream.map(|item| item.map(|value| Item::<TItem, TFilter>::Filter(value)));

                let streams: Vec<PinnedStream<Item<TItem, TFilter>>> =
                    vec![Box::pin(source_stream), Box::pin(filter_stream)];

                let state = Arc::new(Mutex::new((None::<TFilter::Inner>, false)));

                let combined_stream = ordered_merge_with_index(streams).filter_map({
                    let state = Arc::clone(&state);
                    move |(stream_item, _index)| {
                        let state = Arc::clone(&state);
                        let filter = Arc::clone(&filter);

                        async move {
                            match stream_item {
                                StreamItem::Error(e) => Some(StreamItem::Error(e)),
                                StreamItem::Value(item) => {
                                    let mut guard = state.lock();
                                    let (filter_state, terminated) = &mut *guard;

                                    if *terminated {
                                        return None;
                                    }

                                    match item {
                                        Item::Filter(filter_val) => {
                                            *filter_state = Some(filter_val.clone().into_inner());
                                            None
                                        }
                                        Item::Source(source_val) => filter_state.as_ref().map_or_else(
                                            || None,
                                            |fval| {
                                                if filter(fval) {
                                                    Some(StreamItem::Value(source_val.clone()))
                                                } else {
                                                    *terminated = true;
                                                    None
                                                }
                                            },
                                        ),
                                    }
                                }
                            }
                        }
                    }
                });

                Box::pin(combined_stream)
            }
        }

        impl<TItem, TFilter> Timestamped for Item<TItem, TFilter>
        where
            TItem: HasTimestamp + Clone + Debug + Ord + Unpin + $($stream_bounds)* 'static,
            TFilter: HasTimestamp<Timestamp = TItem::Timestamp>
                + Clone
                + Debug
                + Ord
                + Unpin
                + $($stream_bounds)* 'static,
        {
            type Inner = Self;

            fn with_timestamp(value: Self::Inner, _timestamp: Self::Timestamp) -> Self {
                value
            }

            fn into_inner(self) -> Self::Inner {
                self
            }
        }
    };
}
