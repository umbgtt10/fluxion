use futures::Stream;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct OrderedMerge<T> {
    streams: Vec<Pin<Box<dyn Stream<Item = T> + Send>>>,
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

pub trait OrderedMergeExt {
    type Item;

    fn ordered_merge(self) -> OrderedMerge<Self::Item>;
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
