use futures::{Stream, StreamExt, stream::FuturesUnordered};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::time::Instant;

pub struct SelectAll<T> {
    futures: FuturesUnordered<StreamFutureWrapper<T>>, // Dynamically managed streams
    buffer: BinaryHeap<Reverse<OrderedItem<T>>>,       // Priority queue for ordered items
}

/// Wrapper to convert each stream into a `Future` that produces `(stream_id, item)`.
pub struct StreamFutureWrapper<T> {
    stream: Pin<Box<dyn Stream<Item = T> + Send>>, // The stream being managed
    stream_id: usize,                              // Unique ID for the stream
}

impl<T> StreamFutureWrapper<T> {
    pub fn new<S>(stream: S, id: usize) -> Self
    where
        S: Stream<Item = T> + Send + 'static,
    {
        Self {
            stream: Box::pin(stream),
            stream_id: id,
        }
    }
}

impl<T> std::future::Future for StreamFutureWrapper<T> {
    type Output = Option<(usize, T)>; // `(stream_id, item)`

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = Pin::into_inner(self);
        match this.stream.as_mut().poll_next(cx) {
            Poll::Ready(Some(item)) => Poll::Ready(Some((this.stream_id, item))),
            Poll::Ready(None) => Poll::Ready(None), // The stream is finished
            Poll::Pending => Poll::Pending,
        }
    }
}

/// An item that carries ordering metadata (timestamp + value).
#[derive(Debug, Clone, PartialEq, Eq)]
struct OrderedItem<T> {
    timestamp: Instant, // Timestamp when the item was sent
    value: T,           // The actual value
}

// Ordering logic for `OrderedItem`, prioritizing smaller timestamps.
impl<T: PartialOrd> PartialOrd for OrderedItem<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(other.timestamp.cmp(&self.timestamp).reverse())
    }
}

impl<T: Ord> Ord for OrderedItem<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.timestamp.cmp(&self.timestamp).reverse()
    }
}

impl<T: Ord> SelectAll<T>
where
    T: Send + 'static,
{
    /// Constructs a new `SelectAll` stream from a vector of streams.
    pub fn new<S>(streams: Vec<S>) -> Self
    where
        S: Stream<Item = T> + Send + 'static,
    {
        let futures = streams
            .into_iter()
            .enumerate()
            .map(|(id, stream)| StreamFutureWrapper::new(stream, id)) // Wrap each stream
            .collect::<FuturesUnordered<_>>();

        Self {
            futures,
            buffer: BinaryHeap::new(), // Initialize an empty priority queue
        }
    }
}

impl<T: Ord> Stream for SelectAll<T>
where
    T: Send + Unpin + 'static,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = Pin::into_inner(self);

        // Poll all managed streams to collect ready items
        while let Poll::Ready(Some(Some((_stream_id, item)))) = this.futures.poll_next_unpin(cx) {
            // Assign a timestamp to each polled item and push it into the priority queue
            this.buffer.push(Reverse(OrderedItem {
                timestamp: Instant::now(), // Use current time for ordering
                value: item,
            }));
        }

        // Return the earliest item based on the priority queue
        if let Some(Reverse(ordered_item)) = this.buffer.pop() {
            Poll::Ready(Some(ordered_item.value))
        } else if this.futures.is_empty() && this.buffer.is_empty() {
            Poll::Ready(None) // All streams are exhausted
        } else {
            Poll::Pending // Wait for more items
        }
    }
}

/// Extension trait for combining streams into a single `SelectAll`.
pub trait SelectAllExt {
    type Item;

    fn select_all_ordered(self) -> SelectAll<Self::Item>;
}

impl<T, S> SelectAllExt for Vec<S>
where
    S: Stream<Item = T> + Send + 'static,
    T: Send + Ord + 'static,
{
    type Item = T;

    fn select_all_ordered(self) -> SelectAll<Self::Item> {
        SelectAll::new(self)
    }
}
