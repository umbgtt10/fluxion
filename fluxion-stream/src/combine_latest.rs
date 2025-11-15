// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use futures::future::ready;
use futures::{Stream, StreamExt};
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use fluxion_core::into_stream::IntoStream;
use fluxion_core::{CompareByInner, Ordered, OrderedWrapper};
use fluxion_ordered_merge::OrderedMergeExt;

use fluxion_core::lock_utilities::safe_lock;

/// Extension trait providing the `combine_latest` operator for ordered streams.
///
/// This trait enables combining multiple streams where each emission waits for
/// at least one value from all streams, then emits the combination of the latest
/// values from each stream.
pub trait CombineLatestExt<T>: Stream<Item = T> + Sized
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
    /// # Examples
    ///
    /// ```rust,ignore
    /// use fluxion_stream::{CombineLatestExt, FluxionStream};
    /// use fluxion_core::Ordered;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// // Combine multiple streams of ordered data
    /// let stream1 = FluxionStream::from_unbounded_receiver(rx1);
    /// let stream2 = FluxionStream::from_unbounded_receiver(rx2);
    ///
    /// let combined = stream1.combine_latest(
    ///     vec![stream2],
    ///     |state| {
    ///         // Filter: only emit when both values are present
    ///         state.get_state().len() == 2
    ///     }
    /// );
    ///
    /// // Process combined values
    /// combined.for_each(|combined_value| async move {
    ///     let values = combined_value.get().get_state();
    ///     println!("Combined: {:?}", values);
    /// }).await;
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
    ) -> impl Stream<Item = OrderedWrapper<CombinedState<T::Inner>>> + Send + Unpin
    where
        IS: IntoStream<Item = T>,
        IS::Stream: Send + Sync + 'static;
}

type PinnedStreams<T> = Vec<Pin<Box<dyn Stream<Item = (T, usize)> + Send + Sync>>>;

impl<T, S> CombineLatestExt<T> for S
where
    T: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + CompareByInner + 'static,
    T::Inner: Clone + Debug + Ord + Send + Sync + 'static,
    S: Stream<Item = T> + Send + Sync + 'static,
{
    fn combine_latest<IS>(
        self,
        others: Vec<IS>,
        filter: impl Fn(&CombinedState<T::Inner>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = OrderedWrapper<CombinedState<T::Inner>>> + Send + Unpin
    where
        IS: IntoStream<Item = T>,
        IS::Stream: Send + Sync + 'static,
    {
        let mut streams: PinnedStreams<T> = vec![];

        streams.push(Box::pin(self.map(move |value| (value, 0))));
        for (index, into_stream) in others.into_iter().enumerate() {
            let idx = index + 1;
            let stream = into_stream.into_stream();
            streams.push(Box::pin(stream.map(move |value| (value, idx))));
        }

        let num_streams = streams.len();
        let state = Arc::new(Mutex::new(IntermediateState::new(num_streams)));

        Box::pin(
            streams
                .ordered_merge()
                .filter_map({
                    let state = Arc::clone(&state);

                    move |(value, index)| {
                        let state = Arc::clone(&state);
                        let order = value.order(); // Capture order of triggering value
                        async move {
                            // Use safe_lock to handle potential lock errors
                            match safe_lock(&state, "combine_latest state") {
                                Ok(mut guard) => {
                                    guard.insert(index, value);

                                    if guard.is_complete() {
                                        Some((guard.clone(), order))
                                    } else {
                                        None
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to acquire lock in combine_latest: {}", e);
                                    None
                                }
                            }
                        }
                    }
                })
                .map(|(state, order)| {
                    // Extract the inner values to create CombinedState
                    let inner_values: Vec<T::Inner> = state
                        .get_ordered_values()
                        .iter()
                        .map(|ordered_val| ordered_val.get().clone())
                        .collect();
                    let combined = CombinedState::new(inner_values);
                    OrderedWrapper::with_order(combined, order)
                })
                .filter(move |ordered_combined| {
                    let combined_state = ordered_combined.get();
                    ready(filter(combined_state))
                }),
        )
    }
}

/// Represents the combined state of multiple streams in `combine_latest`.
///
/// This type holds a vector of the latest values from each stream being combined.
/// The order of values matches the order streams were provided to `combine_latest`:
/// index 0 is the primary stream, indices 1+ are the `others` streams in order.
///
/// # Type Parameters
///
/// * `V` - The inner value type from the ordered streams
///
/// # Examples
///
/// ```rust
/// use fluxion_stream::CombinedState;
///
/// let state = CombinedState::new(vec![1, 2, 3]);
/// assert_eq!(state.get_state().len(), 3);
/// assert_eq!(state.get_state()[0], 1);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CombinedState<V>
where
    V: Clone + Send + Sync,
{
    state: Vec<V>,
}

impl<V> PartialOrd for CombinedState<V>
where
    V: Clone + Send + Sync + Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<V> Ord for CombinedState<V>
where
    V: Clone + Send + Sync + Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.state.cmp(&other.state)
    }
}

impl<V> CombinedState<V>
where
    V: Clone + Send + Sync,
{
    #[must_use]
    pub const fn new(state: Vec<V>) -> Self {
        Self { state }
    }

    #[must_use]
    pub const fn get_state(&self) -> &Vec<V> {
        &self.state
    }
}

impl<V> Ordered for CombinedState<V>
where
    V: Clone + Send + Sync,
{
    type Inner = Self;

    fn order(&self) -> u64 {
        // CombinedState doesn't have its own order - it's always wrapped by an Ordered type
        // This should never be called directly
        0
    }

    fn get(&self) -> &Self::Inner {
        self
    }

    fn with_order(value: Self::Inner, _order: u64) -> Self {
        value
    }

    fn into_inner(self) -> Self::Inner {
        self
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
