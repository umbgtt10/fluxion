// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::ordered_merge::ordered_merge_with_index;
use crate::types::CombinedState;
use fluxion_core::into_stream::IntoStream;
use fluxion_core::{Fluxion, StreamItem};
use futures::{Stream, StreamExt};
use parking_lot::Mutex;
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::Arc;

/// Extension trait providing the `with_latest_from` operator for timestamped streams.
///
/// This operator combines a primary stream with a secondary stream, emitting only
/// when the primary stream emits, using the latest value from the secondary stream.
pub trait WithLatestFromExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    /// Combines elements from the primary stream (self) with the latest element from the secondary stream (other).
    ///
    /// This operator only emits when the primary stream emits. It waits until both streams
    /// have emitted at least once, then for each primary emission, it combines the primary
    /// value with the most recent secondary value.
    ///
    /// # Behavior
    ///
    /// - Emissions are triggered **only** by the primary stream (self)
    /// - Secondary stream updates are stored but don't trigger emissions
    /// - Waits until both streams have emitted at least once
    /// - Preserves temporal ordering from the primary stream
    ///
    /// # Arguments
    ///
    /// * `other` - Secondary stream whose latest value will be combined with primary emissions
    /// * `result_selector` - Function that transforms the `CombinedState` into the output type `R`.
    ///   The combined state contains `[primary_value, secondary_value]`.
    ///
    /// # Returns
    ///
    /// A stream of `R` where each emission contains the result of applying
    /// `result_selector` to the combined state, ordered by the primary stream's order.
    ///
    /// # Type Parameters
    ///
    /// * `IS` - Type that can be converted into a stream compatible with this stream
    /// * `R` - Result type produced by the `result_selector` function
    ///
    /// # Errors
    ///
    /// This operator emits `StreamItem::Error` when:
    ///
    /// - **Lock acquisition fails**: If the internal state mutex becomes poisoned, a
    ///   `FluxionError::LockError` is emitted and that item is skipped. The stream continues
    ///   processing.
    ///
    /// Errors are emitted as `StreamItem::Error` values in the output stream, allowing
    /// downstream operators to handle them appropriately.
    ///
    /// See the [Error Handling Guide](../../docs/ERROR-HANDLING.md)
    /// for recovery patterns.
    ///
    /// # See Also
    ///
    /// - [`combine_latest`](crate::CombineLatestExt::combine_latest) - Emits when any stream emits
    /// - [`emit_when`](crate::EmitWhenExt::emit_when) - Gates emissions based on filter stream
    /// - [`take_latest_when`](crate::TakeLatestWhenExt::take_latest_when) - Samples on filter condition
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_stream::WithLatestFromExt;
    /// use fluxion_test_utils::{Sequenced, helpers::unwrap_stream, unwrap_value, test_channel};
    /// use fluxion_core::Timestamped as TimestampedTrait;
    ///
    /// # async fn example() {
    /// // Create channels
    /// let (tx_primary, primary) = test_channel::<Sequenced<i32>>();
    /// let (tx_secondary, secondary) = test_channel::<Sequenced<i32>>();
    ///
    /// // Combine streams
    /// let mut combined = primary.with_latest_from(
    ///     secondary,
    ///     |state| state.clone()
    /// );
    ///
    /// // Send values
    /// tx_secondary.send((10, 1).into()).unwrap();
    /// tx_primary.send((1, 2).into()).unwrap();
    ///
    /// // Assert
    /// let result = unwrap_value(Some(unwrap_stream(&mut combined, 500).await));
    /// let values = result.values();
    /// assert_eq!(values[0] + values[1], 11);
    /// # }
    /// ```
    ///
    /// # Thread Safety
    ///
    /// Uses internal locks to maintain shared state. Lock errors are logged and
    /// affected emissions are skipped.
    fn with_latest_from<IS, R>(
        self,
        other: IS,
        result_selector: impl Fn(&CombinedState<T::Inner, T::Timestamp>) -> R + Send + Sync + 'static,
    ) -> impl Stream<Item = StreamItem<R>> + Send + Sync
    where
        IS: IntoStream<Item = StreamItem<T>>,
        IS::Stream: Send + Sync + 'static,
        R: Fluxion,
        R::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
        R::Timestamp: From<T::Timestamp> + Debug + Ord + Send + Sync + Copy + 'static;
}

impl<T, S> WithLatestFromExt<T> for S
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
    S: Stream<Item = StreamItem<T>> + Sized + Unpin + Send + Sync + 'static,
{
    fn with_latest_from<IS, R>(
        self,
        other: IS,
        result_selector: impl Fn(&CombinedState<T::Inner, T::Timestamp>) -> R + Send + Sync + 'static,
    ) -> impl Stream<Item = StreamItem<R>> + Send + Sync
    where
        IS: IntoStream<Item = StreamItem<T>>,
        IS::Stream: Send + Sync + 'static,
        R: Fluxion,
        R::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
        R::Timestamp: From<T::Timestamp> + Debug + Ord + Send + Sync + Copy + 'static,
    {
        let streams: Vec<PinnedStream<T>> = vec![Box::pin(self), Box::pin(other.into_stream())];

        let num_streams = streams.len();
        let state = Arc::new(Mutex::new(IntermediateState::new(num_streams)));
        let selector = Arc::new(result_selector);

        let combined_stream = ordered_merge_with_index(streams).filter_map({
            let state = Arc::clone(&state);
            let selector = Arc::clone(&selector);

            move |(item, stream_index)| {
                let state = Arc::clone(&state);
                let selector = Arc::clone(&selector);

                async move {
                    match item {
                        StreamItem::Value(value) => {
                            let timestamp = value.timestamp();
                            // Update state with new value
                            let mut guard = state.lock();
                            guard.insert(stream_index, value);

                            // Only emit if:
                            // 1. Both streams have emitted at least once (is_complete)
                            // 2. The PRIMARY stream (index 0) triggered this emission
                            if guard.is_complete() && stream_index == 0 {
                                let values = guard.get_values();

                                // values[0] = primary, values[1] = secondary
                                let combined_state = CombinedState::new(
                                    vec![
                                        (values[0].clone().into_inner(), values[0].timestamp()),
                                        (values[1].clone().into_inner(), values[1].timestamp()),
                                    ],
                                    timestamp,
                                );

                                // Apply the result selector to transform the combined state
                                let result = selector(&combined_state);

                                // Return result directly (R implements Timestamped)
                                Some(StreamItem::Value(result))
                            } else {
                                // Secondary stream emitted, just update state but don't emit
                                None
                            }
                        }
                        StreamItem::Error(e) => Some(StreamItem::Error(e)),
                    }
                }
            }
        });

        Box::pin(combined_stream)
    }
}

type PinnedStream<T> = Pin<Box<dyn Stream<Item = StreamItem<T>> + Send + Sync>>;

#[derive(Clone)]
struct IntermediateState<T> {
    values: Vec<Option<T>>,
}

impl<T: Clone> IntermediateState<T> {
    fn new(size: usize) -> Self {
        Self {
            values: vec![None; size],
        }
    }

    fn insert(&mut self, index: usize, value: T) {
        if index < self.values.len() {
            self.values[index] = Some(value);
        }
    }

    fn is_complete(&self) -> bool {
        self.values.iter().all(|v| v.is_some())
    }

    fn get_values(&self) -> Vec<T> {
        self.values
            .iter()
            .filter_map(|v| v.as_ref())
            .cloned()
            .collect()
    }
}
