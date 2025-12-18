// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Sample operator for time-based stream processing.

use crate::InstantTimestamped;
use fluxion_core::StreamItem;
use futures::Stream;
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::time::{sleep_until, Instant, Sleep};

/// Extension trait providing the `sample` operator for streams.
///
/// This trait allows any stream of `StreamItem<InstantTimestamped<T>>` to sample emissions
/// at periodic intervals.
pub trait SampleExt<T>: Stream<Item = StreamItem<InstantTimestamped<T>>> + Sized
where
    T: Send + Clone,
{
    /// Samples the stream at periodic intervals.
    ///
    /// The sample operator emits the most recently emitted value from the source
    /// stream within periodic time intervals.
    ///
    /// - If the source emits multiple values within the interval, only the last one is emitted.
    /// - If the source emits no values within the interval, nothing is emitted for that interval.
    /// - Errors are passed through immediately.
    ///
    /// # Arguments
    ///
    /// * `duration` - The sampling interval
    ///
    /// # Example
    ///
    /// ```rust
    /// use fluxion_stream_time::{SampleExt, InstantTimestamped};
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
    /// let mut sampled = source.sample(Duration::from_millis(10));
    ///
    /// // Emit Alice and Bob immediately
    /// tx.send(InstantTimestamped::now(person_alice())).unwrap();
    /// tx.send(InstantTimestamped::now(person_bob())).unwrap();
    ///
    /// // Wait for sample duration
    /// tokio::time::sleep(Duration::from_millis(20)).await;
    ///
    /// // Sample should pick the latest one (Bob)
    /// let item = sampled.next().await.unwrap().unwrap();
    /// assert_eq!(&*item, &person_bob());
    /// # }
    /// ```
    fn sample(
        self,
        duration: Duration,
    ) -> impl Stream<Item = StreamItem<InstantTimestamped<T>>> + Send;
}

impl<S, T> SampleExt<T> for S
where
    S: Stream<Item = StreamItem<InstantTimestamped<T>>> + Send,
    T: Send + Clone,
{
    fn sample(
        self,
        duration: Duration,
    ) -> impl Stream<Item = StreamItem<InstantTimestamped<T>>> + Send {
        SampleStream {
            stream: self,
            duration,
            sleep: Box::pin(sleep_until(Instant::now() + duration)),
            pending_value: None,
            is_done: false,
        }
    }
}

#[pin_project]
struct SampleStream<S: Stream>
where
    S::Item: Clone,
{
    #[pin]
    stream: S,
    duration: Duration,
    sleep: Pin<Box<Sleep>>,
    pending_value: Option<S::Item>,
    is_done: bool,
}

impl<S, T> Stream for SampleStream<S>
where
    S: Stream<Item = StreamItem<InstantTimestamped<T>>>,
    T: Send + Clone,
{
    type Item = StreamItem<InstantTimestamped<T>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        if *this.is_done && this.pending_value.is_none() {
            return Poll::Ready(None);
        }

        // 1. Poll the stream to collect the latest value
        loop {
            match this.stream.as_mut().poll_next(cx) {
                Poll::Ready(Some(item)) => {
                    match item {
                        StreamItem::Value(_) => {
                            // Update pending value with the latest one
                            *this.pending_value = Some(item);
                        }
                        StreamItem::Error(_) => {
                            // Errors pass through immediately
                            return Poll::Ready(Some(item));
                        }
                    }
                }
                Poll::Ready(None) => {
                    *this.is_done = true;
                    // If stream ends, we stop collecting.
                    // We might still have a pending value waiting for the timer,
                    // or we might want to emit it immediately?
                    // RxJS sample: "If the source Observable completes, the result Observable also completes."
                    // It does NOT emit the last value if the timer hasn't fired.
                    // However, some implementations do "sample(period, emitLast: true)".
                    // Let's stick to strict sampling: only emit on tick.
                    // But if the stream is done, we can't wait for more values.
                    // If we just return None, we drop the pending value.
                    // Let's follow the standard: if source completes, we complete.
                    return Poll::Ready(None);
                }
                Poll::Pending => {
                    break;
                }
            }
        }

        // 2. Check the timer
        match this.sleep.as_mut().poll(cx) {
            Poll::Ready(_) => {
                // Timer fired.
                let next_deadline = Instant::now() + *this.duration;
                this.sleep.as_mut().reset(next_deadline);

                if let Some(value) = this.pending_value.take() {
                    Poll::Ready(Some(value))
                } else {
                    // Timer fired but no value.
                    // We need to register the timer again (reset above does not register waker automatically if we don't poll it or return Pending)
                    // But we are in a loop effectively (poll_next).
                    // If we return Pending, we need to make sure we are woken up by the timer or the stream.
                    // The stream returned Pending above.
                    // The timer returned Ready, so we reset it.
                    // We need to poll the timer again to register the waker for the new deadline.
                    match this.sleep.as_mut().poll(cx) {
                        Poll::Pending => Poll::Pending,
                        Poll::Ready(_) => {
                            // This shouldn't happen immediately unless duration is 0
                            Poll::Pending
                        }
                    }
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
