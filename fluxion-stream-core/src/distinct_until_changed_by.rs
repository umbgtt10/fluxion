// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Distinct-until-changed-by operator with custom comparison.

use alloc::boxed::Box;
use alloc::sync::Arc;
use core::fmt::Debug;
use fluxion_core::fluxion_mutex::Mutex;
use fluxion_core::StreamItem;
use futures::stream::StreamExt;
use futures::Stream;

/// Filter consecutive duplicates using a custom comparison function.
///
/// # Arguments
///
/// * `stream` - The source stream
/// * `compare` - Function returning true if values are considered equal (filtered), false if different (emitted)
///
/// # Examples
///
/// ```rust
/// use fluxion_stream_core::distinct_until_changed_by::distinct_until_changed_by_impl;
/// use fluxion_core::StreamItem;
/// use fluxion_test_utils::Sequenced;
/// use futures::StreamExt;
///
/// # async fn example() {
/// let (tx, rx) = async_channel::unbounded();
/// let stream = rx.map(StreamItem::Value);
///
/// // Case-insensitive comparison
/// let mut distinct = distinct_until_changed_by_impl(stream, |a: &String, b: &String| {
///     a.to_lowercase() == b.to_lowercase()
/// });
///
/// tx.try_send(Sequenced::new("hello".to_string())).unwrap();
/// assert_eq!(distinct.next().await.unwrap().unwrap().into_inner(), "hello");
/// # }
/// ```
pub fn distinct_until_changed_by_impl<S, T, Inner, F>(
    stream: S,
    compare: F,
) -> impl Stream<Item = StreamItem<T>>
where
    S: Stream<Item = StreamItem<T>>,
    T: Clone + 'static,
    T: fluxion_core::Timestamped<Inner = Inner>,
    Inner: Clone + Debug,
    F: Fn(&Inner, &Inner) -> bool + 'static,
{
    let last_value: Arc<Mutex<Option<Inner>>> = Arc::new(Mutex::new(None));
    let compare = Arc::new(compare);

    let stream = stream.filter_map(move |item| {
        let last_value = Arc::clone(&last_value);
        let compare = Arc::clone(&compare);

        async move {
            match item {
                StreamItem::Value(value) => {
                    let current_inner = value.clone().into_inner();

                    let mut last = last_value.lock();

                    // Check if this value is different from the last emitted value
                    let should_emit = match last.as_ref() {
                        None => true, // First value, always emit
                        Some(prev) => !compare(&current_inner, prev),
                    };

                    if should_emit {
                        // Update last value
                        *last = Some(current_inner);

                        // Preserve original timestamp
                        Some(StreamItem::Value(value))
                    } else {
                        None // Filter out duplicate
                    }
                }
                StreamItem::Error(e) => Some(StreamItem::Error(e)), // Propagate errors
            }
        }
    });

    Box::pin(stream)
}
