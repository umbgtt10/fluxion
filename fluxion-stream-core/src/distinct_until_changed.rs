// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Distinct-until-changed operator that filters consecutive duplicates.

use alloc::boxed::Box;
use alloc::sync::Arc;
use core::fmt::Debug;
use fluxion_core::fluxion_mutex::Mutex;
use fluxion_core::StreamItem;
use futures::stream::StreamExt;
use futures::Stream;

/// Emits values only when they differ from the previous emitted value.
///
/// This operator maintains the last emitted inner value and compares each new
/// value against it. Only values that differ are emitted. The comparison is
/// performed on the inner values (`T::Inner`), and timestamps from the original
/// values are preserved.
///
/// # Behavior
///
/// - First value is always emitted (no previous value to compare)
/// - Subsequent values are compared to the last emitted value
/// - Only values where `current != previous` are emitted
/// - Timestamps are preserved from the original incoming values
/// - Errors are always propagated immediately
///
/// # Arguments
///
/// * `stream` - The source stream
///
/// # Returns
///
/// A stream that only emits values different from the previous emission.
///
/// # Error Handling
///
/// Errors from the source stream are always propagated unchanged, regardless
/// of deduplication state.
///
/// # Examples
///
/// ```rust
/// use fluxion_stream_core::distinct_until_changed::distinct_until_changed_impl;
/// use fluxion_core::StreamItem;
/// use fluxion_test_utils::Sequenced;
/// use futures::StreamExt;
///
/// # async fn example() {
/// let (tx, rx) = async_channel::unbounded();
///
/// let mut distinct = distinct_until_changed_impl(rx.map(StreamItem::Value));
///
/// // Send: 1, 1, 2, 2, 2, 3, 2
/// tx.try_send(Sequenced::new(1)).unwrap();
/// tx.try_send(Sequenced::new(1)).unwrap(); // Filtered
/// tx.try_send(Sequenced::new(2)).unwrap();
/// tx.try_send(Sequenced::new(2)).unwrap(); // Filtered
/// tx.try_send(Sequenced::new(2)).unwrap(); // Filtered
/// tx.try_send(Sequenced::new(3)).unwrap();
/// tx.try_send(Sequenced::new(2)).unwrap(); // Emitted (different from 3)
///
/// // Output: 1, 2, 3, 2
/// assert_eq!(distinct.next().await.unwrap().unwrap().into_inner(), 1);
/// assert_eq!(distinct.next().await.unwrap().unwrap().into_inner(), 2);
/// assert_eq!(distinct.next().await.unwrap().unwrap().into_inner(), 3);
/// assert_eq!(distinct.next().await.unwrap().unwrap().into_inner(), 2);
/// # }
/// ```
pub fn distinct_until_changed_impl<S, T, Inner>(stream: S) -> impl Stream<Item = StreamItem<T>>
where
    S: Stream<Item = StreamItem<T>>,
    T: Clone + 'static,
    T: fluxion_core::Timestamped<Inner = Inner>,
    Inner: Clone + Debug + PartialEq,
{
    let last_value: Arc<Mutex<Option<Inner>>> = Arc::new(Mutex::new(None));

    let stream = stream.filter_map(move |item| {
        let last_value = Arc::clone(&last_value);
        async move {
            match item {
                StreamItem::Value(value) => {
                    let current_inner = value.clone().into_inner();

                    let mut last = last_value.lock();

                    // Check if this value is different from the last emitted value
                    let should_emit = match last.as_ref() {
                        None => true, // First value, always emit
                        Some(prev) => current_inner != *prev,
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
