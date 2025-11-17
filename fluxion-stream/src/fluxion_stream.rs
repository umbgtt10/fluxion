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
use crate::Ordered;
use fluxion_core::{CompareByInner, OrderedWrapper};
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
    ) -> FluxionStream<impl Stream<Item = fluxion_core::StreamItem<T>>> {
        FluxionStream::new(
            UnboundedReceiverStream::new(receiver).map(fluxion_core::StreamItem::Value),
        )
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
    S: Stream<Item = fluxion_core::StreamItem<T>>,
    T: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
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
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    /// use tokio::sync::mpsc;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let (tx, rx) = mpsc::unbounded_channel::<Sequenced<i32>>();
    /// let stream = FluxionStream::from_unbounded_receiver(rx);
    ///
    /// // Transform ordered stream to strings
    /// let mut mapped = stream
    ///     .combine_with_previous()
    ///     .map_ordered(|stream_item| {
    ///         let item = stream_item.unwrap();
    ///         format!("Value: {}", item.current.get())
    ///     });
    ///
    /// tx.send(Sequenced::new(42)).unwrap();
    /// assert_eq!(mapped.next().await.unwrap().unwrap(), "Value: 42");
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
    /// ```rust,ignore
    /// stream.map_ordered(|item| {
    ///     match process(item) {
    ///         Ok(result) => StreamItem::Value(result),
    ///         Err(e) => StreamItem::Error(FluxionError::UserError(e.to_string())),
    ///     }
    /// })
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
    ) -> FluxionStream<impl Stream<Item = fluxion_core::StreamItem<U>> + Send + Sync>
    where
        S: Send + Sync + Unpin + 'static,
        U: Send + 'static,
        F: FnMut(T) -> U + Send + Sync + 'static,
    {
        let inner = self.into_inner();
        FluxionStream::new(inner.map(move |item| item.map(|value| f(value))))
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
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    /// use tokio::sync::mpsc;
    ///
    /// # async fn example() {
    /// let (tx, rx) = mpsc::unbounded_channel();
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
    /// assert_eq!(evens.next().await.unwrap().unwrap().get(), &2);
    /// assert_eq!(evens.next().await.unwrap().unwrap().get(), &4);
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
    ) -> FluxionStream<impl Stream<Item = fluxion_core::StreamItem<T>> + Send + Sync>
    where
        S: Send + Sync + Unpin + 'static,
        F: FnMut(&T::Inner) -> bool + Send + Sync + 'static,
    {
        use fluxion_core::StreamItem;
        let inner = self.into_inner();
        FluxionStream::new(inner.filter_map(move |item| {
            futures::future::ready(match item {
                StreamItem::Value(value) if predicate(value.get()) => {
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
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    /// use tokio::sync::mpsc;
    ///
    /// # async fn example() {
    /// let (tx, rx) = mpsc::unbounded_channel();
    /// let stream = FluxionStream::from_unbounded_receiver(rx);
    /// let mut paired = stream.combine_with_previous();
    ///
    /// tx.send(Sequenced::new(10)).unwrap();
    /// tx.send(Sequenced::new(20)).unwrap();
    /// tx.send(Sequenced::new(30)).unwrap();
    ///
    /// // First item has no previous
    /// let item = paired.next().await.unwrap().unwrap();
    /// assert!(item.previous.is_none());
    /// assert_eq!(item.current.get(), &10);
    ///
    /// // Second item has previous value
    /// let item = paired.next().await.unwrap().unwrap();
    /// assert_eq!(item.previous.unwrap().get(), &10);
    /// assert_eq!(item.current.get(), &20);
    ///
    /// // Third item pairs 20 and 30
    /// let item = paired.next().await.unwrap().unwrap();
    /// assert_eq!(item.previous.unwrap().get(), &20);
    /// assert_eq!(item.current.get(), &30);
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
    ) -> FluxionStream<impl Stream<Item = fluxion_core::StreamItem<WithPrevious<T>>> + Send + Sync>
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
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    /// use tokio::sync::mpsc;
    ///
    /// # async fn example() {
    /// let (source_tx, source_rx) = mpsc::unbounded_channel();
    /// let (filter_tx, filter_rx) = mpsc::unbounded_channel();
    ///
    /// let source = FluxionStream::from_unbounded_receiver(source_rx);
    /// let filter = FluxionStream::from_unbounded_receiver(filter_rx);
    ///
    /// let mut taken = source.take_while_with(filter, |value: &i32| *value > 0);
    ///
    /// // Filter allows emissions (value = 1)
    /// filter_tx.send(Sequenced::new(1)).unwrap();
    /// source_tx.send(Sequenced::new(100)).unwrap();
    /// source_tx.send(Sequenced::new(200)).unwrap();
    ///
    /// let mut taken = Box::pin(taken);
    /// assert_eq!(taken.next().await.unwrap().unwrap(), 100);
    /// assert_eq!(taken.next().await.unwrap().unwrap(), 200);
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
    ) -> FluxionStream<impl Stream<Item = fluxion_core::StreamItem<T::Inner>> + Send + Sync>
    where
        S: Stream<Item = fluxion_core::StreamItem<T>> + Send + Sync + Unpin + 'static,
        TFilter: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + 'static,
        TFilter::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
        SF: Stream<Item = fluxion_core::StreamItem<TFilter>> + Send + Sync + 'static,
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
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    /// use tokio::sync::mpsc;
    ///
    /// # async fn example() {
    /// let (source_tx, source_rx) = mpsc::unbounded_channel();
    /// let (trigger_tx, trigger_rx) = mpsc::unbounded_channel();
    ///
    /// let source = FluxionStream::from_unbounded_receiver(source_rx);
    /// let trigger = FluxionStream::from_unbounded_receiver(trigger_rx);
    ///
    /// let mut sampled = source.take_latest_when(trigger, |_| true);
    ///
    /// // Buffer source values
    /// source_tx.send(Sequenced::with_sequence(10, 1)).unwrap();
    /// source_tx.send(Sequenced::with_sequence(20, 2)).unwrap();
    /// source_tx.send(Sequenced::with_sequence(30, 3)).unwrap();
    ///
    /// // Trigger emission - emits latest buffered value (30)
    /// trigger_tx.send(Sequenced::with_sequence(0, 4)).unwrap();
    ///
    /// let result = sampled.next().await.unwrap().unwrap();
    /// assert_eq!(result.get(), &30);
    /// assert_eq!(result.sequence(), 4); // Uses trigger's sequence
    ///
    /// // After trigger, source values emit immediately
    /// source_tx.send(Sequenced::with_sequence(40, 5)).unwrap();
    /// let result = sampled.next().await.unwrap().unwrap();
    /// assert_eq!(result.get(), &40);
    /// assert_eq!(result.sequence(), 5); // Uses source's sequence
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
    ) -> FluxionStream<impl Stream<Item = fluxion_core::StreamItem<T>> + Send + Sync>
    where
        S: Stream<Item = fluxion_core::StreamItem<T>> + Send + Sync + 'static,
        SF: Stream<Item = fluxion_core::StreamItem<T>> + Send + Sync + 'static,
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
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    /// use tokio::sync::mpsc;
    ///
    /// # async fn example() {
    /// let (source_tx, source_rx) = mpsc::unbounded_channel();
    /// let (threshold_tx, threshold_rx) = mpsc::unbounded_channel();
    ///
    /// let source = FluxionStream::from_unbounded_receiver(source_rx);
    /// let threshold = FluxionStream::from_unbounded_receiver(threshold_rx);
    ///
    /// // Emit only when source value exceeds threshold
    /// let mut gated = source.emit_when(threshold, |state: &CombinedState<i32>| {
    ///     let values = state.values();
    ///     values[0] > values[1] // source > threshold
    /// });
    ///
    /// threshold_tx.send(Sequenced::new(50)).unwrap();
    ///
    /// source_tx.send(Sequenced::new(30)).unwrap(); // Below threshold, not emitted
    /// source_tx.send(Sequenced::new(60)).unwrap(); // Above threshold, emitted
    ///
    /// let mut gated = Box::pin(gated);
    /// let result = gated.next().await.unwrap().unwrap();
    /// assert_eq!(result.get(), &42);
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
        filter: impl Fn(&CombinedState<T::Inner>) -> bool + Send + Sync + 'static,
    ) -> FluxionStream<impl Stream<Item = fluxion_core::StreamItem<T>> + Send + Sync>
    where
        S: Stream<Item = fluxion_core::StreamItem<T>> + Send + Sync + 'static,
        SF: Stream<Item = fluxion_core::StreamItem<T>> + Send + Sync + 'static,
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
    /// use fluxion_stream::{FluxionStream, CombinedState, Ordered};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    /// use tokio::sync::mpsc;
    ///
    /// # async fn example() {
    /// let (primary_tx, primary_rx) = mpsc::unbounded_channel();
    /// let (secondary_tx, secondary_rx) = mpsc::unbounded_channel();
    ///
    /// let primary = FluxionStream::from_unbounded_receiver(primary_rx);
    /// let secondary = FluxionStream::from_unbounded_receiver(secondary_rx);
    ///
    /// let mut combined = primary.with_latest_from(secondary, |state: &CombinedState<i32>| {
    ///     let values = state.values();
    ///     values[0] + values[1] // Sum primary and secondary
    /// });
    ///
    /// // Set secondary value
    /// secondary_tx.send(Sequenced::new(100)).unwrap();
    ///
    /// // Primary emissions drive output
    /// primary_tx.send(Sequenced::new(1)).unwrap();
    /// primary_tx.send(Sequenced::new(2)).unwrap();
    ///
    /// assert_eq!(combined.next().await.unwrap().unwrap().get(), &101); // 1 + 100
    /// assert_eq!(combined.next().await.unwrap().unwrap().get(), &102); // 2 + 100
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
        result_selector: impl Fn(&CombinedState<T::Inner>) -> R + Send + Sync + 'static,
    ) -> FluxionStream<impl Stream<Item = fluxion_core::StreamItem<OrderedWrapper<R>>> + Send + Sync>
    where
        S: Stream<Item = fluxion_core::StreamItem<T>> + Send + Sync + Unpin + 'static,
        S2: Stream<Item = fluxion_core::StreamItem<T>> + Send + Sync + 'static,
        T: CompareByInner,
        R: Clone + Debug + Ord + Send + Sync + 'static,
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
    /// use fluxion_stream::{FluxionStream, Ordered};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    /// use tokio::sync::mpsc;
    ///
    /// # async fn example() {
    /// let (tx1, rx1) = mpsc::unbounded_channel();
    /// let (tx2, rx2) = mpsc::unbounded_channel();
    ///
    /// let stream1 = FluxionStream::from_unbounded_receiver(rx1);
    /// let stream2 = FluxionStream::from_unbounded_receiver(rx2);
    ///
    /// let mut combined = stream1.combine_latest(vec![stream2], |_| true);
    ///
    /// tx1.send(Sequenced::new(10)).unwrap();
    /// tx2.send(Sequenced::new(20)).unwrap();
    ///
    /// let result = combined.next().await.unwrap();
    /// let state = result.get().values();
    /// assert_eq!(state[0], 10); // First stream's value
    /// assert_eq!(state[1], 20); // Second stream's value
    ///
    /// // When either stream updates, a new combined state is emitted
    /// tx1.send(Sequenced::new(30)).unwrap();
    /// let result = combined.next().await.unwrap();
    /// let state = result.get().values();
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
        filter: impl Fn(&CombinedState<T::Inner>) -> bool + Send + Sync + 'static,
    ) -> FluxionStream<
        impl Stream<Item = fluxion_core::StreamItem<OrderedWrapper<CombinedState<T::Inner>>>> + Send,
    >
    where
        S: Stream<Item = fluxion_core::StreamItem<T>> + Send + Sync + 'static,
        S2: Stream<Item = fluxion_core::StreamItem<T>> + Send + Sync + 'static,
        T: CompareByInner,
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
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    /// use tokio::sync::mpsc;
    ///
    /// # async fn example() {
    /// let (tx1, rx1) = mpsc::unbounded_channel();
    /// let (tx2, rx2) = mpsc::unbounded_channel();
    /// let (tx3, rx3) = mpsc::unbounded_channel();
    ///
    /// let stream1 = FluxionStream::from_unbounded_receiver(rx1);
    /// let stream2 = FluxionStream::from_unbounded_receiver(rx2);
    /// let stream3 = FluxionStream::from_unbounded_receiver(rx3);
    ///
    /// let mut merged = stream1.ordered_merge(vec![stream2, stream3]);
    ///
    /// // Send out of order across streams
    /// tx1.send(Sequenced::with_sequence("first", 1)).unwrap();
    /// tx3.send(Sequenced::with_sequence("third", 3)).unwrap();
    /// tx2.send(Sequenced::with_sequence("second", 2)).unwrap();
    ///
    /// // Items are emitted in temporal order
    /// assert_eq!(merged.next().await.unwrap().unwrap().get(), &"first");
    /// assert_eq!(merged.next().await.unwrap().unwrap().get(), &"second");
    /// assert_eq!(merged.next().await.unwrap().unwrap().get(), &"third");
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
    ) -> FluxionStream<impl Stream<Item = fluxion_core::StreamItem<T>> + Send + Sync>
    where
        S: Stream<Item = fluxion_core::StreamItem<T>> + Send + Sync + 'static,
        S2: Stream<Item = fluxion_core::StreamItem<T>> + Send + Sync + 'static,
    {
        let inner = self.into_inner();
        let other_streams: Vec<S2> = others.into_iter().map(|fs| fs.into_inner()).collect();
        FluxionStream::new(OrderedStreamExt::ordered_merge(inner, other_streams))
    }
}
