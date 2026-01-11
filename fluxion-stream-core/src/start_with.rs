// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Start-with operator that prepends initial values to a stream.

use alloc::vec::Vec;
use fluxion_core::StreamItem;
use futures::{stream::iter, Stream, StreamExt};

/// Prepends initial values to the stream.
///
/// The initial values will be emitted first, in the order provided, followed by
/// all values from the source stream. The initial values must have timestamps
/// that respect the temporal ordering constraints.
///
/// # Arguments
///
/// * `stream` - The source stream
/// * `initial_values` - Vector of `StreamItem<T>` to emit before the source stream.
///
/// # Returns
///
/// A new stream that emits the initial values followed by all values from the source stream.
///
/// # Error Handling
///
/// Errors in both the initial values and the source stream are propagated as-is.
/// This operator does not consume or transform errors - they flow through unchanged.
/// Use `.on_error()` before or after `start_with()` to handle errors if needed.
///
/// # Examples
///
/// ```rust
/// use fluxion_stream_core::start_with::start_with_impl;
/// use fluxion_core::StreamItem;
/// use fluxion_test_utils::Sequenced;
/// use futures::{StreamExt, pin_mut};
///
/// # async fn example() {
/// let (tx, rx) = async_channel::unbounded();
///
/// let initial = vec![
///     StreamItem::Value(Sequenced::new(1)),
///     StreamItem::Value(Sequenced::new(2)),
/// ];
///
/// let stream_with_prefix = start_with_impl(rx, initial);
/// pin_mut!(stream_with_prefix);
///
/// // Initial values come first
/// assert_eq!(stream_with_prefix.next().await.unwrap().unwrap().into_inner(), 1);
/// assert_eq!(stream_with_prefix.next().await.unwrap().unwrap().into_inner(), 2);
///
/// // Then stream values
/// tx.try_send(StreamItem::Value(Sequenced::new(3))).unwrap();
/// assert_eq!(stream_with_prefix.next().await.unwrap().unwrap().into_inner(), 3);
/// # }
/// ```
pub fn start_with_impl<S, T>(
    stream: S,
    initial_values: Vec<StreamItem<T>>,
) -> impl Stream<Item = StreamItem<T>>
where
    S: Stream<Item = StreamItem<T>>,
{
    let initial_stream = iter(initial_values);
    initial_stream.chain(stream)
}
