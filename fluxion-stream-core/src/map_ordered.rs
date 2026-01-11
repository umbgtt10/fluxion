// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Map-ordered operator for temporal transformation.

use fluxion_core::StreamItem;
use futures::{Stream, StreamExt};

/// Maps each item to a new value while preserving temporal ordering.
///
/// # Arguments
///
/// * `stream` - The source stream
/// * `f` - Function that transforms each item of type `T` into type `U`
///
/// # Examples
///
/// ```rust
/// use fluxion_stream_core::map_ordered::map_ordered_impl;
/// use fluxion_core::StreamItem;
/// use fluxion_test_utils::Sequenced;
/// use futures::{StreamExt, pin_mut};
///
/// # async fn example() {
/// let (tx, rx) = async_channel::unbounded();
/// let stream = rx.map(StreamItem::Value);
///
/// let mapped = map_ordered_impl(stream, |x: Sequenced<i32>| Sequenced::new(x.into_inner() * 2));
/// pin_mut!(mapped);
///
/// tx.try_send(Sequenced::new(5)).unwrap();
/// assert_eq!(mapped.next().await.unwrap().unwrap().into_inner(), 10);
/// # }
/// ```
pub fn map_ordered_impl<S, T, U, F>(stream: S, mut f: F) -> impl Stream<Item = StreamItem<U>>
where
    S: Stream<Item = StreamItem<T>>,
    F: FnMut(T) -> U,
{
    stream.map(move |item| item.map(&mut f))
}
