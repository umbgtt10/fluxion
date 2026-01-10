// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Filter-ordered operator for temporal filtering.

use fluxion_core::StreamItem;
use futures::future::ready;
use futures::Stream;
use futures::StreamExt;

/// Filter items based on a predicate while preserving temporal ordering.
///
/// # Arguments
///
/// * `stream` - The source stream
/// * `predicate` - Function that takes `&Inner` and returns `true` to keep the item
///
/// # Examples
///
/// ```rust
/// use fluxion_stream_core::filter_ordered::filter_ordered_impl;
/// use fluxion_core::StreamItem;
/// use fluxion_test_utils::Sequenced;
/// use futures::StreamExt;
///
/// # async fn example() {
/// let (tx, rx) = async_channel::unbounded();
/// let stream = rx.map(StreamItem::Value);
///
/// // Filter for even numbers
/// let mut evens = filter_ordered_impl(stream, |&n| n % 2 == 0);
///
/// tx.try_send(Sequenced::new(1)).unwrap();
/// tx.try_send(Sequenced::new(2)).unwrap();
/// assert_eq!(evens.next().await.unwrap().unwrap().into_inner(), 2);
/// # }
/// ```
pub fn filter_ordered_impl<S, T, Inner, F>(
    stream: S,
    mut predicate: F,
) -> impl Stream<Item = StreamItem<T>>
where
    S: Stream<Item = StreamItem<T>>,
    T: Clone + 'static,
    T: fluxion_core::Timestamped<Inner = Inner>,
    Inner: Clone,
    F: FnMut(&Inner) -> bool + 'static,
{
    stream.filter_map(move |item| {
        ready(match item {
            StreamItem::Value(value) if predicate(&value.clone().into_inner()) => {
                Some(StreamItem::Value(value))
            }
            StreamItem::Value(_) => None,
            StreamItem::Error(e) => Some(StreamItem::Error(e)),
        })
    })
}
