// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::types::CombinedState;
use crate::FluxionStream;
use fluxion_core::into_stream::IntoStream;
use fluxion_core::lock_utilities::lock_or_error;
use fluxion_core::{StreamItem, Timestamped};
use fluxion_ordered_merge::OrderedMergeExt;
use futures::{Stream, StreamExt};
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

/// Extension trait providing the `emit_when` operator for ordered streams.
///
/// This operator gates a source stream based on conditions from a filter stream,
/// emitting source values only when the combined state passes a predicate.
pub trait EmitWhenExt<T>: Stream<Item = fluxion_core::StreamItem<T>> + Sized
where
    T: Timestamped + Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    /// Emits source stream values only when the filter condition is satisfied.
    ///
    /// This operator maintains the latest values from both the source and filter streams,
    /// creating a combined state. Source values are emitted only when this combined state
    /// passes the provided filter predicate.
    ///
    /// # Behavior
    ///
    /// - Maintains latest value from both source and filter streams
    /// - Evaluates predicate on `CombinedState` containing both values
    /// - Emits source value when predicate returns `true`
    /// - Both streams must emit at least once before any emission occurs
    /// - Preserves temporal ordering of source stream
    ///
    /// # Arguments
    ///
    /// * `filter_stream` - Stream providing filter values for the gate condition
    /// * `filter` - Predicate function that receives `CombinedState<T::Inner>` containing
    ///   `[source_value, filter_value]` and returns `true` to emit.
    ///
    /// # Returns
    ///
    /// A pinned stream of `T` containing source values that pass the filter condition.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_stream::{EmitWhenExt, FluxionStream};
    /// use fluxion_test_utils::Timestamped;
    /// use fluxion_core::Timestamped ;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// // Create channels
    /// let (tx_data, rx_data) = tokio::sync::mpsc::unbounded_channel::<Timestamped<i32>>();
    /// let (tx_enable, rx_enable) = tokio::sync::mpsc::unbounded_channel::<Timestamped<i32>>();
    ///
    /// // Create streams
    /// let data_stream = FluxionStream::from_unbounded_receiver(rx_data);
    /// let enable_stream = FluxionStream::from_unbounded_receiver(rx_enable);
    ///
    /// // Combine streams
    /// let mut gated = data_stream.emit_when(
    ///     enable_stream,
    ///     |state| {
    ///         let values = state.values();
    ///         values[1] > 0  // Enable when value > 0
    ///     }
    /// );
    ///
    /// // Send values
    /// tx_enable.send((1, 1).into()).unwrap();  // Enabled
    /// tx_data.send((42, 2).into()).unwrap();
    ///
    /// // Assert - data emits when enabled
    /// let result = gated.next().await.unwrap().unwrap();
    /// assert_eq!(*result.inner(), 42);
    /// # }
    /// ```
    ///
    /// # Use Cases
    ///
    /// - Conditional forwarding based on external signals
    /// - State-dependent filtering
    /// - Complex gating logic involving multiple stream values
    ///
    /// # Panics
    ///
    /// Uses internal locks to maintain shared state. If a thread panics while holding a lock,
    /// subsequent operations will log a warning and recover the poisoned lock. Affected
    /// emissions are skipped if lock acquisition fails.
    ///
    /// # Errors
    ///
    /// Emits `StreamItem::Error` when lock acquisition fails:
    ///
    /// - **Combined state lock error**: If the internal state mutex becomes poisoned,
    ///   a `FluxionError::LockError` is emitted for that item
    ///
    /// Lock errors are transient - the stream continues processing subsequent items.
    ///
    /// See the [Error Handling Guide](../../docs/ERROR-HANDLING.md).
    ///
    /// # See Also
    ///
    /// - [`take_latest_when`](crate::TakeLatestWhenExt::take_latest_when) - Similar but samples latest instead of gating
    /// - [`with_latest_from`](crate::WithLatestFromExt::with_latest_from) - Combines with secondary stream on primary emission
    /// - [`take_while_with`](crate::TakeWhileExt::take_while_with) - Emits while condition holds, then terminates
    fn emit_when<IS>(
        self,
        filter_stream: IS,
        filter: impl Fn(&CombinedState<T::Inner, T::Timestamp>) -> bool + Send + Sync + 'static,
    ) -> FluxionStream<impl Stream<Item = StreamItem<T>> + Send + Sync>
    where
        IS: IntoStream<Item = fluxion_core::StreamItem<T>>,
        IS::Stream: Send + Sync + 'static;
}

type IndexedStream<T> =
    Pin<Box<dyn Stream<Item = (fluxion_core::StreamItem<T>, usize)> + Send + Sync>>;
impl<T, S> EmitWhenExt<T> for S
where
    S: Stream<Item = fluxion_core::StreamItem<T>> + Send + Sync + 'static,
    T: Timestamped + Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn emit_when<IS>(
        self,
        filter_stream: IS,
        filter: impl Fn(&CombinedState<T::Inner, T::Timestamp>) -> bool + Send + Sync + 'static,
    ) -> FluxionStream<impl Stream<Item = StreamItem<T>> + Send + Sync>
    where
        IS: IntoStream<Item = fluxion_core::StreamItem<T>>,
        IS::Stream: Send + Sync + 'static,
    {
        let source_stream = Box::pin(self.map(|item| (item, 0)));
        let filter_stream = Box::pin(filter_stream.into_stream().map(|item| (item, 1)));

        let streams: Vec<IndexedStream<T>> = vec![source_stream, filter_stream];

        let source_value: Arc<Mutex<Option<T::Inner>>> = Arc::new(Mutex::new(None));
        let filter_value: Arc<Mutex<Option<T::Inner>>> = Arc::new(Mutex::new(None));
        let filter = Arc::new(filter);

        let combined_stream = streams.ordered_merge().filter_map(move |(item, index)| {
            let source_value = Arc::clone(&source_value);
            let filter_value = Arc::clone(&filter_value);
            let filter = Arc::clone(&filter);
            async move {
                match item {
                    StreamItem::Value(ordered_value) => {
                        match index {
                            0 => {
                                // Source stream update
                                // Lock both values once to avoid multiple lock acquisitions
                                let mut source =
                                    match lock_or_error(&source_value, "emit_when source") {
                                        Ok(lock) => lock,
                                        Err(e) => {
                                            return Some(StreamItem::Error(e));
                                        }
                                    };
                                let filter_val =
                                    match lock_or_error(&filter_value, "emit_when filter") {
                                        Ok(lock) => lock,
                                        Err(e) => {
                                            return Some(StreamItem::Error(e));
                                        }
                                    };

                                // Update source value
                                *source = Some(ordered_value.inner().clone());
                                let timestamp = ordered_value.timestamp();

                                if let Some(src) = source.as_ref() {
                                    if let Some(filt) = filter_val.as_ref() {
                                        let combined_state = CombinedState::new(
                                            vec![src.clone(), filt.clone()],
                                            timestamp,
                                        );
                                        if filter(&combined_state) {
                                            Some(StreamItem::Value(T::with_fresh_timestamp(
                                                src.clone(),
                                            )))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            }
                            1 => {
                                // Filter stream update
                                // Lock both values once to avoid multiple lock acquisitions
                                let mut filter_val =
                                    match lock_or_error(&filter_value, "emit_when filter") {
                                        Ok(lock) => lock,
                                        Err(e) => {
                                            return Some(StreamItem::Error(e));
                                        }
                                    };
                                let source = match lock_or_error(&source_value, "emit_when source")
                                {
                                    Ok(lock) => lock,
                                    Err(e) => {
                                        return Some(StreamItem::Error(e));
                                    }
                                };

                                // Update filter value
                                *filter_val = Some(ordered_value.inner().clone());
                                let timestamp = ordered_value.timestamp();

                                if let Some(src) = source.as_ref() {
                                    if let Some(filt) = filter_val.as_ref() {
                                        let combined_state = CombinedState::new(
                                            vec![src.clone(), filt.clone()],
                                            timestamp,
                                        );
                                        if filter(&combined_state) {
                                            Some(StreamItem::Value(T::with_fresh_timestamp(
                                                src.clone(),
                                            )))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            }
                            _ => {
                                warn!("emit_when: unexpected stream index {} – ignoring", index);
                                None
                            }
                        }
                    }
                    StreamItem::Error(e) => Some(StreamItem::Error(e)),
                }
            }
        });

        let result = Box::pin(combined_stream);
        FluxionStream::new(result)
    }
}
