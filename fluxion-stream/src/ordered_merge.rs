use futures::Stream;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fmt::Debug;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::sequenced::Sequenced;
use crate::sequenced_stream::SequencedStreamExt;

/// Internal low-level ordered merge for non-Sequenced streams
/// Used by other operators internally
pub struct OrderedMerge<T> {
    streams: Vec<Pin<Box<dyn Stream<Item = T> + Send>>>,
    buffer: BinaryHeap<Reverse<T>>,
}

pub struct OrderedMergeSync<T> {
    streams: Vec<Pin<Box<dyn Stream<Item = T> + Send + Sync>>>,
    buffer: BinaryHeap<Reverse<T>>,
}

impl<T> OrderedMerge<T>
where
    T: Send + Ord + 'static,
{
    pub fn new<S>(streams: Vec<S>) -> Self
    where
        S: Stream<Item = T> + Send + 'static,
    {
        let streams = streams
            .into_iter()
            .map(|stream| Box::pin(stream) as Pin<Box<dyn Stream<Item = T> + Send>>)
            .collect::<Vec<_>>();

        Self {
            streams,
            buffer: BinaryHeap::new(),
        }
    }
}

impl<T> OrderedMergeSync<T>
where
    T: Send + Ord + 'static,
{
    pub fn new<S>(streams: Vec<S>) -> Self
    where
        S: Stream<Item = T> + Send + Sync + 'static,
    {
        let streams = streams
            .into_iter()
            .map(|stream| Box::pin(stream) as Pin<Box<dyn Stream<Item = T> + Send + Sync>>)
            .collect::<Vec<_>>();

        Self {
            streams,
            buffer: BinaryHeap::new(),
        }
    }
}

impl<T> Stream for OrderedMerge<T>
where
    T: Send + Unpin + Ord + 'static,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = Pin::into_inner(self);

        if let Some(Reverse(item)) = this.buffer.pop() {
            return Poll::Ready(Some(item));
        }

        let mut all_done = true;

        for stream in this.streams.iter_mut() {
            match stream.as_mut().poll_next(cx) {
                Poll::Ready(Some(item)) => {
                    this.buffer.push(Reverse(item));
                    all_done = false;
                }
                Poll::Ready(None) => {}
                Poll::Pending => {
                    all_done = false;
                }
            }
        }

        if let Some(Reverse(item)) = this.buffer.pop() {
            Poll::Ready(Some(item))
        } else if all_done {
            Poll::Ready(None)
        } else {
            Poll::Pending
        }
    }
}

impl<T> Stream for OrderedMergeSync<T>
where
    T: Send + Unpin + Ord + 'static,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = Pin::into_inner(self);

        if let Some(Reverse(item)) = this.buffer.pop() {
            return Poll::Ready(Some(item));
        }

        let mut all_done = true;

        for stream in this.streams.iter_mut() {
            match stream.as_mut().poll_next(cx) {
                Poll::Ready(Some(item)) => {
                    this.buffer.push(Reverse(item));
                    all_done = false;
                }
                Poll::Ready(None) => {}
                Poll::Pending => {
                    all_done = false;
                }
            }
        }

        if let Some(Reverse(item)) = this.buffer.pop() {
            Poll::Ready(Some(item))
        } else if all_done {
            Poll::Ready(None)
        } else {
            Poll::Pending
        }
    }
}

pub trait OrderedMergeExt {
    type Item;

    fn ordered_merge(self) -> OrderedMerge<Self::Item>;
}

pub trait OrderedMergeSyncExt {
    type Item;

    fn ordered_merge_sync(self) -> OrderedMergeSync<Self::Item>;
}

impl<T, S> OrderedMergeExt for Vec<S>
where
    S: Stream<Item = T> + Send + 'static,
    T: Send + Ord + 'static,
{
    type Item = T;

    fn ordered_merge(self) -> OrderedMerge<Self::Item> {
        OrderedMerge::new(self)
    }
}

impl<T, S> OrderedMergeSyncExt for Vec<S>
where
    S: Stream<Item = T> + Send + Sync + 'static,
    T: Send + Ord + 'static,
{
    type Item = T;

    fn ordered_merge_sync(self) -> OrderedMergeSync<Self::Item> {
        OrderedMergeSync::new(self)
    }
}

/// High-level ordered merge for Sequenced streams
/// This is the Fluxion extension trait that merges multiple Sequenced streams
/// and emits all values in sequence order
pub trait OrderedMergeSequencedExt<T, S2>: SequencedStreamExt<T> + Sized
where
    T: Clone + Debug + Send + Sync + Unpin + 'static,
    Sequenced<T>: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    S2: Stream<Item = Sequenced<T>> + Send + 'static,
{
    /// Merges multiple Sequenced streams, emitting all values in sequence order.
    /// Unlike combine_latest, this doesn't wait for all streams - it emits every value
    /// from all streams in order by sequence number.
    ///
    /// # Arguments
    /// * `others` - Vector of other streams to merge with this stream
    ///
    /// # Returns
    /// Stream of `Sequenced<T>` where values are emitted in sequence order
    fn ordered_merge(self, others: Vec<S2>) -> impl Stream<Item = Sequenced<T>> + Send;
}

impl<T, S, S2> OrderedMergeSequencedExt<T, S2> for S
where
    T: Clone + Debug + Send + Sync + Unpin + 'static,
    Sequenced<T>: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    S: SequencedStreamExt<T> + Send + 'static,
    S2: Stream<Item = Sequenced<T>> + Send + 'static,
{
    fn ordered_merge(self, others: Vec<S2>) -> impl Stream<Item = Sequenced<T>> + Send {
        let mut all_streams =
            vec![Box::pin(self) as Pin<Box<dyn Stream<Item = Sequenced<T>> + Send>>];
        for stream in others {
            all_streams.push(Box::pin(stream));
        }

        OrderedMerge::new(all_streams)
    }
}
