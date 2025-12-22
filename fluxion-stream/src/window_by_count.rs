// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Window-by-count operator that batches stream items into fixed-size chunks.
//!
//! This module provides the [`window_by_count`](WindowByCountExt::window_by_count) operator
//! that groups consecutive stream items into vectors of a specified size.
//!
//! # Overview
//!
//! The `window_by_count` operator collects items into windows (batches) of a fixed size.
//! When the window is full, it emits a `Vec` containing all items in that window.
//! On stream completion, any partial window is also emitted.
//!
//! # Basic Usage
//!
//! ```
//! use fluxion_stream::prelude::*;
//! use fluxion_test_utils::{Sequenced, test_channel};
//! use futures::StreamExt;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let (tx, stream) = test_channel::<Sequenced<i32>>();
//!
//! let mut windowed = stream.window_by_count::<Sequenced<Vec<i32>>>(3);
//!
//! tx.unbounded_send(Sequenced::new(1)).unwrap();
//! tx.unbounded_send(Sequenced::new(2)).unwrap();
//! tx.unbounded_send(Sequenced::new(3)).unwrap();  // Window complete!
//! tx.unbounded_send(Sequenced::new(4)).unwrap();
//! tx.unbounded_send(Sequenced::new(5)).unwrap();
//! drop(tx);  // Partial window [4, 5] emitted on completion
//!
//! // First window: [1, 2, 3]
//! let window1 = windowed.next().await.unwrap().unwrap().into_inner();
//! assert_eq!(window1, vec![1, 2, 3]);
//!
//! // Second window (partial): [4, 5]
//! let window2 = windowed.next().await.unwrap().unwrap().into_inner();
//! assert_eq!(window2, vec![4, 5]);
//! # }
//! ```
//!
//! # Use Cases
//!
//! - **Batch processing**: Process items in groups for efficiency
//! - **Micro-batching**: Balance latency and throughput in data pipelines
//! - **Aggregation windows**: Collect data for periodic analysis
//! - **Protocol framing**: Group bytes or messages into frames
//!
//! # Error Handling
//!
//! When an error occurs, the current partial window is discarded and the error
//! is propagated immediately. This ensures clean error boundaries without
//! emitting potentially incomplete data.

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt::Debug;
use core::mem::take;
use fluxion_core::fluxion_mutex::Mutex;
use fluxion_core::{Fluxion, StreamItem};
use futures::{future::ready, Stream, StreamExt};

/// Extension trait providing the [`window_by_count`](Self::window_by_count) operator.
///
/// This trait is implemented for all streams of [`StreamItem<T>`] where `T` implements [`Fluxion`].
pub trait WindowByCountExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    /// Groups consecutive items into fixed-size windows (batches).
    ///
    /// Collects items into vectors of size `n`. When `n` items have been collected,
    /// emits a `Vec<T::Inner>` with the timestamp of the last item in the window.
    /// On stream completion, any remaining items are emitted as a partial window.
    ///
    /// # Type Parameters
    ///
    /// - `Out`: The output wrapper type (must implement `Fluxion` with `Inner = Vec<T::Inner>`)
    ///
    /// # Arguments
    ///
    /// * `n` - The window size. Must be at least 1.
    ///
    /// # Panics
    ///
    /// Panics if `n` is 0.
    ///
    /// # Behavior
    ///
    /// - **Values**: Accumulated until window is full, then emitted as `Vec<T::Inner>`
    /// - **Errors**: Clear the current window and propagate the error immediately
    /// - **Completion**: Emit any partial window before completing
    /// - **Timestamps**: Output uses the timestamp of the **last** item in each window
    ///
    /// # Output Type
    ///
    /// Returns a stream of `Out` (typically `Sequenced<Vec<T::Inner>>`) where:
    /// - The inner value is a `Vec` containing the window items
    /// - The timestamp is from the last item added to that window
    ///
    /// # Examples
    ///
    /// ## Complete Windows
    ///
    /// ```rust
    /// use fluxion_stream::{WindowByCountExt, IntoFluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// let (tx, rx) = futures::channel::mpsc::unbounded();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// let mut windowed = stream.window_by_count::<Sequenced<Vec<i32>>>(2);
    ///
    /// tx.unbounded_send(Sequenced::new(1)).unwrap();
    /// tx.unbounded_send(Sequenced::new(2)).unwrap();
    /// tx.unbounded_send(Sequenced::new(3)).unwrap();
    /// tx.unbounded_send(Sequenced::new(4)).unwrap();
    /// drop(tx);
    ///
    /// assert_eq!(windowed.next().await.unwrap().unwrap().into_inner(), vec![1, 2]);
    /// assert_eq!(windowed.next().await.unwrap().unwrap().into_inner(), vec![3, 4]);
    /// assert!(windowed.next().await.is_none());  // Stream complete
    /// # }
    /// ```
    ///
    /// ## Partial Window on Completion
    ///
    /// ```rust
    /// use fluxion_stream::{WindowByCountExt, IntoFluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// let (tx, rx) = futures::channel::mpsc::unbounded();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// let mut windowed = stream.window_by_count::<Sequenced<Vec<i32>>>(3);
    ///
    /// tx.unbounded_send(Sequenced::new(1)).unwrap();
    /// tx.unbounded_send(Sequenced::new(2)).unwrap();
    /// drop(tx);  // Only 2 items, window size is 3
    ///
    /// // Partial window emitted on completion
    /// assert_eq!(windowed.next().await.unwrap().unwrap().into_inner(), vec![1, 2]);
    /// assert!(windowed.next().await.is_none());
    /// # }
    /// ```
    ///
    /// ## Timestamp Preservation
    ///
    /// ```rust
    /// use fluxion_stream::{WindowByCountExt, IntoFluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use fluxion_core::HasTimestamp;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// let (tx, rx) = futures::channel::mpsc::unbounded();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// let mut windowed = stream.window_by_count::<Sequenced<Vec<i32>>>(2);
    ///
    /// // Items with explicit timestamps
    /// tx.unbounded_send(Sequenced::from((10, 100))).unwrap();  // value=10, ts=100
    /// tx.unbounded_send(Sequenced::from((20, 200))).unwrap();  // value=20, ts=200
    ///
    /// let window = windowed.next().await.unwrap().unwrap();
    /// // Window timestamp is from the LAST item (200)
    /// assert_eq!(window.timestamp(), 200);
    /// assert_eq!(window.into_inner(), vec![10, 20]);
    /// # }
    /// ```
    ///
    /// # Performance
    ///
    /// - O(1) time per item (amortized)
    /// - O(n) memory where n is the window size
    /// - Single allocation per window
    ///
    /// # See Also
    ///
    /// - [`scan_ordered`](crate::ScanOrderedExt::scan_ordered) - General stateful accumulation
    /// - [`take_items`](crate::TakeItemsExt::take_items) - Limit total items
    fn window_by_count<Out>(self, n: usize) -> impl Stream<Item = StreamItem<Out>>
    where
        Out: Fluxion<Inner = Vec<T::Inner>>,
        Out::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
        Out::Timestamp: From<T::Timestamp> + Debug + Ord + Send + Sync + Copy + 'static;
}

impl<S, T> WindowByCountExt<T> for S
where
    S: Stream<Item = StreamItem<T>> + Send + 'static,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    fn window_by_count<Out>(self, n: usize) -> impl Stream<Item = StreamItem<Out>>
    where
        Out: Fluxion<Inner = Vec<T::Inner>>,
        Out::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
        Out::Timestamp: From<T::Timestamp> + Debug + Ord + Send + Sync + Copy + 'static,
    {
        assert!(n >= 1, "window_by_count: window size must be at least 1");

        // State: (buffer, last_timestamp)
        let state = Arc::new(Mutex::new((Vec::with_capacity(n), None::<T::Timestamp>)));

        // Use filter_map to accumulate and emit when window is full
        // We need to handle the completion case separately using chain
        let window_size = n;
        let state_clone = Arc::clone(&state);

        let main_stream = self.filter_map(move |item| {
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
}
