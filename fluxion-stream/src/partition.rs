// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Partition operator that splits a stream into two based on a predicate.
//!
//! The [`partition`](PartitionExt::partition) operator routes each item to one of two
//! output streams based on a predicate function. Items satisfying the predicate go to
//! the "true" stream, while others go to the "false" stream.
//!
//! ## Characteristics
//!
//! - **Chain-breaking**: Returns two streams, cannot chain further on the original
//! - **Spawns task**: Routing runs in a background Tokio task
//! - **Timestamp-preserving**: Original timestamps are preserved in both output streams
//! - **Routing**: Every item goes to exactly one output stream
//! - **Non-blocking**: Both streams can be consumed independently
//! - **Hot**: Uses internal subjects for broadcasting (late consumers miss items)
//! - **Error propagation**: Errors are sent to both output streams
//! - **Unbounded buffers**: Items are buffered in memory until consumed
//!
//! ## Buffer Behavior
//!
//! The partition operator uses unbounded internal channels. If one partition stream
//! is consumed slowly (or not at all), items destined for that stream will accumulate
//! in memory. This is typically fine for balanced workloads, but be aware:
//!
//! - If you only consume one partition, items for the other still buffer
//! - For high-throughput streams with imbalanced consumption, consider adding
//!   backpressure mechanisms downstream
//! - Dropping one partition stream is safe; items for it are simply discarded
//!
//! ## Example
//!
//! ```rust
//! use fluxion_stream::{IntoFluxionStream, PartitionExt};
//! use fluxion_test_utils::Sequenced;
//! use futures::StreamExt;
//!
//! # async fn example() {
//! let (tx, rx) = futures::channel::mpsc::unbounded();
//!
//! // Partition numbers into even and odd
//! let (mut evens, mut odds) = rx.into_fluxion_stream()
//!     .partition(|n: &i32| n % 2 == 0);
//!
//! tx.unbounded_send(Sequenced::new(1)).unwrap();
//! tx.unbounded_send(Sequenced::new(2)).unwrap();
//! tx.unbounded_send(Sequenced::new(3)).unwrap();
//! tx.unbounded_send(Sequenced::new(4)).unwrap();
//! drop(tx);
//!
//! // Evens: 2, 4
//! assert_eq!(evens.next().await.unwrap().unwrap().into_inner(), 2);
//! assert_eq!(evens.next().await.unwrap().unwrap().into_inner(), 4);
//!
//! // Odds: 1, 3
//! assert_eq!(odds.next().await.unwrap().unwrap().into_inner(), 1);
//! assert_eq!(odds.next().await.unwrap().unwrap().into_inner(), 3);
//! # }
//! ```
//!
//! ## Use Cases
//!
//! - **Error routing**: Separate successful values from validation failures
//! - **Priority queues**: Split high-priority and low-priority items
//! - **Type routing**: Route different enum variants to specialized handlers
//! - **Threshold filtering**: Split values above/below a threshold

use core::fmt::Debug;
use core::pin::Pin;
use core::task::{Context, Poll};
use fluxion_core::FluxionTask;
use fluxion_core::{Fluxion, FluxionSubject, StreamItem};
use futures::{FutureExt, Stream, StreamExt};
use std::sync::Arc;

/// Extension trait providing the `partition` operator for streams.
///
/// This trait allows any stream of `StreamItem<T>` to be partitioned into two
/// streams based on a predicate function.
pub trait PartitionExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    /// Partitions the stream into two based on a predicate.
    ///
    /// This operator routes each item to one of two output streams:
    /// - Items where `predicate(&inner)` returns `true` go to the first stream
    /// - Items where `predicate(&inner)` returns `false` go to the second stream
    ///
    /// # Behavior
    ///
    /// - Each item is sent to exactly one output stream
    /// - Errors are propagated to both streams and terminate the partition
    /// - When the source completes, both output streams complete
    /// - The routing task is spawned immediately and runs until source completion
    ///
    /// # Arguments
    ///
    /// * `predicate` - A function that takes `&T::Inner` and returns `true` or `false`
    ///
    /// # Returns
    ///
    /// A tuple of two streams: `(true_stream, false_stream)`
    ///
    /// # Examples
    ///
    /// ## Basic partition by predicate
    ///
    /// ```rust
    /// use fluxion_stream::{IntoFluxionStream, PartitionExt};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// let (tx, rx) = futures::channel::mpsc::unbounded();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// // Partition by even/odd
    /// let (mut evens, mut odds) = stream.partition(|&n| n % 2 == 0);
    ///
    /// tx.unbounded_send(Sequenced::new(1)).unwrap();
    /// tx.unbounded_send(Sequenced::new(2)).unwrap();
    ///
    /// assert_eq!(odds.next().await.unwrap().unwrap().into_inner(), 1);
    /// assert_eq!(evens.next().await.unwrap().unwrap().into_inner(), 2);
    /// # }
    /// ```
    ///
    /// ## Partition with threshold
    ///
    /// ```rust
    /// use fluxion_stream::{IntoFluxionStream, PartitionExt};
    /// use fluxion_test_utils::Sequenced;
    /// use futures::StreamExt;
    ///
    /// # async fn example() {
    /// let (tx, rx) = futures::channel::mpsc::unbounded();
    /// let stream = rx.into_fluxion_stream();
    ///
    /// // Partition high/low priority (threshold = 50)
    /// let (mut high, mut low) = stream.partition(|&n| n >= 50);
    ///
    /// tx.unbounded_send(Sequenced::new(30)).unwrap();  // low
    /// tx.unbounded_send(Sequenced::new(75)).unwrap();  // high
    /// tx.unbounded_send(Sequenced::new(50)).unwrap();  // high (boundary)
    ///
    /// assert_eq!(low.next().await.unwrap().unwrap().into_inner(), 30);
    /// assert_eq!(high.next().await.unwrap().unwrap().into_inner(), 75);
    /// assert_eq!(high.next().await.unwrap().unwrap().into_inner(), 50);
    /// # }
    /// ```
    ///
    /// # Use Cases
    ///
    /// - **Validation routing**: Route valid items to processing, invalid to error handling
    /// - **Load balancing**: Split work between fast and slow processing paths
    /// - **Priority queues**: Separate urgent from non-urgent items
    /// - **A/B routing**: Route items to different handlers for comparison
    ///
    /// # Errors
    ///
    /// Errors from the source stream are propagated to both output streams.
    /// After an error, both streams will complete.
    ///
    /// # See Also
    ///
    /// - [`FilterOrderedExt::filter_ordered`](crate::FilterOrderedExt::filter_ordered) - Keep only matching items
    /// - [`ShareExt::share`](crate::ShareExt::share) - Broadcast to multiple identical subscribers
    fn partition<F>(self, predicate: F) -> (PartitionedStream<T>, PartitionedStream<T>)
    where
        Self: Send + Sync + Unpin + 'static,
        F: Fn(&T::Inner) -> bool + Send + Sync + 'static;
}

/// A partitioned stream that keeps the routing task alive.
///
/// This stream wraps an inner stream and holds an `Arc` reference to the
/// routing task guard. The task remains alive as long as either partitioned
/// stream exists. When both streams are dropped, the task is aborted.
///
/// Implements `Stream` by delegating to the inner stream.
pub struct PartitionedStream<T: Fluxion>
where
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    inner: InnerStream<T>,
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

impl<S, T> PartitionExt<T> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    fn partition<F>(self, predicate: F) -> (PartitionedStream<T>, PartitionedStream<T>)
    where
        Self: Send + Sync + Unpin + 'static,
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
            let mut stream = self;
            loop {
                futures::select! {
                    _ = cancel.cancelled().fuse() => {
                        // Graceful shutdown requested
                        break;
                    }

                    item = stream.next().fuse() => {
                        match item {
                            Some(StreamItem::Value(ref value)) => {
                                let inner = value.clone().into_inner();
                                if predicate(&inner) {
                                    if true_subject.next(value.clone()).is_err() {
                                        // True subscriber dropped, but continue for false subscriber
                                    }
                                } else if false_subject.next(value.clone()).is_err() {
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

type InnerStream<T> = Pin<Box<dyn Stream<Item = StreamItem<T>> + Send + Sync + 'static>>;

#[derive(Debug)]
struct TaskGuard {
    task: FluxionTask,
}

impl Drop for TaskGuard {
    fn drop(&mut self) {
        self.task.cancel();
    }
}
