// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Window-by-count operator for batching items.

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::mem::take;
use fluxion_core::fluxion_mutex::Mutex;
use fluxion_core::{StreamItem, Timestamped};
use futures::{future::ready, Stream, StreamExt};

/// Groups consecutive items into fixed-size windows (batches).
///
/// # Arguments
///
/// * `stream` - The source stream
/// * `n` - The window size (must be at least 1)
///
/// # Panics
///
/// Panics if `n` is 0.
///
/// # Examples
///
/// ```rust
/// use fluxion_stream_core::window_by_count::window_by_count_impl;
/// use fluxion_core::StreamItem;
/// use fluxion_test_utils::Sequenced;
/// use futures::StreamExt;
///
/// # async fn example() {
/// let (tx, rx) = async_channel::unbounded();
/// let stream = rx.map(StreamItem::Value);
///
/// let mut windowed = window_by_count_impl::<_, _, Sequenced<Vec<i32>>>(stream, 3);
///
/// tx.try_send(Sequenced::new(1)).unwrap();
/// tx.try_send(Sequenced::new(2)).unwrap();
/// tx.try_send(Sequenced::new(3)).unwrap();
/// drop(tx);
///
/// assert_eq!(windowed.next().await.unwrap().unwrap().into_inner(), vec![1, 2, 3]);
/// # }
/// ```
pub fn window_by_count_impl<S, T, Out>(stream: S, n: usize) -> impl Stream<Item = StreamItem<Out>>
where
    S: Stream<Item = StreamItem<T>>,
    T: Clone + 'static,
    T: Timestamped,
    T::Inner: Clone,
    Out: Timestamped<Inner = Vec<T::Inner>>,
    Out::Timestamp: From<T::Timestamp>,
{
    assert!(n >= 1, "window_by_count: window size must be at least 1");

    // State: (buffer, last_timestamp)
    let state = Arc::new(Mutex::new((Vec::with_capacity(n), None::<T::Timestamp>)));

    // Use filter_map to accumulate and emit when window is full
    let window_size = n;
    let state_clone = Arc::clone(&state);

    let main_stream = stream.filter_map(move |item| {
        let state = Arc::clone(&state_clone);
        let window_size = window_size;

        ready(match item {
            StreamItem::Value(value) => {
                let timestamp = value.timestamp();
                let inner = value.into_inner();

                let mut guard = state.lock();
                let (buffer, last_ts) = &mut *guard;

                buffer.push(inner);
                *last_ts = Some(timestamp);

                if buffer.len() >= window_size {
                    let window = take(buffer);
                    *buffer = Vec::with_capacity(window_size);
                    let ts = last_ts.take().expect("timestamp must exist");
                    Some(StreamItem::Value(Out::with_timestamp(window, ts.into())))
                } else {
                    None
                }
            }
            StreamItem::Error(e) => {
                // Clear buffer and propagate error
                let mut guard = state.lock();
                let (buffer, last_ts) = &mut *guard;
                buffer.clear();
                *last_ts = None;
                Some(StreamItem::Error(e))
            }
        })
    });

    // Chain with a stream that emits partial window on completion
    let final_state = state;
    let flush_stream = futures::stream::once(async move {
        let mut guard = final_state.lock();
        let (buffer, last_ts) = &mut *guard;

        if !buffer.is_empty() {
            let window = take(buffer);
            let ts = last_ts
                .take()
                .expect("timestamp must exist for partial window");
            Some(StreamItem::Value(Out::with_timestamp(window, ts.into())))
        } else {
            None
        }
    })
    .filter_map(ready);

    Box::pin(main_stream.chain(flush_stream))
}
