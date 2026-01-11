// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Extension trait for `take_while_with` operator (single-threaded runtimes).
//!
//! Conditionally emits elements from a source stream based on values from a
//! separate filter stream. The stream terminates when the filter condition becomes false.

use core::fmt::Debug;
use fluxion_core::{Fluxion, StreamItem};
use futures::Stream;

/// Extension trait providing the `take_while_with` operator for timestamped streams.
///
/// This operator conditionally emits elements from a source stream based on values
/// from a separate filter stream. The stream terminates when the filter condition
/// becomes false.
///
/// Available on single-threaded runtimes (WASM, Embassy) with relaxed `Send`-only bounds.
pub trait TakeWhileExt<TItem, TFilter, S>: Stream<Item = StreamItem<TItem>> + Sized
where
    TItem: Fluxion,
    TItem::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    TItem::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
    TFilter: Fluxion<Timestamp = TItem::Timestamp>,
    TFilter::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    S: Stream<Item = StreamItem<TFilter>> + Send + Sync + Unpin + 'static,
{
    /// Takes elements from the source stream while the filter predicate returns true.
    ///
    /// This operator merges the source stream with a filter stream in temporal order.
    /// It maintains the latest filter value and only emits source values when the
    /// filter predicate evaluates to `true`. Once the predicate returns `false`,
    /// the entire stream terminates.
    ///
    /// # Behavior
    ///
    /// - Source values are emitted only when latest filter passes the predicate
    /// - Filter stream updates change the gating condition
    /// - Stream terminates immediately when filter predicate returns `false`
    /// - Emitted values maintain their ordered wrapper
    ///
    /// # Arguments
    ///
    /// * `filter_stream` - Stream providing filter values that control emission
    /// * `filter` - Predicate function applied to filter values. Returns `true` to continue.
    ///
    /// # Returns
    ///
    /// A stream of source elements that are emitted while the filter condition
    /// remains true. Stream terminates when condition becomes false.
    ///
    /// # Errors
    ///
    /// This operator may produce `StreamItem::Error` in the following cases:
    ///
    /// - **Lock Errors**: When acquiring the combined state lock fails (e.g., due to lock poisoning).
    ///   These are transient errors - the stream continues processing and may succeed on subsequent items.
    ///
    /// Lock errors are typically non-fatal and indicate temporary contention. The operator will continue
    /// processing subsequent items. See the [Error Handling Guide](../docs/ERROR-HANDLING.md) for patterns
    /// on handling these errors in your application.
    ///
    /// # See Also
    ///
    /// - [`emit_when`](crate::EmitWhenExt::emit_when) - Gates emissions but doesn't terminate
    /// - [`take_latest_when`](crate::TakeLatestWhenExt::take_latest_when) - Samples on condition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_stream_multi::TakeWhileExt;
    /// use fluxion_test_utils::{Sequenced, helpers::unwrap_stream, test_channel};
    /// use fluxion_core::Timestamped as TimestampedTrait;
    ///
    /// # async fn example() {
    /// // Create channels
    /// let (tx_data, data_stream) = test_channel::<Sequenced<i32>>();
    /// let (tx_filter, filter_stream) = test_channel::<Sequenced<bool>>();
    ///
    /// // Take while filter is true
    /// let mut gated = data_stream.take_while_with(
    ///     filter_stream,
    ///     |enabled| *enabled  // Continue while true
    /// );
    ///
    /// // Send values
    /// tx_filter.unbounded_send((true, 1).into()).unwrap();  // Enable
    /// tx_data.unbounded_send((100, 2).into()).unwrap();
    /// tx_filter.unbounded_send((false, 3).into()).unwrap(); // Disable - stream terminates
    ///
    /// // Only one value before termination
    /// let result = unwrap_stream(&mut gated, 500).await;
    /// assert_eq!(result.unwrap().value, 100);
    /// assert!(unwrap_stream(&mut gated, 500).await.is_none()); // Stream ended
    /// # }
    /// ```
    ///
    /// # Use Cases
    ///
    /// - Emergency stop mechanism for data streams
    /// - Time-bounded stream processing
    /// - Conditional data forwarding with external control
    fn take_while_with(
        self,
        filter_stream: S,
        filter: impl Fn(&TFilter::Inner) -> bool + Send + Sync + Unpin + 'static,
    ) -> impl Stream<Item = StreamItem<TItem>> + Send + Sync + Unpin;
}

impl<TItem, TFilter, S, P> TakeWhileExt<TItem, TFilter, S> for P
where
    P: Stream<Item = StreamItem<TItem>> + Send + Sync + Unpin + Unpin + 'static,
    TItem: Fluxion,
    TItem::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    TItem::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
    TFilter: Fluxion<Timestamp = TItem::Timestamp>,
    TFilter::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    S: Stream<Item = StreamItem<TFilter>> + Send + Sync + Unpin + 'static,
{
    fn take_while_with(
        self,
        filter_stream: S,
        filter: impl Fn(&TFilter::Inner) -> bool + Send + Sync + Unpin + 'static,
    ) -> impl Stream<Item = StreamItem<TItem>> + Send + Sync + Unpin {
        fluxion_stream_core::take_while_with::take_while_with_impl(self, filter_stream, filter)
    }
}
