// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! On-error operator for error handling with handler functions.

use fluxion_core::{FluxionError, StreamItem};
use futures::future::ready;
use futures::{Stream, StreamExt};

/// Handle errors in the stream with a handler function.
///
/// The handler receives a reference to each error and returns:
/// - `true` to consume the error (remove from stream)
/// - `false` to propagate the error downstream
///
/// Multiple `on_error` operators can be chained to implement the
/// Chain of Responsibility pattern for error handling.
///
/// # Arguments
///
/// * `stream` - The source stream
/// * `handler` - A function that receives each error and returns true to consume it
///
/// # Examples
///
/// ```rust
/// use fluxion_stream_core::on_error::on_error_impl;
/// use fluxion_core::{FluxionError, StreamItem};
/// use fluxion_test_utils::Sequenced;
/// use futures::{StreamExt, pin_mut};
///
/// # async fn example() {
/// let (tx, rx) = async_channel::unbounded();
///
/// let stream = on_error_impl(rx.map(StreamItem::Value), |err| {
///     eprintln!("Error: {}", err);
///     true // Consume all errors
/// });
/// pin_mut!(stream);
///
/// tx.try_send(Sequenced::new(1)).unwrap();
/// assert_eq!(stream.next().await.unwrap().unwrap().into_inner(), 1);
/// # }
/// ```
pub fn on_error_impl<S, T, F>(stream: S, mut handler: F) -> impl Stream<Item = StreamItem<T>>
where
    S: Stream<Item = StreamItem<T>>,
    F: FnMut(&FluxionError) -> bool,
{
    stream.filter_map(move |item| {
        ready(match item {
            StreamItem::Error(err) => {
                if handler(&err) {
                    // Error handled, skip it
                    None
                } else {
                    // Error not handled, propagate
                    Some(StreamItem::Error(err))
                }
            }
            other => Some(other),
        })
    })
}
