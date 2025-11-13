use futures::Stream;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Low-level ordered merge that combines multiple streams emitting items in order.
/// Items are emitted based on their `Ord` implementation - smallest items first.
///
/// This is the `Send` variant that boxes streams as `dyn Stream + Send`.
pub struct OrderedMerge<T> {
    streams: Vec<Pin<Box<dyn Stream<Item = T> + Send>>>,
    buffer: BinaryHeap<Reverse<T>>,
}

/// Low-level ordered merge that combines multiple streams emitting items in order.
/// Items are emitted based on their `Ord` implementation - smallest items first.
///
/// This is the `Send + Sync` variant that boxes streams as `dyn Stream + Send + Sync`.
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

// SAFETY: OrderedMerge only accesses streams through pinned references and doesn't
// share mutable state across threads. The streams are polled only through &mut self.
unsafe impl<T> Sync for OrderedMerge<T> where T: Send + Ord + 'static {}

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

/// Extension trait for merging a vector of streams in order.
/// This is the `Send` variant.
pub trait OrderedMergeExt {
    type Item;

    /// Merges multiple streams, emitting items in order based on their `Ord` implementation.
    fn ordered_merge(self) -> OrderedMerge<Self::Item>;
}

/// Extension trait for merging a vector of streams in order.
/// This is the `Send + Sync` variant.
pub trait OrderedMergeSyncExt {
    type Item;

    /// Merges multiple streams, emitting items in order based on their `Ord` implementation.
    /// Returns a `Sync` stream for composition with operators requiring `Sync`.
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
