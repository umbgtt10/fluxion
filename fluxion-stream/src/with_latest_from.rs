// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use futures::{Stream, StreamExt};
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use crate::Ordered;
use crate::ordered_merge::OrderedMergeExt;
use crate::types::CombinedState;
use fluxion_core::into_stream::IntoStream;
use fluxion_core::lock_utilities::safe_lock;
use fluxion_core::{CompareByInner, OrderedWrapper};

/// Extension trait providing the `with_latest_from` operator for ordered streams.
///
/// This operator combines a primary stream with a secondary stream, emitting only
/// when the primary stream emits, using the latest value from the secondary stream.
pub trait WithLatestFromExt<T>: Stream<Item = T> + Sized
where
    T: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + CompareByInner + 'static,
    T::Inner: Clone + Debug + Ord + Send + Sync + 'static,
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
    /// A stream of `OrderedWrapper<R>` where each emission contains the result of applying
    /// `result_selector` to the combined state, ordered by the primary stream's order.
    ///
    /// # Type Parameters
    ///
    /// * `IS` - Type that can be converted into a stream compatible with this stream
    /// * `R` - Result type produced by the `result_selector` function
    ///
    /// # Panics
    ///
    /// Uses internal locks to maintain the latest value from the secondary stream. If a thread
    /// panics while holding a lock, subsequent operations will log a warning and recover the
    /// poisoned lock. Individual emissions may be skipped if lock acquisition fails.
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
    /// use fluxion_stream::{WithLatestFromExt, FluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use fluxion_core::Ordered;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// // Create channels
    /// let (tx_primary, rx_primary) = tokio::sync::mpsc::unbounded_channel();
    /// let (tx_secondary, rx_secondary) = tokio::sync::mpsc::unbounded_channel();
    ///
    /// // Create streams
    /// let primary = FluxionStream::from_unbounded_receiver(rx_primary);
    /// let secondary = FluxionStream::from_unbounded_receiver(rx_secondary);
    ///
    /// // Combine streams
    /// let mut combined = primary.with_latest_from(
    ///     secondary,
    ///     |state| {
    ///         let values = state.get_state();
    ///         values[0] + values[1]
    ///     }
    /// );
    ///
    /// // Send values
    /// tx_secondary.send(Sequenced::with_sequence(10, 1)).unwrap();
    /// tx_primary.send(Sequenced::with_sequence(1, 2)).unwrap();
    ///
    /// // Assert
    /// let result = combined.next().await.unwrap();
    /// assert_eq!(*result.get(), 11);
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
        result_selector: impl Fn(&CombinedState<T::Inner>) -> R + Send + Sync + 'static,
    ) -> impl Stream<Item = OrderedWrapper<R>> + Send
    where
        IS: IntoStream<Item = T>,
        IS::Stream: Send + Sync + 'static,
        R: Clone + Debug + Ord + Send + Sync + 'static;
}

impl<T, P> WithLatestFromExt<T> for P
where
    T: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + CompareByInner + 'static,
    T::Inner: Clone + Debug + Ord + Send + Sync + 'static,
    P: Stream<Item = T> + Sized + Unpin + Send + Sync + 'static,
{
    fn with_latest_from<IS, R>(
        self,
        other: IS,
        result_selector: impl Fn(&CombinedState<T::Inner>) -> R + Send + Sync + 'static,
    ) -> impl Stream<Item = OrderedWrapper<R>> + Send
    where
        IS: IntoStream<Item = T>,
        IS::Stream: Send + Sync + 'static,
        R: Clone + Debug + Ord + Send + Sync + 'static,
    {
        type PinnedStream<T> = std::pin::Pin<Box<dyn Stream<Item = (T, usize)> + Send + Sync>>;
        let streams: Vec<PinnedStream<T>> = vec![
            Box::pin(self.map(move |value| (value, 0))),
            Box::pin(other.into_stream().map(move |value| (value, 1))),
        ];

        let num_streams = streams.len();
        let state = Arc::new(Mutex::new(IntermediateState::new(num_streams)));
        let selector = Arc::new(result_selector);

        Box::pin(streams.ordered_merge().filter_map({
            let state = Arc::clone(&state);
            let selector = Arc::clone(&selector);

            move |(value, stream_index)| {
                let state = Arc::clone(&state);
                let selector = Arc::clone(&selector);
                let order = value.order();

                async move {
                    // Update state with new value
                    match safe_lock(&state, "with_latest_from state") {
                        Ok(mut guard) => {
                            guard.insert(stream_index, value);

                            // Only emit if:
                            // 1. Both streams have emitted at least once (is_complete)
                            // 2. The PRIMARY stream (index 0) triggered this emission
                            if guard.is_complete() && stream_index == 0 {
                                let values = guard.get_values();

                                // values[0] = primary, values[1] = secondary
                                let combined_state = CombinedState::new(vec![
                                    values[0].get().clone(),
                                    values[1].get().clone(),
                                ]);

                                // Apply the result selector to transform the combined state
                                let result = selector(&combined_state);

                                // Wrap result with the primary's order
                                Some(OrderedWrapper::with_order(result, order))
                            } else {
                                // Secondary stream emitted, just update state but don't emit
                                None
                            }
                        }
                        Err(e) => {
                            error!("Failed to acquire lock in with_latest_from: {}", e);
                            None
                        }
                    }
                }
            }
        }))
    }
}

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
