// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Throttle operator for time-based stream processing.

use crate::ChronoTimestamped;
use fluxion_core::StreamItem;
use futures::Stream;
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::time::{sleep, Instant, Sleep};

/// Extension trait providing the `throttle` operator for streams.
///
/// This trait allows any stream of `StreamItem<ChronoTimestamped<T>>` to throttle emissions
/// by a specified duration.
pub trait ThrottleExt<T>: Stream<Item = StreamItem<ChronoTimestamped<T>>> + Sized
where
    T: Send,
{
    /// Throttles the stream by the specified duration.
    ///
    /// The throttle operator emits the first value, then ignores subsequent values
    /// for the specified duration. After the duration expires, it accepts the next
    /// value and repeats the process.
    ///
    /// This implements **leading throttle** semantics:
    /// - When a value arrives and we are not throttling:
    ///   - Emit the value immediately
    ///   - Start the throttle timer
    ///   - Ignore subsequent values until the timer expires
    /// - When the timer expires:
    ///   - We become ready to accept a new value
    ///
    /// Errors pass through immediately without throttling, to ensure timely
    /// error propagation.
    ///
    /// # Arguments
    ///
    /// * `duration` - The duration to ignore values after an emission
    ///
    /// # Example
    ///
    /// ```rust
    /// use fluxion_stream_time::{ThrottleExt, ChronoTimestamped};
    /// use fluxion_core::StreamItem;
    /// use fluxion_test_utils::test_data::{person_alice, person_bob};
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
    /// let mut throttled = source.throttle(Duration::from_millis(100));
    ///
    /// // Alice and Bob emitted immediately. Bob should be throttled (dropped).
    /// tx.send(ChronoTimestamped::now(person_alice())).unwrap();
    /// tx.send(ChronoTimestamped::now(person_bob())).unwrap();
    ///
    /// // Only Alice should remain (leading throttle)
    /// let item = throttled.next().await.unwrap().unwrap();
    /// assert_eq!(&*item, &person_alice());
    /// # }
    /// ```
    fn throttle(
        self,
        duration: Duration,
    ) -> impl Stream<Item = StreamItem<ChronoTimestamped<T>>> + Send;
}

impl<S, T> ThrottleExt<T> for S
where
    S: Stream<Item = StreamItem<ChronoTimestamped<T>>> + Send,
    T: Send,
{
    fn throttle(
        self,
        duration: Duration,
    ) -> impl Stream<Item = StreamItem<ChronoTimestamped<T>>> + Send {
        ThrottleStream {
            stream: self,
            duration,
            sleep: Box::pin(sleep(Duration::from_millis(0))),
            throttling: false,
        }
    }
}

#[pin_project]
struct ThrottleStream<S: Stream> {
    #[pin]
    stream: S,
    duration: std::time::Duration,
    sleep: Pin<Box<Sleep>>,
    throttling: bool,
}

impl<S, T> Stream for ThrottleStream<S>
where
    S: Stream<Item = StreamItem<ChronoTimestamped<T>>>,
    T: Send,
{
    type Item = StreamItem<ChronoTimestamped<T>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            // 1. Check timer if throttling
            if *this.throttling {
                // Check if deadline has passed explicitly to handle cases where Sleep poll lags
                if Instant::now() >= this.sleep.deadline() {
                    *this.throttling = false;
                } else {
                    match this.sleep.as_mut().poll(cx) {
                        Poll::Ready(_) => {
                            *this.throttling = false;
                            // Timer expired, we are open to new values.
                        }
                        Poll::Pending => {
                            // Timer running.
                        }
                    }
                }
            }

            // 2. Poll stream
            match this.stream.as_mut().poll_next(cx) {
                Poll::Ready(Some(StreamItem::Value(value))) => {
                    if !*this.throttling {
                        // Not throttling: Emit value, start timer
                        let deadline = Instant::now() + *this.duration;
                        this.sleep.as_mut().reset(deadline);
                        *this.throttling = true;
                        return Poll::Ready(Some(StreamItem::Value(value)));
                    } else {
                        // Throttling: Drop value, continue polling stream
                        continue;
                    }
                }
                Poll::Ready(Some(StreamItem::Error(err))) => {
                    // Errors pass through immediately
                    return Poll::Ready(Some(StreamItem::Error(err)));
                }
                Poll::Ready(None) => {
                    return Poll::Ready(None);
                }
                Poll::Pending => {
                    // If we are throttling, we need to ensure we are woken up by the timer.
                    // this.sleep.poll(cx) above already registered the waker if it was Pending.
                    // this.stream.poll_next(cx) registered the waker if it was Pending.
                    return Poll::Pending;
                }
            }
        }
    }
}
