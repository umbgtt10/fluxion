// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Generic implementation of partition operator - runtime-agnostic

use alloc::boxed::Box;
use alloc::sync::Arc;
use core::fmt::Debug;
use core::pin::Pin;
use core::task::{Context, Poll};
use fluxion_core::FluxionTask;
use fluxion_core::{Fluxion, FluxionSubject, StreamItem};
use futures::future::{select, Either};
use futures::{Stream, StreamExt};

/// Generic implementation of partition - works with any runtime that supports FluxionTask
pub fn partition_impl<S, T, F>(
    source: S,
    predicate: F,
) -> (PartitionedStream<T>, PartitionedStream<T>)
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
    S: Stream<Item = StreamItem<T>> + Send + Unpin + 'static,
    F: Fn(&T::Inner) -> bool + Send + Sync + 'static,
{
    let true_subject = FluxionSubject::<T>::new();
    let false_subject = FluxionSubject::<T>::new();

    // Subscribe before spawning to ensure no items are missed
    let true_stream = true_subject
        .subscribe()
        .unwrap_or_else(|_| unreachable!("fresh subject should allow subscription"));
    let false_stream = false_subject
        .subscribe()
        .unwrap_or_else(|_| unreachable!("fresh subject should allow subscription"));

    let task = FluxionTask::spawn(|cancel| async move {
        let mut stream = source;
        while let Either::Left((stream_item, _)) = select(stream.next(), cancel.cancelled()).await {
            match stream_item {
                Some(StreamItem::Value(ref value)) => {
                    let inner = value.clone().into_inner();
                    if predicate(&inner) {
                        let _ = true_subject.next(value.clone());
                    } else {
                        let _ = false_subject.next(value.clone());
                    }
                }
                Some(StreamItem::Error(e)) => {
                    let _ = true_subject.error(e.clone());
                    let _ = false_subject.error(e);
                    break;
                }
                None => {
                    break;
                }
            }
        }
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

/// A partitioned stream that keeps the routing task alive.
pub struct PartitionedStream<T: Fluxion>
where
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    inner: Pin<Box<dyn Stream<Item = StreamItem<T>> + Send + Sync + 'static>>,
    _guard: Arc<TaskGuard>,
}

impl<T: Fluxion> Debug for PartitionedStream<T>
where
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PartitionedStream")
            .field("inner", &"<stream>")
            .finish()
    }
}

impl<T: Fluxion> Stream for PartitionedStream<T>
where
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    type Item = StreamItem<T>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

#[derive(Debug)]
struct TaskGuard {
    task: FluxionTask,
}

impl Drop for TaskGuard {
    fn drop(&mut self) {
        self.task.cancel();
    }
}
