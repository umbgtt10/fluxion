// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::Ordered;
use crate::combine_latest::CombineLatestExt;
use crate::combine_with_previous::CombineWithPreviousExt;
use crate::emit_when::EmitWhenExt;
use crate::ordered_merge::OrderedStreamExt;
use crate::take_latest_when::TakeLatestWhenExt;
use crate::take_while_with::TakeWhileExt;
use crate::types::{CombinedState, WithPrevious};
use crate::with_latest_from::WithLatestFromExt;
use fluxion_core::{CompareByInner, OrderedWrapper};
use futures::Stream;
use futures::StreamExt;
use pin_project::pin_project;
use std::fmt::Debug;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A concrete wrapper type that provides all fluxion stream extensions.
///
/// This type wraps any stream of ordered items and provides all the fluxion
/// extension methods directly, allowing easy chaining and composition.
///
/// `FluxionStream` is designed for **pure, functional stream operations** with no
/// mutation. For testing scenarios where you need to push values into a stream,
/// use `TestChannel` from the `fluxion-test-utils` crate instead.
///
/// # Design Philosophy
///
/// - **Production code**: Uses `FluxionStream` for composable, immutable stream transformations
/// - **Test code**: Uses `TestChannel` which wraps this and adds push capabilities
///
/// This separation solves the fundamental conflict between:
/// - Consuming operations (stream extensions that take `self`)
/// - Mutation operations (push that needs `&self`)
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

    /// Creates a `FluxionStream` from any existing stream.
    ///
    /// Use this when you have a stream from another library or source and want
    /// to apply fluxion's extension methods.
    ///
    /// This is just an alias for `FluxionStream::new()` but may be more discoverable.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fluxion_stream::FluxionStream;
    /// use futures::stream;
    ///
    /// let existing_stream = stream::iter(vec![1, 2, 3]);
    /// let stream = FluxionStream::from_stream(existing_stream);
    /// ```
    pub fn from_stream(stream: S) -> Self {
        FluxionStream::new(stream)
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
    ) -> FluxionStream<tokio_stream::wrappers::UnboundedReceiverStream<T>> {
        FluxionStream::new(tokio_stream::wrappers::UnboundedReceiverStream::new(
            receiver,
        ))
    }
}

impl<T> From<tokio_stream::wrappers::UnboundedReceiverStream<T>>
    for FluxionStream<tokio_stream::wrappers::UnboundedReceiverStream<T>>
{
    fn from(stream: tokio_stream::wrappers::UnboundedReceiverStream<T>) -> Self {
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
    S: Stream<Item = T>,
    T: Ordered<Inner = T> + Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    /// Enables ordered stream operations on items that are self-ordered.
    ///
    /// This method is a convenience for streams where the items implement `Ordered`
    /// with `Inner = Self`. It simply returns the FluxionStream itself, but makes
    /// the code more expressive and enables ordered operations.
    ///
    /// # When to use
    ///
    /// Use this when your domain types directly implement `Ordered` (e.g., they have
    /// a timestamp field) and you want to use ordered stream operations like
    /// `map_ordered`, `filter_ordered`, `combine_latest`, etc.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_stream::FluxionStream;
    /// use fluxion_core::Ordered;
    ///
    /// #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    /// struct Event {
    ///     timestamp: u64,
    ///     data: String,
    /// }
    ///
    /// impl Ordered for Event {
    ///     type Inner = Event;
    ///     fn order(&self) -> u64 { self.timestamp }
    ///     fn get(&self) -> &Self::Inner { self }
    ///     fn with_order(value: Self::Inner, _order: u64) -> Self { value }
    /// }
    ///
    /// # async fn example() {
    /// let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    /// 
    /// // Enable ordered operations on self-ordered events
    /// let stream = FluxionStream::from_unbounded_receiver(rx)
    ///     .auto_ordered()
    ///     .map_ordered(|event| event.data);
    /// # }
    /// ```
    pub fn auto_ordered(self) -> Self {
        self
    }
}

impl<S, T> FluxionStream<S>
where
    S: Stream<Item = T>,
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
    /// use fluxion_stream::{FluxionStream, CombineWithPreviousExt, Ordered};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    /// use tokio::sync::mpsc;
    /// use tokio_stream::wrappers::UnboundedReceiverStream;
    ///
    /// async fn example() {
    /// let (tx, rx) = mpsc::unbounded_channel::<Sequenced<i32>>();
    /// let stream = UnboundedReceiverStream::new(rx);
    ///
    /// // Transform ordered stream to strings
    /// let mut mapped = stream
    ///     .combine_with_previous()
    ///     .map_ordered(|item| {
    ///         format!("Value: {}", item.current.get())
    ///     });
    ///
    /// tx.send(Sequenced::new(42)).unwrap();
    /// assert_eq!(mapped.next().await.unwrap(), "Value: 42");
    /// }
    /// ```
    ///
    /// # See Also
    ///
    /// - [`filter_ordered`](FluxionStream::filter_ordered) - Filter items while preserving order
    /// - [`combine_with_previous`](crate::CombineWithPreviousExt::combine_with_previous) - Often used before mapping
    ///
    /// [`StreamExt`]: futures::StreamExt
    pub fn map_ordered<U, F>(self, f: F) -> FluxionStream<impl Stream<Item = U> + Send + Sync>
    where
        S: Send + Sync + Unpin + 'static,
        U: Send + 'static,
        F: FnMut(T) -> U + Send + Sync + 'static,
    {
        let inner = self.into_inner();
        FluxionStream::new(inner.map(f))
    }

    /// Filters items based on a predicate while preserving temporal ordering.
    ///
    /// This operator filters the stream using the provided predicate function.
    /// Unlike the standard `filter` operator from [`StreamExt`], `filter_ordered` maintains
    /// the ordering guarantees of the wrapped stream, ensuring that items passing the
    /// filter are emitted in temporal order.
    ///
    /// # Why use `filter_ordered` instead of `StreamExt::filter`?
    ///
    /// The standard `filter` operator breaks the `FluxionStream` wrapper, losing temporal
    /// ordering guarantees. Using `filter_ordered` preserves the ordering contract needed
    /// for correct operator chaining with other Fluxion operators.
    ///
    /// # Arguments
    ///
    /// * `predicate` - A function that returns `true` for items to keep, `false` to filter out.
    ///   The predicate receives a reference to the inner value of type `T::Inner`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fluxion_stream::FluxionStream;
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    /// use tokio::sync::mpsc;
    /// use tokio_stream::wrappers::UnboundedReceiverStream;
    ///
    /// async fn example() {
    /// let (tx, rx) = mpsc::unbounded_channel();
    /// let stream = UnboundedReceiverStream::new(rx);
    ///
    /// // Filter for even numbers only
    /// let mut filtered = FluxionStream::new(stream)
    ///     .filter_ordered(|n| n % 2 == 0);
    ///
    /// tx.send(Sequenced::new(1)).unwrap();
    /// tx.send(Sequenced::new(2)).unwrap();
    /// tx.send(Sequenced::new(3)).unwrap();
    /// tx.send(Sequenced::new(4)).unwrap();
    ///
    /// assert_eq!(filtered.next().await.unwrap().get(), &2);
    /// assert_eq!(filtered.next().await.unwrap().get(), &4);
    /// }
    /// ```
    ///
    /// # See Also
    ///
    /// - [`map_ordered`](FluxionStream::map_ordered) - Transform items while preserving order
    /// - [`take_while_with`](crate::TakeWhileExt::take_while_with) - Conditional filtering with external stream
    ///
    /// [`StreamExt`]: futures::StreamExt
    pub fn filter_ordered<F>(
        self,
        mut predicate: F,
    ) -> FluxionStream<impl Stream<Item = T> + Send + Sync>
    where
        S: Send + Sync + Unpin + 'static,
        F: FnMut(&T::Inner) -> bool + Send + Sync + 'static,
    {
        let inner = self.into_inner();
        FluxionStream::new(inner.filter(move |item| {
            let inner = item.get();
            let result = predicate(inner);
            futures::future::ready(result)
        }))
    }

    pub fn combine_with_previous(
        self,
    ) -> FluxionStream<impl Stream<Item = WithPrevious<T>> + Send + Sync>
    where
        S: Send + Sync + Unpin + 'static,
    {
        let inner = self.into_inner();
        FluxionStream::new(CombineWithPreviousExt::combine_with_previous(inner))
    }

    pub fn take_while_with<TFilter, SF>(
        self,
        filter_stream: SF,
        filter: impl Fn(&TFilter::Inner) -> bool + Send + Sync + 'static,
    ) -> FluxionStream<impl Stream<Item = T::Inner> + Send + Sync>
    where
        S: Stream<Item = T> + Send + Sync + Unpin + 'static,
        TFilter: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + 'static,
        TFilter::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
        SF: Stream<Item = TFilter> + Send + Sync + 'static,
    {
        let inner = self.into_inner();
        FluxionStream::new(TakeWhileExt::take_while_with(inner, filter_stream, filter))
    }

    pub fn take_latest_when<SF>(
        self,
        filter_stream: SF,
        filter: impl Fn(&T::Inner) -> bool + Send + Sync + 'static,
    ) -> FluxionStream<Pin<Box<dyn Stream<Item = T> + Send + Sync>>>
    where
        S: Stream<Item = T> + Send + Sync + 'static,
        SF: Stream<Item = T> + Send + Sync + 'static,
    {
        let inner = self.into_inner();
        FluxionStream::new(TakeLatestWhenExt::take_latest_when(
            inner,
            filter_stream,
            filter,
        ))
    }

    pub fn emit_when<SF>(
        self,
        filter_stream: SF,
        filter: impl Fn(&CombinedState<T::Inner>) -> bool + Send + Sync + 'static,
    ) -> FluxionStream<Pin<Box<dyn Stream<Item = T> + Send + Sync>>>
    where
        S: Stream<Item = T> + Send + Sync + 'static,
        SF: Stream<Item = T> + Send + Sync + 'static,
    {
        let inner = self.into_inner();
        FluxionStream::new(EmitWhenExt::emit_when(inner, filter_stream, filter))
    }

    pub fn with_latest_from<S2, R>(
        self,
        other: S2,
        result_selector: impl Fn(&CombinedState<T::Inner>) -> R + Send + Sync + 'static,
    ) -> FluxionStream<impl Stream<Item = OrderedWrapper<R>> + Send + Sync>
    where
        S: Stream<Item = T> + Send + Sync + Unpin + 'static,
        S2: Stream<Item = T> + Send + Sync + 'static,
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

    pub fn combine_latest<S2>(
        self,
        others: Vec<S2>,
        filter: impl Fn(&CombinedState<T::Inner>) -> bool + Send + Sync + 'static,
    ) -> FluxionStream<impl Stream<Item = OrderedWrapper<CombinedState<T::Inner>>> + Send>
    where
        S: Stream<Item = T> + Send + Sync + 'static,
        S2: Stream<Item = T> + Send + Sync + 'static,
        T: CompareByInner,
    {
        let inner = self.into_inner();
        FluxionStream::new(CombineLatestExt::combine_latest(inner, others, filter))
    }

    pub fn ordered_merge<S2>(
        self,
        others: Vec<FluxionStream<S2>>,
    ) -> FluxionStream<impl Stream<Item = T> + Send + Sync>
    where
        S: Stream<Item = T> + Send + Sync + 'static,
        S2: Stream<Item = T> + Send + Sync + 'static,
    {
        let inner = self.into_inner();
        let other_streams: Vec<S2> = others.into_iter().map(|fs| fs.into_inner()).collect();
        FluxionStream::new(OrderedStreamExt::ordered_merge(inner, other_streams))
    }
}
