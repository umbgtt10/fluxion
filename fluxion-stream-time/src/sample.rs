// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::timer::Timer;
use crate::InstantTimestamped;
use fluxion_core::StreamItem;
use futures::Stream;
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

/// Extension trait providing the `sample` operator for streams.
///
/// This trait allows any stream of `StreamItem<InstantTimestamped<T>>` to sample emissions
/// at periodic intervals.
pub trait SampleExt<T, TM>: Stream<Item = StreamItem<InstantTimestamped<T, TM>>> + Sized
where
    T: Send + Clone,
    TM: Timer,
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
    /// use fluxion_stream_time::{SampleExt, InstantTimestamped, TokioTimer};
    /// use fluxion_stream_time::timer::Timer;
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
    /// let timer = TokioTimer;
    /// let mut sampled = source.sample(Duration::from_millis(10), timer.clone());
    ///
    /// // Emit Alice and Bob immediately
    /// tx.send(InstantTimestamped::new(person_alice(), timer.now())).unwrap();
    /// tx.send(InstantTimestamped::new(person_bob(), timer.now())).unwrap();
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
        timer: TM,
    ) -> impl Stream<Item = StreamItem<InstantTimestamped<T, TM>>>;
}

impl<S, T, TM> SampleExt<T, TM> for S
where
    T: Send + Clone,
    TM: Timer,
    S: Stream<Item = StreamItem<InstantTimestamped<T, TM>>>,
{
    fn sample(
        self,
        duration: Duration,
        timer: TM,
    ) -> impl Stream<Item = StreamItem<InstantTimestamped<T, TM>>> {
        Box::pin(SampleStream {
            stream: self,
            duration,
            timer: timer.clone(),
            sleep: Some(timer.sleep_future(duration)),
            pending_value: None,
            is_done: false,
        })
    }
}

#[pin_project]
struct SampleStream<S: Stream, TM>
where
    S::Item: Clone,
    TM: Timer,
{
    #[pin]
    stream: S,
    duration: Duration,
    timer: TM,
    #[pin]
    sleep: Option<TM::Sleep>,
    pending_value: Option<S::Item>,
    is_done: bool,
}

impl<S, T, TM> Stream for SampleStream<S, TM>
where
    S: Stream<Item = StreamItem<InstantTimestamped<T, TM>>>,
    T: Send + Clone,
    TM: Timer,
{
    type Item = StreamItem<InstantTimestamped<T, TM>>;

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
                    return Poll::Ready(None);
                }
                Poll::Pending => {
                    break;
                }
            }
        }

        // 2. Check the timer
        if let Some(sleep) = this.sleep.as_mut().as_pin_mut() {
            match sleep.poll(cx) {
                Poll::Ready(_) => {
                    // Timer fired.
                    this.sleep
                        .set(Some(this.timer.sleep_future(*this.duration)));

                    if let Some(value) = this.pending_value.take() {
                        Poll::Ready(Some(value))
                    } else {
                        Poll::Pending
                    }
                }
                Poll::Pending => Poll::Pending,
            }
        } else {
            unreachable!("sleep future should always be Some after initialization")
        }
    }
}
