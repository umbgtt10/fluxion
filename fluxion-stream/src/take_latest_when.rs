// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::FluxionStream;
use fluxion_core::into_stream::IntoStream;
use fluxion_core::lock_utilities::lock_or_recover;
use fluxion_core::{Fluxion, StreamItem};
use fluxion_ordered_merge::OrderedMergeExt;
use futures::{Stream, StreamExt};
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

/// Extension trait providing the `take_latest_when` operator for timestamped streams.
///
/// This operator samples the latest value from a source stream whenever a filter
/// stream emits a value that passes a predicate.
pub trait TakeLatestWhenExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
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
    /// tx_data.send((100, 1).into()).unwrap();
    /// tx_trigger.send((1, 2).into()).unwrap();  // Trigger
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
    /// See the [Error Handling Guide](../../docs/ERROR-HANDLING.md)
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
    ) -> FluxionStream<impl Stream<Item = StreamItem<T>> + Send + Sync>
    where
        IS: IntoStream<Item = StreamItem<T>>,
        IS::Stream: Send + Sync + 'static;
}

type IndexedStream<T> = Pin<Box<dyn Stream<Item = (StreamItem<T>, usize)> + Send + Sync>>;
impl<T, S> TakeLatestWhenExt<T> for S
where
    S: Stream<Item = StreamItem<T>> + Send + Sync + Unpin + 'static,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    fn take_latest_when<IS>(
        self,
        filter_stream: IS,
        filter: impl Fn(&T::Inner) -> bool + Send + Sync + 'static,
    ) -> FluxionStream<impl Stream<Item = StreamItem<T>> + Send + Sync>
    where
        IS: IntoStream<Item = StreamItem<T>>,
        IS::Stream: Send + Sync + 'static,
    {
        let source_stream = Box::pin(self.map(|item| (item, 0)));
        let filter_stream = Box::pin(filter_stream.into_stream().map(|item| (item, 1)));

        let streams: Vec<IndexedStream<T>> = vec![source_stream, filter_stream];

        let source_value = Arc::new(Mutex::new(None));
        let filter = Arc::new(filter);

        let combined_stream = streams.ordered_merge().filter_map(move |(item, index)| {
            let source_value = Arc::clone(&source_value);
            let filter = Arc::clone(&filter);
            async move {
                match item {
                    StreamItem::Value(ordered_value) => {
                        match index {
                            0 => {
                                // Source stream update - just cache the value, don't emit
                                let mut source =
                                    lock_or_recover(&source_value, "take_latest_when source");
                                *source = Some(ordered_value);
                                None
                            }
                            1 => {
                                // Filter stream update - check if we should sample the source
                                let source =
                                    lock_or_recover(&source_value, "take_latest_when source");

                                // Update filter value
                                let filter_inner = ordered_value.clone().into_inner();

                                // Now check the condition and potentially emit
                                if filter(&filter_inner) {
                                    source.as_ref().map(|src| {
                                        StreamItem::Value(T::with_timestamp(
                                            src.clone().into_inner(),
                                            ordered_value.timestamp(),
                                        ))
                                    })
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
        });

        FluxionStream::new(Box::pin(combined_stream))
    }
}
