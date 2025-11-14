// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::Ordered;
use crate::combine_latest::{CombineLatestExt, CombinedState};
use crate::combine_with_previous::CombineWithPreviousExt;
use crate::emit_when::EmitWhenExt;
use crate::ordered_merge::OrderedStreamExt;
use crate::take_latest_when::TakeLatestWhenExt;
use crate::take_while_with::TakeWhileExt;
use crate::with_latest_from::WithLatestFromExt;
use fluxion_core::CompareByInner;
use futures::Stream;
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
    /// ```rust,ignore
    /// use fluxion::FluxionStream;
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
    /// ```rust,ignore
    /// use fluxion::FluxionStream;
    /// use tokio::sync::mpsc;
    ///
    /// let (tx, rx) = mpsc::unbounded_channel();
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

// Extension methods directly on FluxionStream
impl<S, T> FluxionStream<S>
where
    S: Stream<Item = T>,
    T: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    /// Combines each element with the previous element, tracking state changes
    pub fn combine_with_previous(self) -> FluxionStream<impl Stream<Item = (Option<T>, T)>>
    where
        S: Send + Sync + Unpin + 'static,
    {
        let inner = self.into_inner();
        FluxionStream::new(CombineWithPreviousExt::combine_with_previous(inner))
    }

    /// Takes elements while a condition on the filter stream is satisfied
    pub fn take_while_with<TFilter, SF>(
        self,
        filter_stream: SF,
        filter: impl Fn(&TFilter::Inner) -> bool + Send + Sync + 'static,
    ) -> FluxionStream<impl Stream<Item = T::Inner>>
    where
        S: Stream<Item = T> + Send + Sync + Unpin + 'static,
        TFilter: Ordered + Clone + Debug + Ord + Send + Sync + Unpin + 'static,
        TFilter::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
        SF: Stream<Item = TFilter> + Send + Sync + 'static,
    {
        let inner = self.into_inner();
        TakeWhileExt::take_while_with(inner, filter_stream, filter)
    }

    /// Takes the latest value from source when filter predicate is satisfied
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

    /// Emits the latest value from source when filter predicate is satisfied (original behavior)
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
        FluxionStream::new(EmitWhenExt::emit_when(
            inner,
            filter_stream,
            filter,
        ))
    }

    /// Combines this stream with another, emitting when the primary stream emits
    pub fn with_latest_from<S2>(
        self,
        other: S2,
        filter: impl Fn(&CombinedState<T::Inner>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = (T, T)> + Send
    where
        S: Stream<Item = T> + Send + Sync + Unpin + 'static,
        S2: Stream<Item = T> + Send + Sync + 'static,
        T: CompareByInner,
    {
        let inner = self.into_inner();
        WithLatestFromExt::with_latest_from(inner, other, filter)
    }

    /// Combines this stream with multiple others using `combine_latest` semantics
    pub fn combine_latest<S2>(
        self,
        others: Vec<S2>,
        filter: impl Fn(&CombinedState<T::Inner>) -> bool + Send + Sync + 'static,
    ) -> FluxionStream<
        impl Stream<Item = fluxion_core::OrderedWrapper<CombinedState<T::Inner>>> + Send,
    >
    where
        S: Stream<Item = T> + Send + Sync + 'static,
        S2: Stream<Item = T> + Send + Sync + 'static,
        T: CompareByInner,
    {
        let inner = self.into_inner();
        FluxionStream::new(CombineLatestExt::combine_latest(inner, others, filter))
    }

    /// Merges this stream with multiple others, emitting all values in order.
    /// Unlike `combine_latest`, this doesn't wait for all streams - it emits every value
    /// from all streams individually in order.
    pub fn ordered_merge<S2>(
        self,
        others: Vec<S2>,
    ) -> FluxionStream<impl Stream<Item = T> + Send + Sync>
    where
        S: Stream<Item = T> + Send + Sync + 'static,
        S2: Stream<Item = T> + Send + Sync + 'static,
    {
        let inner = self.into_inner();
        FluxionStream::new(OrderedStreamExt::ordered_merge(inner, others))
    }
}
