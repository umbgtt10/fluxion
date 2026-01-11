// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Window-by-count operator for single-threaded runtimes.

use core::fmt::Debug;
use fluxion_core::{Fluxion, StreamItem};
use fluxion_stream_core::window_by_count::window_by_count_impl;
use futures::Stream;

/// Extension trait providing the `window_by_count` operator.
pub trait WindowByCountExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
{
    /// Groups consecutive items into fixed-size windows (batches).
    ///
    /// Collects items into vectors of size `n`. When `n` items have been collected,
    /// emits a `Vec<T::Inner>` with the timestamp of the last item in the window.
    /// On stream completion, any remaining items are emitted as a partial window.
    ///
    /// # Arguments
    ///
    /// * `n` - The window size. Must be at least 1.
    ///
    /// # Panics
    ///
    /// Panics if `n` is 0.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_stream_multi::{WindowByCountExt, IntoFluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// let (tx, rx) = async_channel::unbounded();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// let mut windowed = stream.window_by_count::<Sequenced<Vec<i32>>>(3);
    ///
    /// tx.try_send(Sequenced::new(1)).unwrap();
    /// tx.try_send(Sequenced::new(2)).unwrap();
    /// tx.try_send(Sequenced::new(3)).unwrap();
    /// drop(tx);
    ///
    /// assert_eq!(windowed.next().await.unwrap().unwrap().into_inner(), vec![1, 2, 3]);
    /// # }
    /// ```
    fn window_by_count<Out>(self, n: usize) -> impl Stream<Item = StreamItem<Out>>
    where
        Self: Unpin + 'static,
        Out: Fluxion<Inner = Vec<T::Inner>>,
        Out::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
        Out::Timestamp: From<T::Timestamp> + Debug + Ord + Send + Sync + Unpin + Copy + 'static;
}

impl<S, T> WindowByCountExt<T> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
{
    fn window_by_count<Out>(self, n: usize) -> impl Stream<Item = StreamItem<Out>>
    where
        Self: Unpin + 'static,
        Out: Fluxion<Inner = Vec<T::Inner>>,
        Out::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
        Out::Timestamp: From<T::Timestamp> + Debug + Ord + Send + Sync + Unpin + Copy + 'static,
    {
        window_by_count_impl(self, n)
    }
}
