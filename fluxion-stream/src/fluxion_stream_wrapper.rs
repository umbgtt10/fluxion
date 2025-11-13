use crate::combine_latest::{CombineLatestExt, CombinedState, CompareByInner};
use crate::combine_with_previous::CombineWithPreviousExt;
use crate::ordered_merge::OrderedMergeSequencedExt;
use crate::sequenced::Sequenced;
use crate::sequenced_stream::SequencedStreamExt;
use crate::take_latest_when::TakeLatestWhenExt;
use crate::take_while_with::TakeWhileExt;
use crate::with_latest_from::WithLatestFromExt;
use futures::Stream;
use pin_project::pin_project;
use std::fmt::Debug;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A concrete wrapper type that provides all fluxion stream extensions.
///
/// This type wraps any stream of `Sequenced<T>` and provides all the fluxion
/// extension methods directly, allowing easy chaining and composition.
#[pin_project]
pub struct FluxionStream<S> {
    #[pin]
    inner: S,
}

impl<S> FluxionStream<S> {
    /// Wrap a stream in a FluxionStream wrapper
    pub fn new(stream: S) -> Self {
        FluxionStream { inner: stream }
    }

    /// Unwrap to get the inner stream
    pub fn into_inner(self) -> S {
        self.inner
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
    S: Stream<Item = Sequenced<T>>,
    T: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    /// Combines each element with the previous element, tracking state changes
    pub fn combine_with_previous(
        self,
    ) -> FluxionStream<impl Stream<Item = (Option<Sequenced<T>>, Sequenced<T>)>>
    where
        S: Send + Sync + Unpin + 'static,
    {
        let inner = self.into_inner();
        FluxionStream::new(CombineWithPreviousExt::combine_with_previous(inner))
    }

    /// Takes elements while a condition on the filter stream is satisfied
    pub fn take_while_with<TF, SF>(
        self,
        filter_stream: SF,
        filter: impl Fn(&TF) -> bool + Send + Sync + 'static,
    ) -> FluxionStream<impl Stream<Item = T>>
    where
        S: SequencedStreamExt<T> + Send + Sync + Unpin + 'static,
        TF: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
        SF: Stream<Item = Sequenced<TF>> + Send + Sync + 'static,
    {
        let inner = self.into_inner();
        TakeWhileExt::take_while_with(inner, filter_stream, filter)
    }

    /// Takes the latest value from source when filter predicate is satisfied
    pub fn take_latest_when<SF>(
        self,
        filter_stream: SF,
        filter: impl Fn(&CombinedState<T>) -> bool + Send + Sync + 'static,
    ) -> FluxionStream<Pin<Box<dyn Stream<Item = Sequenced<T>> + Send + Sync>>>
    where
        S: SequencedStreamExt<T> + Send + Sync + 'static,
        SF: Stream<Item = Sequenced<T>> + Send + Sync + 'static,
    {
        let inner = self.into_inner();
        FluxionStream::new(TakeLatestWhenExt::take_latest_when(
            inner,
            filter_stream,
            filter,
        ))
    }

    /// Combines this stream with another, emitting when the primary stream emits
    pub fn with_latest_from<S2>(
        self,
        other: S2,
        filter: impl Fn(&CombinedState<T>) -> bool + Send + Sync + 'static,
    ) -> impl Stream<Item = (Sequenced<T>, Sequenced<T>)> + Send
    where
        S: SequencedStreamExt<T> + Send + Unpin + 'static,
        S2: Stream<Item = Sequenced<T>> + Send + 'static,
        Sequenced<T>: CompareByInner,
    {
        let inner = self.into_inner();
        WithLatestFromExt::with_latest_from(inner, other, filter)
    }

    /// Combines this stream with multiple others using combine_latest semantics
    pub fn combine_latest<S2>(
        self,
        others: Vec<S2>,
        filter: impl Fn(&CombinedState<T>) -> bool + Send + Sync + 'static,
    ) -> FluxionStream<impl Stream<Item = Sequenced<CombinedState<T>>> + Send>
    where
        S: SequencedStreamExt<T> + Send + 'static,
        S2: Stream<Item = Sequenced<T>> + Send + 'static,
        Sequenced<T>: CompareByInner,
    {
        let inner = self.into_inner();
        FluxionStream::new(CombineLatestExt::combine_latest(inner, others, filter))
    }

    /// Merges this stream with multiple others, emitting all values in sequence order.
    /// Unlike combine_latest, this doesn't wait for all streams - it emits every value
    /// from all streams individually in sequence order.
    pub fn ordered_merge<S2>(
        self,
        others: Vec<S2>,
    ) -> FluxionStream<impl Stream<Item = Sequenced<T>> + Send>
    where
        S: SequencedStreamExt<T> + Send + 'static,
        S2: Stream<Item = Sequenced<T>> + Send + 'static,
    {
        let inner = self.into_inner();
        FluxionStream::new(OrderedMergeSequencedExt::ordered_merge(inner, others))
    }
}

// Ensure Send and Sync if the inner stream is Send
unsafe impl<S: Send> Send for FluxionStream<S> {}
unsafe impl<S: Send + Sync> Sync for FluxionStream<S> {}
