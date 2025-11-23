// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::combine_latest::CombineLatestExt;
use crate::combine_with_previous::CombineWithPreviousExt;
use crate::emit_when::EmitWhenExt;
use crate::ordered_merge::OrderedStreamExt;
use crate::take_latest_when::TakeLatestWhenExt;
use crate::take_while_with::TakeWhileExt;
use crate::types::{CombinedState, WithPrevious};
use crate::with_latest_from::WithLatestFromExt;
use fluxion_core::ComparableUnpin;
use fluxion_core::{FluxionError, StreamItem, Timestamped};
use futures::future::ready;
use futures::Stream;
use futures::StreamExt;
use pin_project::pin_project;
use std::fmt::Debug;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_stream::wrappers::UnboundedReceiverStream;

/// A concrete wrapper type that provides all fluxion stream extensions.
///
/// This type wraps any stream of ordered items and provides all the fluxion
/// extension methods directly, allowing easy chaining and composition.
///
/// `FluxionStream` is designed for **pure, functional stream operations** with no
/// mutation. For testing scenarios where you need to push values into a stream,
/// use `tokio::sync::mpsc::unbounded_channel` with `from_unbounded_receiver`.
///
/// # Design Philosophy
///
/// - **Production code**: Uses `FluxionStream` for composable, immutable stream transformations
/// - **Test code**: Uses `tokio::sync::mpsc` channels for imperative test setup
///
/// This separation solves the fundamental conflict between:
/// - Consuming operations (stream extensions that take `self`)
/// - Mutation operations (sending values via channels)
#[pin_project]
pub struct FluxionStream<S> {
    #[pin]
    inner: S,
}

impl<S> FluxionStream<S> {
    /// Wrap a stream in a `FluxionStream` wrapper
    pub const fn new(stream: S) -> Self {
        Self { inner: stream }
    }

    /// Unwrap to get the inner stream
    pub fn into_inner(self) -> S {
        self.inner
    }
}

// Separate impl for the constructor that changes the type parameter
impl FluxionStream<()> {
    /// Creates a `FluxionStream` from a tokio unbounded receiver.
    ///
    /// This is the most common constructor for production code that receives
    /// values from other async tasks or components.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fluxion_stream::FluxionStream;
    /// use tokio::sync::mpsc;
    ///
    /// let (tx, rx) = mpsc::unbounded_channel::<i32>();
    /// let stream = FluxionStream::from_unbounded_receiver(rx);
    /// ```
    pub fn from_unbounded_receiver<T>(
        receiver: tokio::sync::mpsc::UnboundedReceiver<T>,
    ) -> FluxionStream<impl Stream<Item = StreamItem<T>>> {
        FluxionStream::new(UnboundedReceiverStream::new(receiver).map(StreamItem::Value))
    }
}

impl<T> From<UnboundedReceiverStream<T>> for FluxionStream<UnboundedReceiverStream<T>> {
    fn from(stream: UnboundedReceiverStream<T>) -> Self {
        FluxionStream::new(stream)
    }
}

impl<S> Stream for FluxionStream<S>
where
    S: Stream,
{
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<S, T> FluxionStream<S>
where
    S: Stream<Item = StreamItem<T>>,
    T: ComparableUnpin,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin,
{
    /// Maps each item to a new value while preserving temporal ordering.
    ///
    /// This operator transforms each item in the stream using the provided function.
    /// Unlike the standard `map` operator from [`StreamExt`], `map_ordered` maintains
    /// the ordering guarantees of the wrapped stream, ensuring that transformed items
    /// are emitted in temporal order.
    ///
    /// # Why use `map_ordered` instead of `StreamExt::map`?
    ///
    /// The standard `map` operator breaks the `FluxionStream` wrapper, losing temporal
    /// ordering guarantees. Using `map_ordered` preserves the ordering contract needed
    /// for correct operator chaining.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that transforms each item of type `T` into type `U`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_stream::{FluxionStream, CombineWithPreviousExt};
    /// use fluxion_test_utils::{Sequenced, test_channel, helpers::unwrap_stream, unwrap_value};
    /// use fluxion_core::Timestamped as TimestampedTrait;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let (tx, stream) = test_channel::<Sequenced<i32>>();
    /// let stream = FluxionStream::new(stream);
    ///
    /// // Transform ordered stream to strings
    /// let mut mapped = stream
    ///     .combine_with_previous()
    ///     .map_ordered(|with_previous| {
    ///         format!("Value: {}", with_previous.current.value)
    ///     });
    ///
    /// tx.send(Sequenced::new(42)).unwrap();
    /// assert_eq!(unwrap_value(Some(unwrap_stream(&mut mapped, 500).await)), "Value: 42");
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// This operator does not produce errors under normal operation. The mapping function is applied
    /// to unwrapped values, and any errors from upstream are passed through unchanged.
    ///
    /// If your mapping function can fail, consider using a pattern like:
    ///
    /// ```rust
    /// # use fluxion_core::{StreamItem, FluxionError};
    /// # use futures::Stream;
    /// # fn example<T, S: Stream>(stream: S, process: impl Fn(T) -> Result<T, String>) {
    /// // stream.map_ordered(|item| {
    /// //     match process(item) {
    /// //         Ok(result) => StreamItem::Value(result),
    /// //         Err(e) => StreamItem::Error(FluxionError::UserError(e)),
    /// //     }
    /// // })
    /// # }
    /// ```
    ///
    /// See the [Error Handling Guide](../docs/ERROR-HANDLING.md) for comprehensive error handling patterns.
    ///
    /// # See Also
    ///
    /// - [`filter_ordered`](FluxionStream::filter_ordered) - Filter items while preserving order
    /// - [`combine_with_previous`](crate::CombineWithPreviousExt::combine_with_previous) - Often used before mapping
    ///
    /// [`StreamExt`]: futures::StreamExt
    pub fn map_ordered<U, F>(
        self,
        mut f: F,
    ) -> FluxionStream<impl Stream<Item = StreamItem<U>> + Send + Sync>
    where
        S: Send + Sync + Unpin + 'static,
        U: Send + 'static,
        F: FnMut(T) -> U + Send + Sync + 'static,
    {
        let inner = self.into_inner();
        FluxionStream::new(inner.map(move |item| item.map(&mut f)))
    }

    /// Filters items based on a predicate while preserving temporal ordering.
    ///
    /// This operator selectively emits items from the stream based on a predicate function
    /// that evaluates the inner value. Items that satisfy the predicate are emitted, while
    /// others are filtered out. Temporal ordering is maintained across filtered items.
    ///
    /// # Behavior
    ///
    /// - Predicate receives a reference to the **inner value** (`&T::Inner`), not the full ordered wrapper
    /// - Only items where predicate returns `true` are emitted
    /// - Filtered items preserve their original temporal order
    /// - Emitted items are wrapped in `StreamItem::Value`
    ///
    /// # Arguments
    ///
    /// * `predicate` - A function that takes `&T::Inner` and returns `true` to keep the item
    ///
    /// # Returns
    ///
    /// A stream of `StreamItem<T>` containing only items that passed the filter
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_stream::FluxionStream;
    /// use fluxion_test_utils::{Sequenced, helpers::unwrap_stream, unwrap_value};
    /// use fluxion_core::Timestamped as TimestampedTrait;
    /// use tokio::sync::mpsc;
    ///
    /// # async fn example() {
    /// let (tx, rx) = mpsc::unbounded_channel::<Sequenced<i32>>();
    /// let stream = FluxionStream::from_unbounded_receiver(rx);
    ///
    /// // Filter for even numbers
    /// let mut evens = stream.filter_ordered(|&n| n % 2 == 0);
    ///
    /// tx.send(Sequenced::new(1)).unwrap();
    /// tx.send(Sequenced::new(2)).unwrap();
    /// tx.send(Sequenced::new(3)).unwrap();
    /// tx.send(Sequenced::new(4)).unwrap();
    ///
    /// assert_eq!(&unwrap_value(Some(unwrap_stream(&mut evens, 500).await)).value, &2);
    /// assert_eq!(&unwrap_value(Some(unwrap_stream(&mut evens, 500).await)).value, &4);
    /// # }
    /// ```
    ///
    /// # Use Cases
    ///
    /// - **Threshold Filtering**: Remove values outside acceptable ranges
    /// - **Type-Based Filtering**: Select specific enum variants
    /// - **Conditional Processing**: Process only relevant events
    /// - **Noise Reduction**: Remove unwanted data from streams
    ///
    /// # Errors
    ///
    /// This operator does not produce errors under normal operation. The predicate function receives
    /// unwrapped values, and any errors from upstream are passed through unchanged.
    ///
    /// If your predicate function can panic, ensure it handles edge cases gracefully. See the
    /// [Error Handling Guide](../docs/ERROR-HANDLING.md) for patterns on defensive filtering.
    ///
    /// # See Also
    ///
    /// - [`map_ordered`](FluxionStream::map_ordered) - Transform items while preserving order
    /// - [`emit_when`](FluxionStream::emit_when) - Filter based on secondary stream conditions
    /// - [`take_while_with`](FluxionStream::take_while_with) - Filter until condition fails, then stop
    pub fn filter_ordered<F>(
        self,
        mut predicate: F,
    ) -> FluxionStream<impl Stream<Item = StreamItem<T>> + Send + Sync>
    where
        S: Send + Sync + Unpin + 'static,
        F: FnMut(&T::Inner) -> bool + Send + Sync + 'static,
    {
        use StreamItem;
        let inner = self.into_inner();
        FluxionStream::new(inner.filter_map(move |item| {
            futures::future::ready(match item {
                StreamItem::Value(value) if predicate(&value.clone().into_inner()) => {
                    Some(StreamItem::Value(value))
                }
                StreamItem::Value(_) => None,
                StreamItem::Error(e) => Some(StreamItem::Error(e)),
            })
        }))
    }

    /// Pairs each item with its previous value, creating a sliding window of consecutive items.
    ///
    /// This operator transforms a stream of items into a stream of [`WithPrevious`] structs,
    /// where each emission contains both the current item and the previous item (if any).
    /// The first emission will have `previous = None`.
    ///
    /// This is useful for detecting changes, calculating deltas, or performing operations
    /// that require comparing consecutive values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_stream::FluxionStream;
    /// use fluxion_test_utils::{Sequenced, helpers::unwrap_stream, unwrap_value, test_channel};
    /// use fluxion_core::Timestamped as TimestampedTrait;
    ///
    /// # async fn example() {
    /// let (tx, stream) = test_channel::<Sequenced<i32>>();
    /// let mut paired = FluxionStream::new(stream).combine_with_previous();
    ///
    /// tx.send(Sequenced::new(10)).unwrap();
    /// tx.send(Sequenced::new(20)).unwrap();
    /// tx.send(Sequenced::new(30)).unwrap();
    ///
    /// // First item has no previous
    /// let item = unwrap_value(Some(unwrap_stream(&mut paired, 500).await));
    /// assert!(item.previous.is_none());
    /// assert_eq!(&item.current.value, &10);
    ///
    /// // Second item has previous value
    /// let item = unwrap_value(Some(unwrap_stream(&mut paired, 500).await));
    /// assert_eq!(&item.previous.unwrap().value, &10);
    /// assert_eq!(&item.current.value, &20);
    ///
    /// // Third item pairs 20 and 30
    /// let item = unwrap_value(Some(unwrap_stream(&mut paired, 500).await));
    /// assert_eq!(&item.previous.unwrap().value, &20);
    /// assert_eq!(&item.current.value, &30);
    /// # }
    /// ```
    ///
    /// # Use Cases
    ///
    /// - **Change Detection**: Compare current and previous values to detect changes
    /// - **Delta Calculation**: Compute differences between consecutive values
    /// - **State Transitions**: Track transitions between states
    /// - **Trend Analysis**: Analyze patterns across consecutive items
    ///
    /// # See Also
    ///
    /// - [`ordered_merge`](FluxionStream::ordered_merge) - Often used before this operator
    /// - [`take_latest_when`](FluxionStream::take_latest_when) - Can be chained after this
    pub fn combine_with_previous(
        self,
    ) -> FluxionStream<impl Stream<Item = StreamItem<WithPrevious<T>>> + Send + Sync>
    where
        S: Send + Sync + Unpin + 'static,
    {
        let inner = self.into_inner();
        CombineWithPreviousExt::combine_with_previous(inner)
    }

    /// Emits items from the source stream while a predicate on a separate filter stream remains true.
    ///
    /// This operator conditionally emits items from the source stream based on the state of a
    /// separate filter stream. Items are emitted as long as the most recent value from the filter
    /// stream satisfies the provided predicate. Once the predicate returns false, the stream
    /// completes and no further items are emitted.
    ///
    /// # Arguments
    ///
    /// * `filter_stream` - A stream whose latest value determines whether to emit items
    /// * `filter` - A predicate function that evaluates the filter stream's value
    ///
    /// # Behavior
    ///
    /// - Items from the source stream are emitted only when the filter predicate is true
    /// - The filter predicate is evaluated against the **latest value** from the filter stream
    /// - Once the predicate returns false, the stream terminates
    /// - Emitted items are unwrapped to their inner type (`T::Inner`)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_stream::FluxionStream;
    /// use fluxion_test_utils::{Sequenced, helpers::unwrap_stream, unwrap_value, test_channel};
    /// use fluxion_core::Timestamped as TimestampedTrait;
    ///
    /// # async fn example() {
    /// let (source_tx, source) = test_channel::<Sequenced<i32>>();
    /// let (filter_tx, filter) = test_channel::<Sequenced<i32>>();
    ///
    /// let mut taken = FluxionStream::new(source).take_while_with(FluxionStream::new(filter), |value: &i32| *value > 0);
    ///
    /// // Filter allows emissions (value = 1)
    /// filter_tx.send(Sequenced::new(1)).unwrap();
    /// source_tx.send(Sequenced::new(100)).unwrap();
    /// source_tx.send(Sequenced::new(200)).unwrap();
    ///
    /// assert_eq!(&unwrap_value(Some(unwrap_stream(&mut taken, 500).await)).value, &100);
    /// assert_eq!(&unwrap_value(Some(unwrap_stream(&mut taken, 500).await)).value, &200);
    /// # }
    /// ```
    ///
    /// # See Also
    ///
    /// - [`take_latest_when`](FluxionStream::take_latest_when) - Sample on trigger events
    /// - [`emit_when`](FluxionStream::emit_when) - Gate emissions with combined state
    /// - [`filter_ordered`](FluxionStream::filter_ordered) - Simple filtering without external stream
    pub fn take_while_with<TFilter, SF>(
        self,
        filter_stream: SF,
        filter: impl Fn(&TFilter::Inner) -> bool + Send + Sync + 'static,
    ) -> FluxionStream<impl Stream<Item = StreamItem<T>> + Send + Sync>
    where
        S: Stream<Item = StreamItem<T>> + Send + Sync + Unpin + 'static,
        TFilter: Timestamped<Timestamp = T::Timestamp>
            + Clone
            + Debug
            + Ord
            + Send
            + Sync
            + Unpin
            + 'static,
        TFilter::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
        SF: Stream<Item = StreamItem<TFilter>> + Send + Sync + 'static,
    {
        let inner = self.into_inner();
        FluxionStream::new(TakeWhileExt::take_while_with(inner, filter_stream, filter))
    }

    /// Emits the latest value from the source stream when the trigger stream emits.
    ///
    /// This operator implements sampling behavior: it buffers values from the source stream
    /// and emits the most recent value whenever the trigger stream produces a value that
    /// satisfies the filter predicate.
    ///
    /// # Arguments
    ///
    /// * `filter_stream` - A trigger stream that controls when to emit
    /// * `filter` - A predicate function that determines if a trigger should cause emission
    ///
    /// # Behavior
    ///
    /// - **Before first trigger**: Source values are buffered but not emitted
    /// - **On trigger emission**: The latest buffered source value is emitted with the trigger's sequence number
    /// - **After trigger**: Subsequent source values are emitted immediately with their own sequence numbers
    /// - The filter predicate must return true for the trigger to cause emission
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_stream::FluxionStream;
    /// use fluxion_test_utils::{Sequenced, helpers::unwrap_stream, unwrap_value, test_channel};
    /// use fluxion_core::{HasTimestamp, Timestamped as TimestampedTrait};
    ///
    /// # async fn example() {
    /// let (source_tx, source) = test_channel::<Sequenced<i32>>();
    /// let (trigger_tx, trigger) = test_channel::<Sequenced<i32>>();
    ///
    /// let mut sampled = FluxionStream::new(source).take_latest_when(FluxionStream::new(trigger), |_| true);
    ///
    /// // Buffer source values
    /// source_tx.send((10, 1).into()).unwrap();
    /// source_tx.send((20, 2).into()).unwrap();
    /// source_tx.send((30, 3).into()).unwrap();
    ///
    /// // Trigger emission - emits latest buffered value (30)
    /// trigger_tx.send((0, 4).into()).unwrap();
    ///
    /// let result = unwrap_value(Some(unwrap_stream(&mut sampled, 500).await));
    /// assert_eq!(&result.value, &30);
    /// assert_eq!(result.timestamp(), 4); // Uses trigger's sequence
    ///
    /// // After trigger, source values emit immediately
    /// source_tx.send((40, 5).into()).unwrap();
    /// let result = unwrap_value(Some(unwrap_stream(&mut sampled, 500).await));
    /// assert_eq!(&result.value, &40);
    /// assert_eq!(result.timestamp(), 5); // Uses source's sequence
    /// # }
    /// ```
    ///
    /// # Use Cases
    ///
    /// - **Rate Limiting**: Sample a high-frequency stream at a lower rate
    /// - **Conditional Sampling**: Emit values only when specific conditions are met
    /// - **Event-Driven Processing**: Process data only when triggered by external events
    ///
    /// # See Also
    ///
    /// - [`take_while_with`](FluxionStream::take_while_with) - Conditional take until predicate fails
    /// - [`emit_when`](FluxionStream::emit_when) - Similar but with combined state filtering
    /// - [`with_latest_from`](FluxionStream::with_latest_from) - Sample on primary stream emissions
    pub fn take_latest_when<SF>(
        self,
        filter_stream: SF,
        filter: impl Fn(&T::Inner) -> bool + Send + Sync + 'static,
    ) -> FluxionStream<impl Stream<Item = StreamItem<T>> + Send + Sync>
    where
        S: Stream<Item = StreamItem<T>> + Send + Sync + Unpin + 'static,
        SF: Stream<Item = StreamItem<T>> + Send + Sync + 'static,
    {
        let inner = self.into_inner();
        FluxionStream::new(TakeLatestWhenExt::take_latest_when(
            inner,
            filter_stream,
            filter,
        ))
    }

    /// Gates emissions based on a predicate that evaluates the combined state of source and filter streams.
    ///
    /// This operator combines the source stream with a filter stream and only emits source values
    /// when the filter predicate (evaluated on the combined state) returns true. Unlike
    /// [`take_latest_when`](FluxionStream::take_latest_when), the filter has access to both
    /// the source and filter values in the combined state.
    ///
    /// # Arguments
    ///
    /// * `filter_stream` - A stream whose values are combined with the source for filtering
    /// * `filter` - A predicate function that evaluates the combined state of both streams
    ///
    /// # Behavior
    ///
    /// - Source and filter streams are combined into a [`CombinedState`]
    /// - The predicate receives the combined state with both values
    /// - Only source values that satisfy the predicate are emitted
    /// - Emitted values preserve their original sequence numbers
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_stream::{FluxionStream, CombinedState};
    /// use fluxion_test_utils::{Sequenced, helpers::unwrap_stream, unwrap_value, test_channel};
    /// use fluxion_core::Timestamped as TimestampedTrait;
    ///
    /// # async fn example() {
    /// let (source_tx, source) = test_channel::<Sequenced<i32>>();
    /// let (threshold_tx, threshold) = test_channel::<Sequenced<i32>>();
    ///
    /// // Emit only when source value exceeds threshold
    /// let mut gated = FluxionStream::new(source).emit_when(FluxionStream::new(threshold), |state: &CombinedState<i32>| {
    ///     let values = state.values();
    ///     values[0] > values[1] // source > threshold
    /// });
    ///
    /// threshold_tx.send(Sequenced::new(50)).unwrap();
    ///
    /// source_tx.send(Sequenced::new(30)).unwrap(); // Below threshold, not emitted
    /// source_tx.send(Sequenced::new(60)).unwrap(); // Above threshold, emitted
    ///
    /// let result = unwrap_value(Some(unwrap_stream(&mut gated, 500).await));
    /// assert_eq!(&result.value, &60);
    /// # }
    /// ```
    ///
    /// # Use Cases
    ///
    /// - **Threshold Filtering**: Emit only when values exceed dynamic thresholds
    /// - **Conditional Gating**: Control emissions based on runtime conditions
    /// - **Cross-Stream Validation**: Validate source values against reference streams
    ///
    /// # See Also
    ///
    /// - [`take_latest_when`](FluxionStream::take_latest_when) - Simpler trigger-based sampling
    /// - [`filter_ordered`](FluxionStream::filter_ordered) - Filter based on source value only
    /// - [`combine_latest`](FluxionStream::combine_latest) - Combine multiple streams without filtering
    pub fn emit_when<SF>(
        self,
        filter_stream: SF,
        filter: impl Fn(&CombinedState<T::Inner, T::Timestamp>) -> bool + Send + Sync + 'static,
    ) -> FluxionStream<impl Stream<Item = StreamItem<T>> + Send + Sync>
    where
        S: Stream<Item = StreamItem<T>> + Send + Sync + Unpin + 'static,
        SF: Stream<Item = StreamItem<T>> + Send + Sync + 'static,
    {
        let inner = self.into_inner();
        FluxionStream::new(EmitWhenExt::emit_when(inner, filter_stream, filter))
    }

    /// Combines each emission from the primary stream with the latest value from a secondary stream.
    ///
    /// This operator implements a **primary-driven** combination pattern where emissions occur
    /// only when the primary (self) stream emits. The latest value from the secondary stream
    /// is sampled and combined with each primary emission using the provided selector function.
    ///
    /// # Arguments
    ///
    /// * `other` - The secondary stream to sample from
    /// * `result_selector` - A function that transforms the combined state into the output value
    ///
    /// # Behavior
    ///
    /// - Emissions occur **only when the primary stream emits**
    /// - Each primary emission samples the **latest value** from the secondary stream
    /// - The combined state is passed to the result selector to produce the output
    /// - Sequence numbers from the primary stream are preserved
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_stream::{FluxionStream, CombinedState};
    /// use fluxion_test_utils::{Sequenced, helpers::unwrap_stream, unwrap_value, test_channel};
    /// use fluxion_core::Timestamped as TimestampedTrait;
    ///
    /// # async fn example() {
    /// let (primary_tx, primary) = test_channel::<Sequenced<i32>>();
    /// let (secondary_tx, secondary) = test_channel::<Sequenced<i32>>();
    ///
    /// let mut combined = FluxionStream::new(primary).with_latest_from(FluxionStream::new(secondary), |state: &CombinedState<i32, u64>| {
    ///     state.clone() // Return the combined state itself
    /// });
    ///
    /// // Set secondary value
    /// secondary_tx.send(Sequenced::new(100)).unwrap();
    ///
    /// // Primary emissions drive output
    /// primary_tx.send(Sequenced::new(1)).unwrap();
    /// primary_tx.send(Sequenced::new(2)).unwrap();
    ///
    /// let result = unwrap_value(Some(unwrap_stream(&mut combined, 500).await));
    /// let values = result.values();
    /// assert_eq!(values[0] + values[1], 101); // 1 + 100
    /// # }
    /// ```
    ///
    /// # Use Cases
    ///
    /// - **Configuration Sampling**: Combine data streams with configuration streams
    /// - **Context Enrichment**: Add context from reference streams to primary events
    /// - **Rate Transformation**: Sample slow streams with fast primary streams
    ///
    /// # See Also
    ///
    /// - [`combine_latest`](FluxionStream::combine_latest) - All streams drive emissions
    /// - [`take_latest_when`](FluxionStream::take_latest_when) - Simpler trigger-based sampling
    pub fn with_latest_from<S2, R>(
        self,
        other: S2,
        result_selector: impl Fn(&CombinedState<T::Inner, T::Timestamp>) -> R + Send + Sync + 'static,
    ) -> FluxionStream<impl Stream<Item = StreamItem<R>> + Send + Sync>
    where
        S: Stream<Item = StreamItem<T>> + Send + Sync + Unpin + 'static,
        S2: Stream<Item = StreamItem<T>> + Send + Sync + 'static,
        R: Timestamped<Timestamp = T::Timestamp> + Clone + Debug + Ord + Send + Sync + 'static,
    {
        let inner = self.into_inner();
        FluxionStream::new(WithLatestFromExt::with_latest_from(
            inner,
            other,
            result_selector,
        ))
    }

    /// Combines the latest values from multiple streams into a single stream of combined states.
    ///
    /// This operator merges the primary stream with one or more secondary streams, emitting
    /// a [`CombinedState`] whenever **any** of the streams produces a value. Each emission
    /// contains the latest value from all streams. Emissions can be filtered using the provided
    /// predicate.
    ///
    /// # Arguments
    ///
    /// * `others` - A vector of secondary streams to combine with the primary stream
    /// * `filter` - A predicate that determines whether to emit the combined state
    ///
    /// # Behavior
    ///
    /// - Emissions occur when **any** input stream emits a value
    /// - Each emission contains the **latest value from all streams**
    /// - First emission requires at least one value from each stream
    /// - The filter predicate can suppress emissions that don't meet criteria
    /// - Combined state preserves temporal ordering across all streams
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_stream::FluxionStream;
    /// use fluxion_test_utils::{Sequenced, helpers::unwrap_stream, unwrap_value, test_channel};
    /// use fluxion_core::Timestamped as TimestampedTrait;
    ///
    /// # async fn example() {
    /// let (tx1, stream1) = test_channel::<Sequenced<i32>>();
    /// let (tx2, stream2) = test_channel::<Sequenced<i32>>();
    ///
    /// let mut combined = FluxionStream::new(stream1).combine_latest(vec![FluxionStream::new(stream2)], |_| true);
    ///
    /// tx1.send(Sequenced::new(10)).unwrap();
    /// tx2.send(Sequenced::new(20)).unwrap();
    ///
    /// let result = unwrap_value(Some(unwrap_stream(&mut combined, 500).await));
    /// let state = result.values();
    /// assert_eq!(state[0], 10); // First stream's value
    /// assert_eq!(state[1], 20); // Second stream's value
    ///
    /// // When either stream updates, a new combined state is emitted
    /// tx1.send(Sequenced::new(30)).unwrap();
    /// let result = unwrap_value(Some(unwrap_stream(&mut combined, 500).await));
    /// let state = result.values();
    /// assert_eq!(state[0], 30); // Updated
    /// assert_eq!(state[1], 20); // Still latest from stream2
    /// # }
    /// ```
    ///
    /// # Use Cases
    ///
    /// - **Multi-Source Aggregation**: Combine data from multiple independent sources
    /// - **Coordinated Updates**: React to changes across multiple streams
    /// - **State Synchronization**: Maintain synchronized view of multiple data sources
    /// - **Complex Filtering**: Filter based on relationships between multiple streams
    ///
    /// # See Also
    ///
    /// - [`with_latest_from`](FluxionStream::with_latest_from) - Primary-driven combination
    /// - [`ordered_merge`](FluxionStream::ordered_merge) - Merge without combining values
    /// - [`emit_when`](FluxionStream::emit_when) - Similar filtering with different semantics
    pub fn combine_latest<S2>(
        self,
        others: Vec<S2>,
        filter: impl Fn(&CombinedState<T::Inner, T::Timestamp>) -> bool + Send + Sync + 'static,
    ) -> FluxionStream<impl Stream<Item = StreamItem<CombinedState<T::Inner, T::Timestamp>>> + Send>
    where
        S: Stream<Item = StreamItem<T>> + Send + Sync + 'static,
        S2: Stream<Item = StreamItem<T>> + Send + Sync + 'static,
    {
        let inner = self.into_inner();
        FluxionStream::new(CombineLatestExt::combine_latest(inner, others, filter))
    }

    /// Merges multiple streams into a single stream with guaranteed temporal ordering.
    ///
    /// This operator combines the primary stream with one or more secondary streams into a unified
    /// stream where all items are emitted in temporal order based on their sequence numbers.
    /// This is the fundamental operator for aggregating multiple event sources while preserving
    /// temporal consistency.
    ///
    /// # Arguments
    ///
    /// * `others` - A vector of secondary streams to merge with the primary stream
    ///
    /// # Behavior
    ///
    /// - Items from all streams are emitted in **temporal order** by sequence number
    /// - Each stream's items maintain their original values and types
    /// - No combination or transformation occurs—items flow through as-is
    /// - All input streams must have the same item type
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_stream::FluxionStream;
    /// use fluxion_test_utils::{Sequenced, helpers::unwrap_stream, unwrap_value, test_channel};
    /// use fluxion_core::Timestamped as TimestampedTrait;
    ///
    /// # async fn example() {
    /// let (tx1, stream1) = test_channel::<Sequenced<&str>>();
    /// let (tx2, stream2) = test_channel::<Sequenced<&str>>();
    /// let (tx3, stream3) = test_channel::<Sequenced<&str>>();
    ///
    /// let mut merged = FluxionStream::new(stream1).ordered_merge(vec![FluxionStream::new(stream2), FluxionStream::new(stream3)]);
    ///
    /// // Send out of order across streams
    /// tx1.send(("first", 1).into()).unwrap();
    /// tx3.send(("third", 3).into()).unwrap();
    /// tx2.send(("second", 2).into()).unwrap();
    ///
    /// // Items are emitted in temporal order
    /// assert_eq!(&unwrap_value(Some(unwrap_stream(&mut merged, 500).await)).value, &"first");
    /// assert_eq!(&unwrap_value(Some(unwrap_stream(&mut merged, 500).await)).value, &"second");
    /// assert_eq!(&unwrap_value(Some(unwrap_stream(&mut merged, 500).await)).value, &"third");
    /// # }
    /// ```
    ///
    /// # Use Cases
    ///
    /// - **Event Aggregation**: Merge events from multiple sources into a single timeline
    /// - **Multi-Producer Systems**: Combine outputs from multiple producers
    /// - **Distributed Systems**: Merge streams from different services or nodes
    /// - **Pipeline Composition**: Foundation for building complex operator chains
    ///
    /// # See Also
    ///
    /// - [`combine_latest`](FluxionStream::combine_latest) - Merge with value combination
    /// - [`combine_with_previous`](FluxionStream::combine_with_previous) - Often chained after merge
    /// - **[stream-aggregation example](https://github.com/umbgtt10/fluxion/tree/main/examples/stream-aggregation)** - Production pattern
    pub fn ordered_merge<S2>(
        self,
        others: Vec<FluxionStream<S2>>,
    ) -> FluxionStream<impl Stream<Item = StreamItem<T>> + Send + Sync>
    where
        S: Stream<Item = StreamItem<T>> + Send + Sync + 'static,
        S2: Stream<Item = StreamItem<T>> + Send + Sync + 'static,
    {
        let inner = self.into_inner();
        let other_streams: Vec<S2> = others.into_iter().map(|fs| fs.into_inner()).collect();
        FluxionStream::new(OrderedStreamExt::ordered_merge(inner, other_streams))
    }

    /// Handle errors in the stream with a handler function.
    ///
    /// The handler receives a reference to each error and returns:
    /// - `true` to consume the error (remove from stream)
    /// - `false` to propagate the error downstream
    ///
    /// Multiple `on_error` operators can be chained to implement the
    /// Chain of Responsibility pattern for error handling.
    ///
    /// # Examples
    ///
    /// ## Basic Error Consumption
    ///
    /// ```rust
    /// use fluxion_stream::FluxionStream;
    /// use fluxion_core::{FluxionError, StreamItem, Timestamped};
    /// use fluxion_test_utils::{Sequenced, test_channel_with_errors, helpers::unwrap_stream};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    /// let mut stream = FluxionStream::new(stream)
    ///     .on_error(|err| {
    ///         eprintln!("Error: {}", err);
    ///         true // Consume all errors
    ///     });
    ///
    /// tx.send(StreamItem::Value(Sequenced::new(1))).unwrap();
    /// tx.send(StreamItem::Error(FluxionError::stream_error("oops"))).unwrap();
    /// tx.send(StreamItem::Value(Sequenced::new(2))).unwrap();
    ///
    /// if let StreamItem::Value(v) = unwrap_stream(&mut stream, 500).await {
    ///     assert_eq!(v.into_inner(), 1);
    /// }
    /// if let StreamItem::Value(v) = unwrap_stream(&mut stream, 500).await {
    ///     assert_eq!(v.into_inner(), 2);
    /// }
    /// # }
    /// ```
    ///
    /// ## Chain of Responsibility
    ///
    /// ```rust
    /// use fluxion_stream::FluxionStream;
    /// use fluxion_core::{FluxionError, StreamItem, Timestamped};
    /// use fluxion_test_utils::{Sequenced, test_channel_with_errors, helpers::unwrap_stream};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    /// let mut stream = FluxionStream::new(stream)
    ///     .on_error(|err| err.to_string().contains("validation"))
    ///     .on_error(|err| err.to_string().contains("network"))
    ///     .on_error(|_| true); // Catch-all
    ///
    /// tx.send(StreamItem::Error(FluxionError::stream_error("validation"))).unwrap();
    /// tx.send(StreamItem::Error(FluxionError::stream_error("network"))).unwrap();
    /// tx.send(StreamItem::Value(Sequenced::new(1))).unwrap();
    ///
    /// if let StreamItem::Value(v) = unwrap_stream(&mut stream, 500).await {
    ///     assert_eq!(v.into_inner(), 1);
    /// }
    /// # }
    /// ```
    ///
    /// # See Also
    ///
    /// - [Error Handling Guide](../docs/ERROR-HANDLING.md) - Comprehensive error patterns
    pub fn on_error<F>(self, mut handler: F) -> FluxionStream<impl Stream<Item = StreamItem<T>>>
    where
        S: Stream<Item = StreamItem<T>>,
        F: FnMut(&FluxionError) -> bool,
    {
        let inner = self.into_inner();
        FluxionStream::new(inner.filter_map(move |item| {
            ready(match item {
                StreamItem::Error(err) => {
                    if handler(&err) {
                        // Error handled, skip it
                        None
                    } else {
                        // Error not handled, propagate
                        Some(StreamItem::Error(err))
                    }
                }
                other => Some(other),
            })
        }))
    }
}
