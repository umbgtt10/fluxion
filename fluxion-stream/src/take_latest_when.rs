// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::Ordered;
use fluxion_core::into_stream::IntoStream;
use fluxion_core::lock_utilities::lock_or_error;
use fluxion_core::StreamItem;
use fluxion_ordered_merge::OrderedMergeExt;
use futures::{Stream, StreamExt};
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

/// Extension trait providing the `take_latest_when` operator for ordered streams.
///
/// This operator samples the latest value from a source stream whenever a filter
/// stream emits a value that passes a predicate.
pub trait TakeLatestWhenExt<T>: Stream<Item = fluxion_core::StreamItem<T>> + Sized
where
    T: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
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
    /// use fluxion_stream::{TakeLatestWhenExt, FluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// // Create channels
    /// let (tx_data, rx_data) = tokio::sync::mpsc::unbounded_channel();
    /// let (tx_trigger, rx_trigger) = tokio::sync::mpsc::unbounded_channel();
    ///
    /// // Create streams
    /// let data_stream = FluxionStream::from_unbounded_receiver(rx_data);
    /// let trigger_stream = FluxionStream::from_unbounded_receiver(rx_trigger);
    ///
    /// // Combine streams
    /// let mut sampled = data_stream.take_latest_when(
    ///     trigger_stream,
    ///     |trigger_value| *trigger_value > 0  // Trigger when value > 0
    /// );
    ///
    /// // Send values
    /// tx_data.send(Sequenced::with_sequence(100, 1)).unwrap();
    /// tx_trigger.send(Sequenced::with_sequence(1, 2)).unwrap();  // Trigger
    ///
    /// // Assert - trigger emits the latest data value
    /// let result = sampled.next().await.unwrap().unwrap();
    /// assert_eq!(*result.get(), 100);
    /// # }
    /// ```
    ///
    /// # Use Cases
    ///
    /// - Sampling sensor data on timer ticks
    /// - Gate pattern: emit values only when enabled
    /// - Throttling with external control signals
    ///
    /// # Panics
    ///
    /// Uses internal locks to maintain the latest source value. If a thread panics while
    /// holding a lock, subsequent operations will log a warning and recover the poisoned
    /// lock. The affected emission is skipped if lock acquisition fails.
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
    /// See the [Error Handling Guide](https://github.com/umbgtt10/fluxion/blob/main/docs/ERROR-HANDLING.md)
    /// for handling strategies.
    ///
    /// # See Also
    ///
    /// - [`emit_when`](crate::EmitWhenExt::emit_when) - Gates source emissions rather than sampling
    /// - [`with_latest_from`](crate::WithLatestFromExt::with_latest_from) - Emits on primary, samples secondary
    /// - [`take_while_with`](crate::TakeWhileExt::take_while_with) - Terminates when condition becomes false
    fn take_latest_when<IS>(
        self,
        filter_stream: IS,
        filter: impl Fn(&T::Inner) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = StreamItem<T>> + Send + Sync
    where
        IS: IntoStream<Item = fluxion_core::StreamItem<T>>,
        IS::Stream: Send + Sync + 'static;
}

type IndexedStream<T> =
    Pin<Box<dyn Stream<Item = (fluxion_core::StreamItem<T>, usize)> + Send + Sync>>;
impl<T, S> TakeLatestWhenExt<T> for S
where
    S: Stream<Item = fluxion_core::StreamItem<T>> + Send + Sync + 'static,
    T: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn take_latest_when<IS>(
        self,
        filter_stream: IS,
        filter: impl Fn(&T::Inner) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = StreamItem<T>> + Send + Sync
    where
        IS: IntoStream<Item = fluxion_core::StreamItem<T>>,
        IS::Stream: Send + Sync + 'static,
    {
        let source_stream = Box::pin(self.map(|item| (item, 0)));
        let filter_stream = Box::pin(filter_stream.into_stream().map(|item| (item, 1)));

        let streams: Vec<IndexedStream<T>> = vec![source_stream, filter_stream];

        let source_value = Arc::new(Mutex::new(None));
        let filter_value = Arc::new(Mutex::new(None));
        let filter = Arc::new(filter);

        Box::pin(streams.ordered_merge().filter_map(move |(item, index)| {
            let source_value = Arc::clone(&source_value);
            let filter_value = Arc::clone(&filter_value);
            let filter = Arc::clone(&filter);
            async move {
                match item {
                    StreamItem::Value(ordered_value) => {
                        let order = ordered_value.order();
                        match index {
                            0 => {
                                let mut source =
                                    match lock_or_error(&source_value, "take_latest_when source") {
                                        Ok(lock) => lock,
                                        Err(e) => {
                                            return Some(StreamItem::Error(e));
                                        }
                                    };
                                *source = Some(ordered_value.get().clone());
                                None
                            }
                            1 => {
                                let mut filter_val =
                                    match lock_or_error(&filter_value, "take_latest_when filter") {
                                        Ok(lock) => lock,
                                        Err(e) => {
                                            return Some(StreamItem::Error(e));
                                        }
                                    };
                                *filter_val = Some(ordered_value.get().clone());

                                let source =
                                    match lock_or_error(&source_value, "take_latest_when source") {
                                        Ok(lock) => lock,
                                        Err(e) => {
                                            return Some(StreamItem::Error(e));
                                        }
                                    };

                                if let Some(src) = source.as_ref() {
                                    if let Some(filt) = filter_val.as_ref() {
                                        if filter(filt) {
                                            Some(StreamItem::Value(T::with_order(
                                                src.clone(),
                                                order,
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
                                warn!(
                                    "take_latest_when: unexpected stream index {} – ignoring",
                                    index
                                );
                                None
                            }
                        }
                    }
                    StreamItem::Error(e) => Some(StreamItem::Error(e)),
                }
            }
        }))
    }
}
