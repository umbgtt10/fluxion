// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Scan-ordered operator for stateful accumulation.

use alloc::sync::Arc;
use fluxion_core::fluxion_mutex::Mutex;
use fluxion_core::{StreamItem, Timestamped};
use futures::{future::ready, Stream, StreamExt};

/// Accumulates state across stream items, emitting intermediate results.
///
/// # Arguments
///
/// * `stream` - The source stream
/// * `initial` - Initial accumulator value
/// * `accumulator` - Function that updates state and produces output
///
/// # Examples
///
/// ```rust
/// use fluxion_stream_core::scan_ordered::scan_ordered_impl;
/// use fluxion_core::StreamItem;
/// use fluxion_test_utils::Sequenced;
/// use futures::{StreamExt, pin_mut};
///
/// # async fn example() {
/// let (tx, rx) = async_channel::unbounded();
/// let stream = rx.map(StreamItem::Value);
///
/// // Running sum
/// let sums = scan_ordered_impl::<_, Sequenced<i32>, i32, Sequenced<i32>, i32, i32, _>(stream, 0, |acc: &mut i32, val: &i32| {
///     *acc += val;
///     *acc
/// });
/// pin_mut!(sums);
///
/// tx.try_send(Sequenced::new(10)).unwrap();
/// assert_eq!(sums.next().await.unwrap().unwrap().into_inner(), 10);
/// # }
/// ```
pub fn scan_ordered_impl<S, T, TInner, Out, OutInner, Acc, F>(
    stream: S,
    initial: Acc,
    accumulator: F,
) -> impl Stream<Item = StreamItem<Out>>
where
    S: Stream<Item = StreamItem<T>>,
    T: Clone + 'static,
    T: Timestamped<Inner = TInner>,
    TInner: Clone,
    Out: Timestamped<Inner = OutInner>,
    OutInner: Clone,
    Out::Timestamp: From<T::Timestamp>,
    F: FnMut(&mut Acc, &TInner) -> OutInner + 'static,
{
    let state = Arc::new(Mutex::new((initial, accumulator)));

    stream.then(move |item| {
        let state = Arc::clone(&state);
        ready(match item {
            StreamItem::Value(value) => {
                let timestamp = value.timestamp();
                let inner = value.into_inner();

                // Lock state and apply accumulator function
                let mut guard = state.lock();
                let (acc, ref mut f) = &mut *guard;
                let output = f(acc, &inner);
                StreamItem::Value(Out::with_timestamp(output, timestamp.into()))
            }
            StreamItem::Error(e) => {
                // Propagate error without affecting accumulator state
                StreamItem::Error(e)
            }
        })
    })
}
