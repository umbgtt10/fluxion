use futures::Stream;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct SelectAll<T> {
    streams: Vec<Pin<Box<dyn Stream<Item = T> + Send>>>, // Collection of streams
    buffer: BinaryHeap<Reverse<T>>,                      // Min-heap for ordered items
}

impl<T> SelectAll<T>
where
    T: Send + Ord + 'static,
{
    /// Constructs a new `SelectAll` stream from a vector of streams.
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

impl<T> Stream for SelectAll<T>
where
    T: Send + Unpin + Ord + 'static,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = Pin::into_inner(self);

        // If we have buffered items, return them first (in order)
        if let Some(Reverse(item)) = this.buffer.pop() {
            return Poll::Ready(Some(item));
        }

        // Poll all streams once and collect ready items
        let mut all_done = true;

        for stream in this.streams.iter_mut() {
            match stream.as_mut().poll_next(cx) {
                Poll::Ready(Some(item)) => {
                    this.buffer.push(Reverse(item));
                    all_done = false;
                }
                Poll::Ready(None) => {
                    // Stream is done
                }
                Poll::Pending => {
                    all_done = false;
                }
            }
        }

        // Return the first buffered item if we got any (smallest by Ord)
        if let Some(Reverse(item)) = this.buffer.pop() {
            Poll::Ready(Some(item))
        } else if all_done {
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
