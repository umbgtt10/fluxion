// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::FluxionStream;
use fluxion_core::lock_utilities::lock_or_recover;
use fluxion_core::{ComparableUnpinTimestamped, FluxionError, HasTimestamp, StreamItem};
use fluxion_ordered_merge::OrderedMergeExt;
use futures::stream::StreamExt;
use futures::Stream;
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

type PinnedItemStream<TItem, TFilter> =
    Pin<Box<dyn Stream<Item = Item<TItem, TFilter>> + Send + Sync + 'static>>;

/// Extension trait providing the `take_while_with` operator for timestamped streams.
///
/// This operator conditionally emits elements from a source stream based on values
/// from a separate filter stream. The stream terminates when the filter condition
/// becomes false.
pub trait TakeWhileExt<TItem, TFilter, S>: Stream<Item = StreamItem<TItem>> + Sized
where
    TItem: ComparableUnpinTimestamped,
    TItem::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    TFilter: ComparableUnpinTimestamped<Timestamp = TItem::Timestamp>,
    TFilter::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    S: Stream<Item = StreamItem<TFilter>> + Send + Sync + 'static,
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
    /// A `FluxionStream` of source elements that are emitted while the filter condition
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
    /// use fluxion_stream::{TakeWhileExt, FluxionStream};
    /// use fluxion_test_utils::Sequenced;
    /// use fluxion_core::Timestamped as TimestampedTrait;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// // Create channels
    /// let (tx_data, rx_data) = tokio::sync::mpsc::unbounded_channel::<Sequenced<i32>>();
    /// let (tx_gate, rx_gate) = tokio::sync::mpsc::unbounded_channel::<Sequenced<bool>>();
    ///
    /// // Create streams
    /// let data_stream = FluxionStream::from_unbounded_receiver(rx_data);
    /// let gate_stream = FluxionStream::from_unbounded_receiver(rx_gate);
    ///
    /// // Combine streams
    /// let mut gated = data_stream.take_while_with(
    ///     gate_stream,
    ///     |gate_value| *gate_value == true
    /// );
    ///
    /// // Send values
    /// tx_gate.send((true, 1).into()).unwrap();
    /// tx_data.send((1, 2).into()).unwrap();
    ///
    /// // Assert
    /// assert_eq!(&gated.next().await.unwrap().unwrap().value, &1);
    /// # }
    /// ```
    ///
    /// # Use Cases
    ///
    /// - Emergency stop mechanism for data streams
    /// - Time-bounded stream processing
    /// - Conditional data forwarding with external control
    ///
    /// # Thread Safety
    ///
    /// Uses internal locks to maintain state. Lock errors are logged and cause
    /// the stream to terminate.
    fn take_while_with(
        self,
        filter_stream: S,
        filter: impl Fn(&TFilter::Inner) -> bool + Send + Sync + 'static,
    ) -> FluxionStream<impl Stream<Item = StreamItem<TItem>> + Send>;
}

impl<TItem, TFilter, S, P> TakeWhileExt<TItem, TFilter, S> for P
where
    P: Stream<Item = StreamItem<TItem>> + Send + Sync + Unpin + 'static,
    TItem: ComparableUnpinTimestamped,
    TItem::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    TFilter: ComparableUnpinTimestamped<Timestamp = TItem::Timestamp>,
    TFilter::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    S: Stream<Item = StreamItem<TFilter>> + Send + Sync + 'static,
{
    fn take_while_with(
        self,
        filter_stream: S,
        filter: impl Fn(&TFilter::Inner) -> bool + Send + Sync + 'static,
    ) -> FluxionStream<impl Stream<Item = StreamItem<TItem>> + Send> {
        let filter = Arc::new(filter);

        // Tag each stream with its type - unwrap StreamItem first
        let source_stream = self.map(|item| match item {
            StreamItem::Value(value) => Item::<TItem, TFilter>::Source(value),
            StreamItem::Error(e) => Item::<TItem, TFilter>::Error(e),
        });
        let filter_stream = filter_stream.map(|item| match item {
            StreamItem::Value(value) => Item::<TItem, TFilter>::Filter(value),
            StreamItem::Error(e) => Item::<TItem, TFilter>::Error(e),
        });

        // Box the streams to make them the same type
        let streams: Vec<PinnedItemStream<TItem, TFilter>> =
            vec![Box::pin(source_stream), Box::pin(filter_stream)];

        // State to track the latest filter value and termination
        let state = Arc::new(Mutex::new((None::<TFilter::Inner>, false)));

        // Use ordered_merge and process items in order
        let combined_stream = streams.ordered_merge().filter_map({
            let state = Arc::clone(&state);
            move |item| {
                let state = Arc::clone(&state);
                let filter = Arc::clone(&filter);

                async move {
                    // Restrict the mutex guard's lifetime to the smallest possible scope
                    let mut guard = lock_or_recover(&state, "take_while_with state");
                    let (filter_state, terminated) = &mut *guard;

                    if *terminated {
                        return None;
                    }

                    match item {
                        Item::Error(e) => Some(StreamItem::Error(e)),
                        Item::Filter(filter_val) => {
                            *filter_state = Some(filter_val.clone().into_inner());
                            None
                        }
                        Item::Source(source_val) => filter_state.as_ref().map_or_else(
                            || None,
                            |fval| {
                                if filter(fval) {
                                    Some(StreamItem::Value(source_val.clone()))
                                } else {
                                    *terminated = true;
                                    None
                                }
                            },
                        ),
                    }
                }
            }
        });

        FluxionStream::new(Box::pin(combined_stream))
    }
}

#[derive(Clone, Debug)]
pub enum Item<TItem, TFilter> {
    Source(TItem),
    Filter(TFilter),
    Error(FluxionError),
}

impl<TItem, TFilter> HasTimestamp for Item<TItem, TFilter>
where
    TItem: HasTimestamp,
    TFilter: HasTimestamp<Timestamp = TItem::Timestamp>,
{
    type Inner = TItem::Inner;
    type Timestamp = TItem::Timestamp;

    fn timestamp(&self) -> Self::Timestamp {
        match self {
            Self::Source(s) => s.timestamp(),
            Self::Filter(f) => f.timestamp(),
            Self::Error(_) => panic!("Error items cannot provide timestamps"),
        }
    }
}

impl<TItem, TFilter> PartialEq for Item<TItem, TFilter>
where
    TItem: HasTimestamp,
    TFilter: HasTimestamp<Timestamp = TItem::Timestamp>,
{
    fn eq(&self, other: &Self) -> bool {
        self.timestamp() == other.timestamp()
    }
}

impl<TItem, TFilter> Eq for Item<TItem, TFilter>
where
    TItem: HasTimestamp,
    TFilter: HasTimestamp<Timestamp = TItem::Timestamp>,
{
}

impl<TItem, TFilter> PartialOrd for Item<TItem, TFilter>
where
    TItem: HasTimestamp,
    TFilter: HasTimestamp<Timestamp = TItem::Timestamp>,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<TItem, TFilter> Ord for Item<TItem, TFilter>
where
    TItem: HasTimestamp,
    TFilter: HasTimestamp<Timestamp = TItem::Timestamp>,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.timestamp().cmp(&other.timestamp())
    }
}
