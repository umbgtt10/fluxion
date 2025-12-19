// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Throttle operator for time-based stream processing.

use crate::timer::Timer;
use crate::InstantTimestamped;
use fluxion_core::StreamItem;
use futures::Stream;
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

/// Extension trait providing the `throttle` operator for streams.
///
/// This trait allows any stream of `StreamItem<InstantTimestamped<T>>` to throttle emissions
/// by a specified duration.
pub trait ThrottleExt<T, TM>: Stream<Item = StreamItem<InstantTimestamped<T, TM>>> + Sized
where
    T: Send,
    TM: Timer,
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
    /// use fluxion_stream_time::{ThrottleExt, InstantTimestamped};
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
    /// tx.send(InstantTimestamped::now(person_alice())).unwrap();
    /// tx.send(InstantTimestamped::now(person_bob())).unwrap();
    ///
    /// // Only Alice should remain (leading throttle)
    /// let item = throttled.next().await.unwrap().unwrap();
    /// assert_eq!(&*item, &person_alice());
    /// # }
    /// ```
    fn throttle(
        self,
        duration: Duration,
        timer: TM,
    ) -> impl Stream<Item = StreamItem<InstantTimestamped<T, TM>>> + Send;
}

impl<S, T, TM> ThrottleExt<T, TM> for S
where
    T: Send,
    TM: Timer,
    S: Stream<Item = StreamItem<InstantTimestamped<T, TM>>> + Send,
{
    fn throttle(
        self,
        duration: Duration,
        timer: TM,
    ) -> impl Stream<Item = StreamItem<InstantTimestamped<T, TM>>> + Send {
        Box::pin(ThrottleStream {
            stream: self,
            duration,
            timer: timer.clone(),
            sleep: Some(timer.sleep_future(duration)),
            throttling: false,
        })
    }
}

#[pin_project]
struct ThrottleStream<S: Stream, TM: Timer> {
    #[pin]
    stream: S,
    duration: Duration,
    timer: TM,
    #[pin]
    sleep: Option<TM::Sleep>,
    throttling: bool,
}

impl<S, T, TM> Stream for ThrottleStream<S, TM>
where
    T: Send,
    TM: Timer,
    S: Stream<Item = StreamItem<InstantTimestamped<T, TM>>>,
{
    type Item = StreamItem<InstantTimestamped<T, TM>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            // 1. Check timer if throttling
            // Simplify to just:
            if *this.throttling {
                if let Some(sleep) = this.sleep.as_mut().as_pin_mut() {
                    match sleep.poll(cx) {
                        Poll::Ready(_) => {
                            *this.throttling = false;
                        }
                        Poll::Pending => {
                            // Timer still running
                        }
                    }
                }
            }

            // 2. Poll stream
            match this.stream.as_mut().poll_next(cx) {
                Poll::Ready(Some(StreamItem::Value(value))) => {
                    if !*this.throttling {
                        // Not throttling: Emit value, start timer
                        this.sleep
                            .set(Some(this.timer.sleep_future(*this.duration)));
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
