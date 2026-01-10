// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Take-items operator that limits stream to first n items.

use fluxion_core::StreamItem;
use futures::{Stream, StreamExt};

/// Emits only the first `n` items from the stream, then completes.
///
/// After emitting `n` items, the stream will complete and no further items
/// will be emitted, even if the source stream continues to produce values.
///
/// # Arguments
///
/// * `stream` - The source stream
/// * `n` - The maximum number of items to emit.
///
/// # Returns
///
/// A new stream that emits at most `n` items from the source stream.
///
/// # Error Handling
///
/// **Important:** Errors count as items for the purpose of the limit.
/// If you want to take 3 values and the stream emits `[Value, Error, Value, Value, Value]`,
/// only the first 3 items will be emitted: `[Value, Error, Value]`.
///
/// Errors are propagated unchanged. Use `.on_error()` before `take_items()` if you want
/// to filter errors before counting.
///
/// # Examples
///
/// ```rust
/// use fluxion_stream_core::take_items::take_items_impl;
/// use fluxion_core::StreamItem;
/// use fluxion_test_utils::Sequenced;
/// use futures::StreamExt;
///
/// # async fn example() {
/// let (tx, rx) = async_channel::unbounded();
///
/// let mut first_two = take_items_impl(rx, 2);
///
/// tx.try_send(StreamItem::Value(Sequenced::new(1))).unwrap();
/// tx.try_send(StreamItem::Value(Sequenced::new(2))).unwrap();
/// tx.try_send(StreamItem::Value(Sequenced::new(3))).unwrap();
/// drop(tx);
///
/// assert_eq!(first_two.next().await.unwrap().unwrap().into_inner(), 1);
/// assert_eq!(first_two.next().await.unwrap().unwrap().into_inner(), 2);
/// assert!(first_two.next().await.is_none());
/// # }
/// ```
pub fn take_items_impl<S, T>(stream: S, n: usize) -> impl Stream<Item = StreamItem<T>>
where
    S: Stream<Item = StreamItem<T>>,
{
    StreamExt::take(stream, n)
}
