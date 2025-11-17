// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use futures::future::ready;
use futures::{Stream, StreamExt};
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use crate::types::CombinedState;
use fluxion_core::into_stream::IntoStream;
use fluxion_core::lock_utilities::lock_or_error;
use fluxion_core::{CompareByInner, Ordered, OrderedWrapper};
use fluxion_ordered_merge::OrderedMergeExt;

/// Extension trait providing the `combine_latest` operator for ordered streams.
///
/// This trait enables combining multiple streams where each emission waits for
/// at least one value from all streams, then emits the combination of the latest
/// values from each stream.
pub trait CombineLatestExt<T>: Stream<Item = fluxion_core::StreamItem<T>> + Sized
where
    T: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + CompareByInner + 'static,
    T::Inner: Clone + Debug + Ord + Send + Sync + 'static,
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
    /// A stream of `OrderedWrapper<CombinedState<T::Inner>>` where each emission contains
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
    /// See the [Error Handling Guide](https://github.com/umbgtt10/fluxion/blob/main/docs/ERROR-HANDLING.md)
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
    /// use fluxion_stream::{CombineLatestExt, FluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use fluxion_core::Ordered;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// // Create channels
    /// let (tx1, rx1) = tokio::sync::mpsc::unbounded_channel();
    /// let (tx2, rx2) = tokio::sync::mpsc::unbounded_channel();
    ///
    /// // Create streams
    /// let stream1 = FluxionStream::from_unbounded_receiver(rx1);
    /// let stream2 = FluxionStream::from_unbounded_receiver(rx2);
    ///
    /// // Combine streams
    /// let mut combined = stream1.combine_latest(
    ///     vec![stream2],
    ///     |state| state.values().len() == 2
    /// );
    ///
    /// // Send values
    /// tx1.send(Sequenced::with_sequence(1, 1)).unwrap();
    /// tx2.send(Sequenced::with_sequence(2, 2)).unwrap();
    ///
    /// // Assert
    /// let result = combined.next().await.unwrap();
    /// let values = result.get().values();
    /// assert_eq!(values.len(), 2);
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
        filter: impl Fn(&CombinedState<T::Inner>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = fluxion_core::StreamItem<OrderedWrapper<CombinedState<T::Inner>>>>
           + Send
           + Unpin
    where
        IS: IntoStream<Item = fluxion_core::StreamItem<T>>,
        IS::Stream: Send + Sync + 'static;
}

type PinnedStreams<T> =
    Vec<Pin<Box<dyn Stream<Item = (fluxion_core::StreamItem<T>, usize)> + Send + Sync>>>;

impl<T, S> CombineLatestExt<T> for S
where
    T: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + CompareByInner + 'static,
    T::Inner: Clone + Debug + Ord + Send + Sync + 'static,
    S: Stream<Item = fluxion_core::StreamItem<T>> + Send + Sync + 'static,
{
    fn combine_latest<IS>(
        self,
        others: Vec<IS>,
        filter: impl Fn(&CombinedState<T::Inner>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = fluxion_core::StreamItem<OrderedWrapper<CombinedState<T::Inner>>>>
           + Send
           + Unpin
    where
        IS: IntoStream<Item = fluxion_core::StreamItem<T>>,
        IS::Stream: Send + Sync + 'static,
    {
        use fluxion_core::StreamItem;
        let mut streams: PinnedStreams<T> = vec![];

        streams.push(Box::pin(self.map(move |item| (item, 0))));
        for (index, into_stream) in others.into_iter().enumerate() {
            let idx = index + 1;
            let stream = into_stream.into_stream();
            streams.push(Box::pin(stream.map(move |item| (item, idx))));
        }

        let num_streams = streams.len();
        let state = Arc::new(Mutex::new(IntermediateState::new(num_streams)));

        Box::pin(
            streams
                .ordered_merge()
                .filter_map({
                    let state = Arc::clone(&state);

                    move |(item, index)| {
                        let state = Arc::clone(&state);
                        async move {
                            match item {
                                StreamItem::Value(value) => {
                                    let order = value.order();
                                    match lock_or_error(&state, "combine_latest state") {
                                        Ok(mut guard) => {
                                            guard.insert(index, value);

                                            if guard.is_complete() {
                                                Some(StreamItem::Value((guard.clone(), order)))
                                            } else {
                                                None
                                            }
                                        }
                                        Err(e) => {
                                            error!(
                                                "Failed to acquire lock in combine_latest: {}",
                                                e
                                            );
                                            Some(StreamItem::Error(e))
                                        }
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
                    item.map(|(state, order)| {
                        // Extract the inner values to create CombinedState
                        let inner_values: Vec<T::Inner> = state
                            .get_ordered_values()
                            .iter()
                            .map(|ordered_val| ordered_val.get().clone())
                            .collect();
                        let combined = CombinedState::new(inner_values);
                        OrderedWrapper::with_order(combined, order)
                    })
                })
                .filter(move |item| {
                    match item {
                        StreamItem::Value(ordered_combined) => {
                            let combined_state = ordered_combined.get();
                            ready(filter(combined_state))
                        }
                        StreamItem::Error(_) => ready(true), // Always emit errors
                    }
                }),
        )
    }
}

#[derive(Clone, Debug)]
struct IntermediateState<V>
where
    V: Clone + Send + Sync + Ord + CompareByInner,
{
    state: Vec<Option<V>>,
    ordered_values: Vec<V>,
    stream_index_to_position: Vec<usize>,
    is_initialized: bool,
}

impl<V> IntermediateState<V>
where
    V: Clone + Send + Sync + Ord + CompareByInner,
{
    pub fn new(num_streams: usize) -> Self {
        Self {
            state: vec![None; num_streams],
            ordered_values: Vec::new(),
            stream_index_to_position: vec![0; num_streams],
            is_initialized: false,
        }
    }

    pub const fn get_ordered_values(&self) -> &Vec<V> {
        &self.ordered_values
    }

    pub fn is_complete(&self) -> bool {
        self.state.iter().all(Option::is_some)
    }

    pub fn insert(&mut self, index: usize, value: V) {
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

            // Sort by inner value to establish order by enum variant type
            indexed_values.sort_by(|a, b| a.1.cmp_inner(&b.1));

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
