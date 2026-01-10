// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Scan-ordered operator for single-threaded runtimes.

use core::fmt::Debug;
use fluxion_core::{Fluxion, StreamItem};
use fluxion_stream_core::scan_ordered::scan_ordered_impl;
use futures::Stream;

/// Extension trait providing the `scan_ordered` operator for streams.
///
/// This operator accumulates state across stream items, emitting intermediate
/// accumulated values. It's similar to `Iterator::fold` but emits a result
/// for each input item rather than just the final result.
pub trait ScanOrderedExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
{
    /// Accumulates state across stream items, emitting intermediate results.
    ///
    /// The `scan_ordered` operator maintains an accumulator value that is updated for each
    /// input item. For each input, it calls the accumulator function with a mutable
    /// reference to the current state and the input value, producing an output value.
    ///
    /// # Type Parameters
    ///
    /// - `Acc`: The accumulator type (internal state)
    /// - `Out`: The output item type (must implement `Fluxion`)
    /// - `F`: The accumulator function
    ///
    /// # Behavior
    ///
    /// - **Timestamp Preservation**: Output items preserve the timestamp of their source items
    /// - **State Accumulation**: The accumulator state persists across all items
    /// - **Error Handling**: Errors are propagated immediately without affecting accumulator state
    /// - **Type Transformation**: Can transform input type to different output type
    ///
    /// # Arguments
    ///
    /// * `initial` - Initial accumulator value
    /// * `accumulator` - Function that updates state and produces output: `FnMut(&mut Acc, &T::Inner) -> Out::Inner`
    ///
    /// # Examples
    ///
    /// ## Running Sum
    ///
    /// ```rust
    /// use fluxion_stream_single::{ScanOrderedExt, IntoFluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// let (tx, rx) = async_channel::unbounded();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// // Accumulate sum
    /// let mut sums = stream.scan_ordered::<Sequenced<i32>, _, _>(0, |acc, val| {
    ///     *acc += val;
    ///     *acc
    /// });
    ///
    /// tx.try_send(Sequenced::new(10)).unwrap();
    /// tx.try_send(Sequenced::new(20)).unwrap();
    ///
    /// assert_eq!(sums.next().await.unwrap().unwrap().into_inner(), 10);  // 0 + 10
    /// assert_eq!(sums.next().await.unwrap().unwrap().into_inner(), 30);  // 10 + 20
    /// # }
    /// ```
    ///
    /// # See Also
    ///
    /// - [`MapOrderedExt::map_ordered`](crate::MapOrderedExt::map_ordered) - Stateless transformation
    /// - [`CombineWithPreviousExt::combine_with_previous`](crate::CombineWithPreviousExt::combine_with_previous) - Simple stateful pairing
    fn scan_ordered<Out, Acc, F>(
        self,
        initial: Acc,
        accumulator: F,
    ) -> impl Stream<Item = StreamItem<Out>>
    where
        Acc: 'static,
        Out: Fluxion,
        Out::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
        Out::Timestamp: From<T::Timestamp> + Debug + Ord + Send + Sync + Unpin + Copy + 'static,
        F: FnMut(&mut Acc, &T::Inner) -> Out::Inner + 'static;
}

impl<T, S> ScanOrderedExt<T> for S
where
    S: Stream<Item = StreamItem<T>> + Sized + 'static,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
{
    fn scan_ordered<Out, Acc, F>(
        self,
        initial: Acc,
        accumulator: F,
    ) -> impl Stream<Item = StreamItem<Out>>
    where
        Acc: 'static,
        Out: Fluxion,
        Out::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
        Out::Timestamp: From<T::Timestamp> + Debug + Ord + Send + Sync + Unpin + Copy + 'static,
        F: FnMut(&mut Acc, &T::Inner) -> Out::Inner + 'static,
    {
        scan_ordered_impl(self, initial, accumulator)
    }
}
