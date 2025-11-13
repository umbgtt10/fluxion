use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Low-level ordered merge that combines multiple streams emitting items in order.
/// Items are emitted based on their `Ord` implementation - smallest items first.
///
/// Streams must be `Send + Sync` to ensure safe concurrent access.
pub struct OrderedMerge<T> {
    streams: Vec<Pin<Box<dyn Stream<Item = T> + Send + Sync>>>,
    buffered: Vec<Option<T>>,
}

impl<T> OrderedMerge<T>
where
    T: Send + Ord + 'static,
{
    #[must_use]
    pub fn new<S>(streams: Vec<S>) -> Self
    where
        S: Stream<Item = T> + Send + Sync + 'static,
    {
        let count = streams.len();
        let streams = streams
            .into_iter()
            .map(|stream| Box::pin(stream) as Pin<Box<dyn Stream<Item = T> + Send + Sync>>)
            .collect::<Vec<_>>();

        let buffered = (0..count).map(|_| None).collect();

        Self { streams, buffered }
    }
}

impl<T> Stream for OrderedMerge<T>
where
    T: Send + Unpin + Ord + 'static,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = Pin::into_inner(self);

        // Poll streams to fill empty buffer slots
        let mut any_pending = false;

        for i in 0..this.streams.len() {
            if this.buffered[i].is_none() {
                match this.streams[i].as_mut().poll_next(cx) {
                    Poll::Ready(Some(item)) => {
                        this.buffered[i] = Some(item);
                    }
                    Poll::Ready(None) => {
                        // Stream is done, leave as None
                    }
                    Poll::Pending => {
                        any_pending = true;
                    }
                }
            }
        }

        // Find the minimum item among all buffered items
        let mut min_idx = None;
        let mut min_val: Option<&T> = None;

        for (i, item) in this.buffered.iter().enumerate() {
            if let Some(val) = item {
                let should_update = min_val.is_none_or(|curr_val| val < curr_val);

                if should_update {
                    min_idx = Some(i);
                    min_val = Some(val);
                }
            }
        } // Return the minimum item if found
        if let Some(idx) = min_idx {
            let item = this.buffered[idx].take().unwrap();
            Poll::Ready(Some(item))
        } else if any_pending {
            Poll::Pending
        } else {
            Poll::Ready(None)
        }
    }
}

/// Extension trait for merging a vector of streams in order.
pub trait OrderedMergeExt {
    type Item;

    /// Merges multiple streams, emitting items in order based on their `Ord` implementation.
    fn ordered_merge(self) -> OrderedMerge<Self::Item>;
}

impl<T, S> OrderedMergeExt for Vec<S>
where
    S: Stream<Item = T> + Send + Sync + 'static,
    T: Send + Ord + 'static,
{
    type Item = T;

    fn ordered_merge(self) -> OrderedMerge<Self::Item> {
        OrderedMerge::new(self)
    }
}
