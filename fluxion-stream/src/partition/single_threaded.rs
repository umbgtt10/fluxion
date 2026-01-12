// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::implementation::TaskGuard;
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::fmt::Debug;
use core::pin::Pin;
use core::task::{Context, Poll};
use fluxion_core::{Fluxion, FluxionSubject, FluxionTask, StreamItem};
use futures::future::{select, Either};
use futures::{Stream, StreamExt};

/// A partitioned stream that keeps the routing task alive.
///
/// This stream wraps an inner stream and holds an `Arc` reference to the
/// routing task guard. The task remains alive as long as either partitioned
/// stream exists. When both streams are dropped, the task is aborted.
///
/// Implements `Stream` by delegating to the inner stream.
pub struct PartitionedStream<T: Fluxion>
where
    T::Inner: Clone + Debug + Ord + Unpin + 'static,
    T::Timestamp: Debug + Ord + Copy + 'static,
{
    inner: InnerStream<T>,
    _guard: Arc<TaskGuard>,
}

impl<T: Fluxion> Debug for PartitionedStream<T>
where
    T::Inner: Clone + Debug + Ord + Unpin + 'static,
    T::Timestamp: Debug + Ord + Copy + 'static,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PartitionedStream")
            .field("inner", &"<stream>")
            .finish()
    }
}

impl<T> Stream for PartitionedStream<T>
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Unpin + 'static,
    T::Timestamp: Debug + Ord + Copy + 'static,
{
    type Item = StreamItem<T>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

type InnerStream<T> = Pin<Box<dyn Stream<Item = StreamItem<T>> + 'static>>;

/// Extension trait providing the `partition` operator for streams.
///
/// This trait allows any stream of `StreamItem<T>` to be partitioned into two
/// streams based on a predicate function.
pub trait PartitionExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Unpin + 'static,
    T::Timestamp: Debug + Ord + Copy + 'static,
{
    /// Partitions the stream into two based on a predicate.
    ///
    /// See the [module-level documentation](crate::partition) for detailed examples and usage patterns.
    fn partition<F>(self, predicate: F) -> (PartitionedStream<T>, PartitionedStream<T>)
    where
        Self: Unpin + 'static,
        F: Fn(&T::Inner) -> bool + 'static;
}

impl<S, T> PartitionExt<T> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Unpin + 'static,
    T::Timestamp: Debug + Ord + Copy + 'static,
{
    fn partition<F>(self, predicate: F) -> (PartitionedStream<T>, PartitionedStream<T>)
    where
        Self: Unpin + 'static,
        F: Fn(&T::Inner) -> bool + 'static,
    {
        let true_subject = FluxionSubject::<T>::new();
        let false_subject = FluxionSubject::<T>::new();

        let true_stream = true_subject
            .subscribe()
            .unwrap_or_else(|_| unreachable!("fresh subject should allow subscription"));
        let false_stream = false_subject
            .subscribe()
            .unwrap_or_else(|_| unreachable!("fresh subject should allow subscription"));

        let task = FluxionTask::spawn(|cancel| async move {
            let mut stream = self;
            while let Either::Left((stream_item, _)) =
                select(stream.next(), cancel.cancelled()).await
            {
                match stream_item {
                    Some(StreamItem::Value(value)) => {
                        let inner = value.clone().into_inner();
                        if predicate(&inner) {
                            if true_subject.next(value).is_err() {
                                // True subscriber dropped, but continue for false subscriber
                            }
                        } else if false_subject.next(value).is_err() {
                            // False subscriber dropped, but continue for true subscriber
                        }
                    }
                    Some(StreamItem::Error(e)) => {
                        // Propagate error to both streams
                        let _ = true_subject.error(e.clone());
                        let _ = false_subject.error(e);
                        break;
                    }
                    None => {
                        // Source completed
                        break;
                    }
                }
            }
            // Close both subjects on exit
            true_subject.close();
            false_subject.close();
        });

        let guard = Arc::new(TaskGuard { task });

        (
            PartitionedStream {
                inner: Box::pin(true_stream),
                _guard: guard.clone(),
            },
            PartitionedStream {
                inner: Box::pin(false_stream),
                _guard: guard,
            },
        )
    }
}
