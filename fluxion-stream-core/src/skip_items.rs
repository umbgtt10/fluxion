// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Skip-items operator that discards the first n items from a stream.

use fluxion_core::StreamItem;
use futures::{Stream, StreamExt};

/// Skips the first `n` items from the stream.
///
/// The first `n` items will be discarded, and only items after that will be emitted.
/// If the stream has fewer than `n` items, no items will be emitted.
///
/// # Arguments
///
/// * `stream` - The source stream
/// * `n` - The number of items to skip.
///
/// # Returns
///
/// A new stream that skips the first `n` items from the source stream.
///
/// # Error Handling
///
/// **Important:** Errors count as items for the purpose of skipping.
/// If you want to skip 2 items and the stream emits `[Error, Error, Value]`,
/// both errors will be skipped and only the value will be emitted.
///
/// Use `.on_error()` before `skip_items()` if you want to handle errors before they
/// are counted in the skip count.
///
/// # Examples
///
/// ```rust
/// use fluxion_stream_core::skip_items::skip_items_impl;
/// use fluxion_core::StreamItem;
/// use fluxion_test_utils::Sequenced;
/// use futures::StreamExt;
///
/// # async fn example() {
/// let (tx, rx) = async_channel::unbounded();
///
/// let mut after_first_two = skip_items_impl(rx, 2);
///
/// tx.try_send(StreamItem::Value(Sequenced::new(1))).unwrap();
/// tx.try_send(StreamItem::Value(Sequenced::new(2))).unwrap();
/// tx.try_send(StreamItem::Value(Sequenced::new(3))).unwrap();
///
/// // First two are skipped
/// assert_eq!(after_first_two.next().await.unwrap().unwrap().into_inner(), 3);
/// # }
/// ```
pub fn skip_items_impl<S, T>(stream: S, n: usize) -> impl Stream<Item = StreamItem<T>>
where
    S: Stream<Item = StreamItem<T>>,
{
    StreamExt::skip(stream, n)
}
