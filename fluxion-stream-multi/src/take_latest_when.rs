// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Extension trait for `take_latest_when` operator (single-threaded runtimes).
//!
//! This operator samples the latest value from a source stream whenever a filter
//! stream emits a value that passes a predicate.

use core::fmt::Debug;
use fluxion_core::into_stream::IntoStream;
use fluxion_core::{Fluxion, StreamItem};
use futures::Stream;

/// Extension trait providing the `take_latest_when` operator for timestamped streams.
///
/// This operator samples the latest value from a source stream whenever a filter
/// stream emits a value that passes a predicate.
///
/// Available on single-threaded runtimes (WASM, Embassy) with relaxed `Send`-only bounds.
pub trait TakeLatestWhenExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
{
    /// Emits the latest value from the source stream when the filter stream emits a passing value.
    ///
    /// This operator maintains the latest value from the source stream and the latest value
    /// from the filter stream. When the filter stream emits a value that passes the predicate,
    /// the operator emits the most recent source value (if one exists).
    ///
    /// # Behavior
    ///
    /// - Source stream values are cached but don't trigger emissions
    /// - Filter stream values are evaluated with the predicate
    /// - Emission occurs when: filter predicate returns `true` AND a source value exists
    /// - Emitted values preserve the temporal order of the triggering filter value
    ///
    /// # Arguments
    ///
    /// * `filter_stream` - Stream whose values control when to sample the source
    /// * `filter` - Predicate function applied to filter stream values. Returns `true` to emit.
    ///
    /// # Returns
    ///
    /// A stream of `T` containing source values sampled when the filter permits.
    ///
    /// # Type Parameters
    ///
    /// * `IS` - Type that can be converted into a stream compatible with this stream
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_stream_multi::TakeLatestWhenExt;
    /// use fluxion_test_utils::{Sequenced, helpers::unwrap_stream, unwrap_value, test_channel};
    /// use fluxion_core::Timestamped as TimestampedTrait;
    ///
    /// # async fn example() {
    /// // Create channels
    /// let (tx_data, data_stream) = test_channel::<Sequenced<i32>>();
    /// let (tx_trigger, trigger_stream) = test_channel::<Sequenced<i32>>();
    ///
    /// // Combine streams
    /// let mut sampled = data_stream.take_latest_when(
    ///     trigger_stream,
    ///     |trigger_value| *trigger_value > 0  // Trigger when value > 0
    /// );
    ///
    /// // Send values
    /// tx_data.unbounded_send((100, 1).into()).unwrap();
    /// tx_trigger.unbounded_send((1, 2).into()).unwrap();  // Trigger
    ///
    /// // Assert - trigger emits the latest data value
    /// let result = unwrap_value(Some(unwrap_stream(&mut sampled, 500).await));
    /// assert_eq!(result.value, 100);
    /// # }
    /// ```
    ///
    /// # Use Cases
    ///
    /// - Sampling sensor data on timer ticks
    /// - Gate pattern: emit values only when enabled
    /// - Throttling with external control signals
    ///
    /// # Errors
    ///
    /// Emits `StreamItem::Error` when internal lock acquisition fails:
    ///
    /// - **Source buffer lock error**: Returns `FluxionError::LockError` if the source buffer
    ///   mutex is poisoned
    /// - **Filter state lock error**: Returns `FluxionError::LockError` if the filter state
    ///   mutex is poisoned
    /// - **Emit flag lock error**: Returns `FluxionError::LockError` if the emission flag
    ///   mutex is poisoned
    ///
    /// All errors are non-fatal - the stream continues processing subsequent items.
    ///
    /// See the [Error Handling Guide](../../docs/ERROR-HANDLING.md)
    /// for handling strategies.
    ///
    /// # See Also
    ///
    /// - [`emit_when`](crate::EmitWhenExt::emit_when) - Gates source emissions rather than sampling
    /// - [`with_latest_from`](crate::WithLatestFromExt::with_latest_from) - Emits on primary, samples secondary
    fn take_latest_when<IS>(
        self,
        filter_stream: IS,
        filter: impl Fn(&T::Inner) -> bool + Send + Sync + Unpin + 'static,
    ) -> impl Stream<Item = StreamItem<T>> + Send
    where
        IS: IntoStream<Item = StreamItem<T>>,
        IS::Stream: Send + Sync + 'static;
}

impl<T, S> TakeLatestWhenExt<T> for S
where
    S: Stream<Item = StreamItem<T>> + Send + Sync + Unpin + Unpin + 'static,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Unpin + Copy + 'static,
{
    fn take_latest_when<IS>(
        self,
        filter_stream: IS,
        filter: impl Fn(&T::Inner) -> bool + Send + Sync + Unpin + 'static,
    ) -> impl Stream<Item = StreamItem<T>> + Send
    where
        IS: IntoStream<Item = StreamItem<T>>,
        IS::Stream: Send + Sync + 'static,
    {
        fluxion_stream_core::take_latest_when::take_latest_when_impl(
            self,
            filter_stream.into_stream(),
            filter,
        )
    }
}
