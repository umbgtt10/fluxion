// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::timer::Timer;
use core::fmt::Debug;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::time::Duration;

use alloc::boxed::Box;
use fluxion_core::{Fluxion, HasTimestamp, StreamItem};
use futures::Stream;
use pin_project::pin_project;

/// Extension trait providing the `sample` operator for streams.
///
/// This trait allows any stream of `StreamItem<T>` where `T: Fluxion` to sample emissions
/// at periodic intervals.
pub trait SampleExt<T, TM>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
    TM: Timer<Instant = T::Timestamp>,
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
    /// ```rust,no_run
    /// # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
    /// use fluxion_stream_time::{SampleExt, TokioTimestamped, TokioTimer};
    /// use fluxion_stream_time::timer::Timer;
    /// use fluxion_core::StreamItem;
    /// # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
    /// use fluxion_stream::prelude::*;
    /// use futures::StreamExt;
    /// use core::time::Duration;
    /// use futures::channel::mpsc;
    ///
    /// # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
    /// # #[tokio::main]
    /// # async fn main() {
    /// let (mut tx, rx) = mpsc::unbounded();
    /// let source = rx.map(StreamItem::Value);
    ///
    /// let timer = TokioTimer;
    /// let mut sampled = source.sample(Duration::from_millis(10));
    ///
    /// // Emit values
    /// tx.unbounded_send(TokioTimestamped::new(1, timer.now())).unwrap();
    /// tx.unbounded_send(TokioTimestamped::new(2, timer.now())).unwrap();
    ///
    /// // Sample should pick the latest one
    /// # }
    /// # #[cfg(not(all(feature = "runtime-tokio", not(target_arch = "wasm32"))))]
    /// # fn main() {}
    /// ```
    fn sample(self, duration: Duration) -> impl Stream<Item = StreamItem<T>>;
}

#[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
impl<S, T> SampleExt<T, crate::TokioTimer> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion<Timestamp = std::time::Instant> + Clone,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn sample(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> {
        Box::pin(SampleStream {
            stream: self,
            duration,
            timer: crate::TokioTimer,
            sleep: Some(crate::TokioTimer.sleep_future(duration)),
            pending_value: None,
            is_done: false,
        })
    }
}

#[cfg(all(feature = "runtime-smol", not(feature = "runtime-tokio")))]
impl<S, T> SampleExt<T, crate::SmolTimer> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion<Timestamp = std::time::Instant> + Clone,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn sample(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> {
        Box::pin(SampleStream {
            stream: self,
            duration,
            timer: crate::SmolTimer,
            sleep: Some(crate::SmolTimer.sleep_future(duration)),
            pending_value: None,
            is_done: false,
        })
    }
}

#[cfg(all(feature = "runtime-wasm", target_arch = "wasm32"))]
impl<S, T> SampleExt<T, crate::runtimes::wasm_implementation::WasmTimer> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion<Timestamp = crate::runtimes::wasm_implementation::WasmInstant> + Clone,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn sample(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> {
        let timer = crate::runtimes::wasm_implementation::WasmTimer::new();
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

#[cfg(all(
    feature = "runtime-async-std",
    not(feature = "runtime-tokio"),
    not(feature = "runtime-smol")
))]
impl<S, T> SampleExt<T, crate::runtimes::AsyncStdTimer> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion<Timestamp = std::time::Instant> + Clone,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn sample(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> {
        Box::pin(SampleStream {
            stream: self,
            duration,
            timer: crate::runtimes::AsyncStdTimer,
            sleep: Some(crate::runtimes::AsyncStdTimer.sleep_future(duration)),
            pending_value: None,
            is_done: false,
        })
    }
}

#[cfg(all(
    feature = "runtime-embassy",
    not(feature = "runtime-tokio"),
    not(feature = "runtime-smol"),
    not(feature = "runtime-async-std")
))]
impl<S, T> SampleExt<T, crate::runtimes::EmbassyTimerImpl> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion<Timestamp = crate::runtimes::EmbassyInstant> + Clone,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn sample(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> {
        Box::pin(SampleStream {
            stream: self,
            duration,
            timer: crate::runtimes::EmbassyTimerImpl,
            sleep: Some(crate::runtimes::EmbassyTimerImpl.sleep_future(duration)),
            pending_value: None,
            is_done: false,
        })
    }
}

#[pin_project]
struct SampleStream<S, T, TM: Timer>
where
    S: Stream<Item = StreamItem<T>>,
    T: HasTimestamp<Timestamp = TM::Instant>,
{
    #[pin]
    stream: S,
    duration: Duration,
    timer: TM,
    #[pin]
    sleep: Option<TM::Sleep>,
    pending_value: Option<StreamItem<T>>,
    is_done: bool,
}

impl<S, T, TM> Stream for SampleStream<S, T, TM>
where
    S: Stream<Item = StreamItem<T>>,
    T: HasTimestamp<Timestamp = TM::Instant> + Clone,
    TM: Timer,
{
    type Item = StreamItem<T>;

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
