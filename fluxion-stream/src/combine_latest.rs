// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::ordered_merge::ordered_merge_with_index;
use crate::types::CombinedState;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt::Debug;
use core::pin::Pin;
use fluxion_core::into_stream::IntoStream;
use fluxion_core::{Fluxion, StreamItem, Timestamped};
use futures::future::ready;
use futures::{Stream, StreamExt};
use parking_lot::Mutex;

/// Extension trait providing the `combine_latest` operator for timestamped streams.
///
/// This trait enables combining multiple streams where each emission waits for
/// at least one value from all streams, then emits the combination of the latest
/// values from each stream.
pub trait CombineLatestExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Clone + Debug + Ord + Send + Sync,
{
    /// Combines this stream with multiple other streams, emitting when any stream emits.
    ///
    /// This operator maintains the latest value from each stream and emits a combined state
    /// whenever any stream produces a new value (after all streams have emitted at least once).
    /// The emitted items preserve temporal ordering based on the triggering stream's order.
    ///
    /// # Behavior
    ///
    /// - Waits until all streams have emitted at least one value
    /// - After initialization, emits whenever any stream produces a value
    /// - Maintains temporal ordering using the `Ordered` trait
    /// - Allows filtering emissions based on the combined state
    ///
    /// # Arguments
    ///
    /// * `others` - Vector of streams to combine with this stream. Each stream must implement
    ///   `IntoStream` with items compatible with this stream's item type.
    /// * `filter` - Predicate function that determines whether to emit a combined state.
    ///   Receives `&CombinedState<T::Inner>` and returns `true` to emit.
    ///
    /// # Returns
    ///
    /// A stream of timestamped `CombinedState<T::Inner>` where each emission contains
    /// the latest values from all streams, preserving the temporal order of the triggering value.
    ///
    /// # Errors
    ///
    /// This operator emits `StreamItem::Error` in the following cases:
    ///
    /// - **Lock acquisition failure**: If the internal mutex becomes poisoned (a thread panicked
    ///   while holding the lock), a `FluxionError::LockError` is emitted. The stream continues
    ///   processing subsequent items.
    ///
    /// These errors flow through the stream as `StreamItem::Error` values and can be handled
    /// using standard stream methods like `filter_map` or pattern matching.
    ///
    /// See the [Error Handling Guide](../../docs/ERROR-HANDLING.md)
    /// for patterns and best practices.
    ///
    /// # See Also
    ///
    /// - [`with_latest_from`](crate::WithLatestFromExt::with_latest_from) - Similar but only emits when primary stream emits
    /// - [`ordered_merge`](crate::OrderedStreamExt::ordered_merge) - Merges streams emitting all items
    /// - [`emit_when`](crate::EmitWhenExt::emit_when) - Gates emissions based on filter conditions
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_stream::CombineLatestExt;
    /// use fluxion_test_utils::{Sequenced, helpers::unwrap_stream, unwrap_value, test_channel};
    /// use fluxion_core::Timestamped as TimestampedTrait;
    ///
    /// # async fn example() {
    /// // Create channels
    /// let (tx1, stream1) = test_channel::<Sequenced<i32>>();
    /// let (tx2, stream2) = test_channel::<Sequenced<i32>>();
    ///
    /// // Combine streams
    /// let mut combined = stream1.combine_latest(
    ///     vec![stream2],
    ///     |state| state.values().len() == 2
    /// );
    ///
    /// // Send values
    /// tx1.unbounded_send((1, 1).into()).unwrap();
    /// tx2.unbounded_send((2, 2).into()).unwrap();
    ///
    /// // Assert
    /// let result = unwrap_value(Some(unwrap_stream(&mut combined, 500).await));
    /// let values = result.values();
    /// assert_eq!(values.len(), 2);
    /// assert_eq!(values[0], 1);
    /// assert_eq!(values[1], 2);
    /// # }
    /// ```
    ///
    /// # Thread Safety
    ///
    /// This operator uses internal locks to maintain shared state. Lock errors are logged
    /// and affected emissions are skipped rather than causing panics.
    fn combine_latest<IS>(
        self,
        others: Vec<IS>,
        filter: impl Fn(&CombinedState<T::Inner, T::Timestamp>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = StreamItem<CombinedState<T::Inner, T::Timestamp>>> + Send + Unpin
    where
        IS: IntoStream<Item = StreamItem<T>>,
        IS::Stream: Send + Sync + 'static,
        CombinedState<T::Inner, T::Timestamp>:
            Timestamped<Inner = CombinedState<T::Inner, T::Timestamp>, Timestamp = T::Timestamp>;
}

type PinnedStreams<T> = Vec<Pin<Box<dyn Stream<Item = StreamItem<T>> + Send + Sync>>>;

impl<T, S> CombineLatestExt<T> for S
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Clone + Debug + Ord + Send + Sync,
    S: Stream<Item = StreamItem<T>> + Send + Sync + 'static,
    CombinedState<T::Inner, T::Timestamp>:
        Timestamped<Inner = CombinedState<T::Inner, T::Timestamp>, Timestamp = T::Timestamp>,
{
    fn combine_latest<IS>(
        self,
        others: Vec<IS>,
        filter: impl Fn(&CombinedState<T::Inner, T::Timestamp>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = StreamItem<CombinedState<T::Inner, T::Timestamp>>> + Send + Unpin
    where
        IS: IntoStream<Item = StreamItem<T>>,
        IS::Stream: Send + Sync + 'static,
        CombinedState<T::Inner, T::Timestamp>:
            Timestamped<Inner = CombinedState<T::Inner, T::Timestamp>, Timestamp = T::Timestamp>,
    {
        use StreamItem;
        let mut streams: PinnedStreams<T> = vec![];

        streams.push(Box::pin(self));
        for into_stream in others {
            let stream = into_stream.into_stream();
            streams.push(Box::pin(stream));
        }

        let num_streams = streams.len();
        let state = Arc::new(Mutex::new(IntermediateState::new(num_streams)));

        let combined_stream = ordered_merge_with_index(streams)
            .filter_map({
                let state = Arc::clone(&state);

                move |(item, index)| {
                    let state = Arc::clone(&state);
                    async move {
                        match item {
                            StreamItem::Value(value) => {
                                let mut guard = state.lock();
                                guard.insert(index, value);

                                if guard.is_complete() {
                                    Some(StreamItem::Value(guard.clone()))
                                } else {
                                    None
                                }
                            }
                            StreamItem::Error(e) => {
                                // Propagate upstream errors immediately without state update
                                Some(StreamItem::Error(e))
                            }
                        }
                    }
                }
            })
            .map(|item| {
                item.map(|state| {
                    // Extract the inner values with their timestamps to create CombinedState
                    let value_timestamp_pairs: Vec<(T::Inner, T::Timestamp)> = state
                        .get_ordered_values()
                        .iter()
                        .map(|ordered_val| {
                            (ordered_val.clone().into_inner(), ordered_val.timestamp())
                        })
                        .collect();
                    let timestamp = state.last_timestamp().expect("State must have timestamp");
                    CombinedState::new(value_timestamp_pairs, timestamp)
                })
            })
            .filter(move |item| {
                match item {
                    StreamItem::Value(combined_state) => ready(filter(combined_state)),
                    StreamItem::Error(_) => ready(true), // Always emit errors
                }
            });

        Box::pin(combined_stream)
    }
}

#[derive(Clone, Debug)]
struct IntermediateState<V>
where
    V: Clone + Send + Sync + Ord + Timestamped,
{
    state: Vec<Option<V>>,
    ordered_values: Vec<V>,
    stream_index_to_position: Vec<usize>,
    is_initialized: bool,
    last_timestamp: Option<V::Timestamp>,
}

impl<V> IntermediateState<V>
where
    V: Clone + Send + Sync + Ord + Timestamped,
{
    pub fn new(num_streams: usize) -> Self {
        Self {
            state: vec![None; num_streams],
            ordered_values: Vec::new(),
            stream_index_to_position: vec![0; num_streams],
            is_initialized: false,
            last_timestamp: None,
        }
    }

    pub const fn get_ordered_values(&self) -> &Vec<V> {
        &self.ordered_values
    }

    pub fn last_timestamp(&self) -> Option<V::Timestamp> {
        self.last_timestamp
    }

    pub fn is_complete(&self) -> bool {
        self.state.iter().all(Option::is_some)
    }

    pub fn insert(&mut self, index: usize, value: V) {
        self.last_timestamp = Some(value.timestamp());
        self.state[index] = Some(value.clone());

        if !self.is_initialized && self.is_complete() {
            // First complete state: establish the ordering
            // Collect all values with their stream indices
            let mut indexed_values: Vec<(usize, V)> = self
                .state
                .iter()
                .enumerate()
                .filter_map(|(i, opt)| opt.as_ref().map(|v| (i, v.clone())))
                .collect();

            // Sort by stream index to establish stable ordering
            indexed_values.sort_by_key(|(stream_idx, _)| *stream_idx);

            // Build the ordered_values and the mapping
            self.ordered_values = indexed_values.iter().map(|(_, v)| v.clone()).collect();

            // Build stream_index_to_position: for each stream index, record its position
            for (position, (stream_idx, _)) in indexed_values.iter().enumerate() {
                self.stream_index_to_position[*stream_idx] = position;
            }

            self.is_initialized = true;
        } else if self.is_initialized {
            // After initialization: update the value at its established position
            let position = self.stream_index_to_position[index];
            if position < self.ordered_values.len() {
                self.ordered_values[position] = value;
            }
        }
    }
}
