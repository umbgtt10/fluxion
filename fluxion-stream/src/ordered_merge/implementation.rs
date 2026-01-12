// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

/// Macro that generates the complete ordered merge implementation.
///
/// This macro eliminates duplication between multi-threaded and single-threaded
/// implementations, which differ only in trait bounds (Send + Sync vs not).
macro_rules! define_ordered_merge_impl {
    (
        inner_bounds: [$($inner_bounds:tt)*],
        timestamp_bounds: [$($timestamp_bounds:tt)*],
        stream_bounds: [$($stream_bounds:tt)*],
        boxed_stream: [$($boxed_stream:tt)*]
    ) => {
        use alloc::boxed::Box;
        use alloc::vec;
        use alloc::vec::Vec;
        use core::fmt::Debug;
        use core::pin::Pin;
        use fluxion_core::{into_stream::IntoStream, Fluxion, StreamItem};
        use futures::task::{Context, Poll};
        use futures::{Stream, StreamExt};

        pub trait OrderedStreamExt<T>: Stream<Item = StreamItem<T>> + Sized
        where
            T: Fluxion,
            T::Inner: Debug + Ord $($inner_bounds)* + 'static,
            T::Timestamp: Debug + Ord $($timestamp_bounds)* + 'static,
        {
            fn ordered_merge<IS>(self, others: Vec<IS>) -> impl Stream<Item = StreamItem<T>>
            where
                IS: IntoStream<Item = StreamItem<T>>,
                IS::Stream: Stream<Item = StreamItem<T>> + $($stream_bounds)* 'static;
        }

        impl<T, S> OrderedStreamExt<T> for S
        where
            T: Fluxion,
            T::Inner: Debug + Ord $($inner_bounds)* + 'static,
            T::Timestamp: Debug + Ord $($timestamp_bounds)* + 'static,
            S: Stream<Item = StreamItem<T>> + $($stream_bounds)* 'static,
        {
            fn ordered_merge<IS>(self, others: Vec<IS>) -> impl Stream<Item = StreamItem<T>>
            where
                IS: IntoStream<Item = StreamItem<T>>,
                IS::Stream: Stream<Item = StreamItem<T>> + $($stream_bounds)* 'static,
            {
                let mut all_streams = vec![Box::pin(self) as $($boxed_stream)*];
                for into_stream in others {
                    let stream = into_stream.into_stream();
                    all_streams.push(Box::pin(stream));
                }

                StreamExt::map(
                    OrderedMergeWithImmediateErrorsIndexed::new(all_streams),
                    |(item, _index)| item,
                )
            }
        }

        pub fn ordered_merge_with_index<T>(
            streams: Vec<$($boxed_stream)*>,
        ) -> impl Stream<Item = (StreamItem<T>, usize)>
        where
            T: Fluxion + Unpin,
            T::Inner: Debug + Ord $($inner_bounds)* + 'static,
            T::Timestamp: Debug + Ord $($timestamp_bounds)* + 'static,
        {
            OrderedMergeWithImmediateErrorsIndexed::new(streams)
        }

        struct OrderedMergeWithImmediateErrorsIndexed<T>
        where
            T: Fluxion,
            T::Inner: Debug + Ord $($inner_bounds)* + 'static,
            T::Timestamp: Debug + Ord $($timestamp_bounds)* + 'static,
        {
            streams: Vec<$($boxed_stream)*>,
            buffered: Vec<Option<T>>,
        }

        impl<T> OrderedMergeWithImmediateErrorsIndexed<T>
        where
            T: Fluxion,
            T::Inner: Debug + Ord $($inner_bounds)* + 'static,
            T::Timestamp: Debug + Ord $($timestamp_bounds)* + 'static,
        {
            fn new(streams: Vec<$($boxed_stream)*>) -> Self {
                let count = streams.len();
                let buffered = (0..count).map(|_| None).collect();
                Self { streams, buffered }
            }
        }

        impl<T> Stream for OrderedMergeWithImmediateErrorsIndexed<T>
        where
            T: Fluxion + Unpin,
            T::Inner: Debug + Ord $($inner_bounds)* + 'static,
            T::Timestamp: Debug + Ord $($timestamp_bounds)* + 'static,
        {
            type Item = (StreamItem<T>, usize);

            fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                let mut any_pending = false;

                for i in 0..self.streams.len() {
                    if self.buffered[i].is_none() {
                        match self.streams[i].as_mut().poll_next(cx) {
                            Poll::Ready(Some(StreamItem::Error(e))) => {
                                return Poll::Ready(Some((StreamItem::Error(e), i)));
                            }
                            Poll::Ready(Some(StreamItem::Value(item))) => {
                                self.buffered[i] = Some(item);
                            }
                            Poll::Ready(None) => {}
                            Poll::Pending => {
                                any_pending = true;
                            }
                        }
                    }
                }

                let mut min_idx = None;
                let mut min_val: Option<&T> = None;

                for (i, item) in self.buffered.iter().enumerate() {
                    if let Some(val) = item {
                        let should_update = min_val.is_none_or(|curr_val| val < curr_val);

                        if should_update {
                            min_idx = Some(i);
                            min_val = Some(val);
                        }
                    }
                }

                if let Some(idx) = min_idx {
                    if let Some(item) = self.buffered[idx].take() {
                        Poll::Ready(Some((StreamItem::Value(item), idx)))
                    } else {
                        unreachable!("min_idx is only Some when buffered[idx] is Some")
                    }
                } else if any_pending {
                    Poll::Pending
                } else {
                    Poll::Ready(None)
                }
            }
        }
    };
}
