// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Tap operator for side-effect observation without modifying the stream.

use fluxion_core::StreamItem;
use futures::{Stream, StreamExt};

/// Invokes a side-effect function for each value without modifying the stream.
///
/// This operator is useful for debugging, logging, or metrics collection
/// without affecting the stream's data flow.
///
/// # Arguments
///
/// * `stream` - The source stream
/// * `f` - A function that receives a reference to each value's inner type.
///   Called for side effects only; return value is ignored.
///
/// # Behavior
///
/// - **Values**: Function `f` is called with a reference to the inner value,
///   then the value passes through unchanged
/// - **Errors**: Pass through without calling `f`
/// - **Timestamps**: Preserved unchanged
/// - **Ordering**: Maintained
///
/// # Examples
///
/// ```rust
/// use fluxion_stream_core::tap::tap_impl;
/// use fluxion_core::StreamItem;
/// use fluxion_test_utils::Sequenced;
/// use futures::StreamExt;
///
/// # async fn example() {
/// let (tx, rx) = async_channel::unbounded();
///
/// let mut tapped = tap_impl(rx.map(StreamItem::Value), |x| println!("Value: {}", x));
///
/// tx.try_send(Sequenced::new(42)).unwrap();
/// let result = tapped.next().await.unwrap().unwrap();
/// assert_eq!(result.into_inner(), 42);
/// # }
/// ```
pub fn tap_impl<S, T, Inner, F>(stream: S, mut f: F) -> impl Stream<Item = StreamItem<T>>
where
    S: Stream<Item = StreamItem<T>>,
    T: Clone + fluxion_core::Timestamped<Inner = Inner>,
    F: FnMut(&Inner),
{
    stream.map(move |item| {
        if let StreamItem::Value(value) = &item {
            f(&value.clone().into_inner());
        }
        item
    })
}
