// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Delay operator for time-based stream processing.

use crate::InstantTimestamped;
use fluxion_core::StreamItem;
use futures::stream::FuturesOrdered;
use futures::{Stream, StreamExt};
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::time::{sleep, Sleep};

/// Extension trait providing the `delay` operator for streams.
///
/// This trait allows any stream of `StreamItem<InstantTimestamped<T>>` to delay emissions
/// by a specified duration.
pub trait DelayExt<T>: Stream<Item = StreamItem<InstantTimestamped<T>>> + Sized
where
    T: Send,
{
    /// Delays each emission by the specified duration.
    ///
    /// Each item is delayed independently - the delay is applied to each item
    /// as it arrives. Errors are passed through without delay to ensure timely
    /// error propagation.
    ///
    /// # Arguments
    ///
    /// * `duration` - The duration by which to delay each emission
    ///
    /// # Example
    ///
    /// ```rust
    /// use fluxion_stream_time::{DelayExt, InstantTimestamped};
    /// use fluxion_core::StreamItem;
    /// use fluxion_test_utils::test_data::person_alice;
    /// use futures::stream::StreamExt;
    /// use std::time::Duration;
    /// use tokio::sync::mpsc;
    /// use tokio_stream::wrappers::UnboundedReceiverStream;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let (tx, rx) = mpsc::unbounded_channel();
    /// let source = UnboundedReceiverStream::new(rx).map(StreamItem::Value);
    ///
    /// let mut delayed = source.delay(Duration::from_millis(10));
    ///
    /// tx.send(InstantTimestamped::now(person_alice())).unwrap();
    ///
    /// let item = delayed.next().await.unwrap().unwrap();
    /// assert_eq!(&*item, &person_alice());
    /// # }
    /// ```
    fn delay(
        self,
        duration: Duration,
    ) -> impl Stream<Item = StreamItem<InstantTimestamped<T>>> + Send;
}

impl<S, T> DelayExt<T> for S
where
    S: Stream<Item = StreamItem<InstantTimestamped<T>>> + Send,
    T: Send,
{
    fn delay(
        self,
        duration: Duration,
    ) -> impl Stream<Item = StreamItem<InstantTimestamped<T>>> + Send {
        DelayStream {
            stream: self,
            duration,
            in_flight: FuturesOrdered::new(),
            upstream_done: false,
        }
    }
}

#[pin_project]
struct DelayFuture<T> {
    #[pin]
    delay: Sleep,
    value: Option<InstantTimestamped<T>>,
}

impl<T> Future for DelayFuture<T> {
    type Output = StreamItem<InstantTimestamped<T>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.delay.poll(cx) {
            Poll::Ready(()) => {
                if let Some(value) = this.value.take() {
                    Poll::Ready(StreamItem::Value(value))
                } else {
                    // Future contract violation: poll called after completion
                    unreachable!("DelayFuture polled after completion")
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

#[pin_project]
struct DelayStream<S, T> {
    #[pin]
    stream: S,
    duration: Duration,
    in_flight: FuturesOrdered<DelayFuture<T>>,
    upstream_done: bool,
}

impl<S, T> Stream for DelayStream<S, T>
where
    S: Stream<Item = StreamItem<InstantTimestamped<T>>>,
    T: Send,
{
    type Item = StreamItem<InstantTimestamped<T>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        // 1. Poll upstream for new items if not done
        if !*this.upstream_done {
            loop {
                match this.stream.as_mut().poll_next(cx) {
                    Poll::Ready(Some(StreamItem::Value(value))) => {
                        let future = DelayFuture {
                            delay: sleep(*this.duration),
                            value: Some(value),
                        };
                        this.in_flight.push_back(future);
                    }
                    Poll::Ready(Some(StreamItem::Error(err))) => {
                        // Errors pass through immediately without delay
                        return Poll::Ready(Some(StreamItem::Error(err)));
                    }
                    Poll::Ready(None) => {
                        *this.upstream_done = true;
                        break;
                    }
                    Poll::Pending => {
                        break;
                    }
                }
            }
        }

        // 2. Poll in_flight for completed delays
        match this.in_flight.poll_next_unpin(cx) {
            Poll::Ready(Some(item)) => Poll::Ready(Some(item)),
            Poll::Ready(None) => {
                if *this.upstream_done {
                    Poll::Ready(None)
                } else {
                    Poll::Pending
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
