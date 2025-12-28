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
use fluxion_core::{Fluxion, FluxionError, HasTimestamp, StreamItem};
use futures::Stream;
use pin_project::pin_project;

/// Extension trait providing the `timeout` operator for streams.
///
/// This trait allows any stream of `StreamItem<T>` where `T: Fluxion` to enforce a timeout
/// between emissions.
pub trait TimeoutExt<T, TM>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
    TM: Timer<Instant = T::Timestamp>,
{
    /// Errors if the stream does not emit any value within the specified duration.
    ///
    /// The timeout operator monitors the time interval between emissions from the source stream.
    /// If the source stream does not emit any value within the specified duration, the operator
    /// emits a `FluxionError::TimeoutError` with "Timeout" context and terminates the stream.
    ///
    /// - If the source emits a value, the timer is reset.
    /// - If the source completes, the timeout operator completes.
    /// - If the source errors, the error is passed through.
    ///
    /// # Arguments
    ///
    /// * `duration` - The timeout duration
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
    /// use fluxion_stream_time::{TimeoutExt, TokioTimestamped, TokioTimer};
    /// use fluxion_stream_time::timer::Timer;
    /// use fluxion_core::StreamItem;
    /// # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
    /// use fluxion_test_utils::test_data::person_alice;
    /// use futures::stream::StreamExt;
    /// use std::time::Duration;
    /// use futures::channel::mpsc;
    ///
    /// # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
    /// # #[tokio::main]
    /// # async fn main() {
    /// let (mut tx, rx) = mpsc::unbounded();
    /// let source = rx.map(StreamItem::Value);
    ///
    /// let timer = TokioTimer;
    /// let mut timed_out = source.timeout(Duration::from_millis(100));
    ///
    /// tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now())).unwrap();
    /// // Timer auto-selected!
    /// # }
    /// # #[cfg(not(all(feature = "runtime-tokio", not(target_arch = "wasm32"))))]
    /// # fn main() {}
    /// ```
    fn timeout(self, duration: Duration) -> impl Stream<Item = StreamItem<T>>;
}

// Feature-gated implementations - one per runtime

#[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
impl<S, T> TimeoutExt<T, crate::TokioTimer> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion<Timestamp = std::time::Instant>,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn timeout(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> {
        Box::pin(TimeoutStream::<S, T, _> {
            stream: self,
            duration,
            timer: crate::TokioTimer,
            sleep: Some(crate::TokioTimer.sleep_future(duration)),
            is_done: false,
        })
    }
}

#[cfg(all(feature = "runtime-smol", not(feature = "runtime-tokio")))]
impl<S, T> TimeoutExt<T, crate::SmolTimer> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion<Timestamp = std::time::Instant>,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn timeout(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> {
        Box::pin(TimeoutStream::<S, T, _> {
            stream: self,
            duration,
            timer: crate::SmolTimer,
            sleep: Some(crate::SmolTimer.sleep_future(duration)),
            is_done: false,
        })
    }
}

#[cfg(all(feature = "runtime-wasm", target_arch = "wasm32"))]
impl<S, T> TimeoutExt<T, crate::runtimes::wasm_implementation::WasmTimer> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion<Timestamp = crate::runtimes::wasm_implementation::WasmInstant>,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn timeout(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> {
        let timer = crate::runtimes::wasm_implementation::WasmTimer::new();
        Box::pin(TimeoutStream::<S, T, _> {
            stream: self,
            duration,
            sleep: Some(timer.sleep_future(duration)),
            timer,
            is_done: false,
        })
    }
}

#[cfg(all(
    feature = "runtime-async-std",
    not(feature = "runtime-tokio"),
    not(feature = "runtime-smol")
))]
impl<S, T> TimeoutExt<T, crate::runtimes::AsyncStdTimer> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion<Timestamp = std::time::Instant>,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn timeout(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> {
        Box::pin(TimeoutStream::<S, T, _> {
            stream: self,
            duration,
            timer: crate::runtimes::AsyncStdTimer,
            sleep: Some(crate::runtimes::AsyncStdTimer.sleep_future(duration)),
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
impl<S, T> TimeoutExt<T, crate::runtimes::EmbassyTimerImpl> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion<Timestamp = crate::runtimes::EmbassyInstant>,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn timeout(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> {
        Box::pin(TimeoutStream::<S, T, _> {
            stream: self,
            duration,
            timer: crate::runtimes::EmbassyTimerImpl,
            sleep: Some(crate::runtimes::EmbassyTimerImpl.sleep_future(duration)),
            is_done: false,
        })
    }
}

#[pin_project]
struct TimeoutStream<S, T, TM: Timer>
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
    is_done: bool,
}

impl<S, T, TM> Stream for TimeoutStream<S, T, TM>
where
    S: Stream<Item = StreamItem<T>>,
    T: HasTimestamp<Timestamp = TM::Instant>,
    TM: Timer,
{
    type Item = StreamItem<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        if *this.is_done {
            return Poll::Ready(None);
        }

        // 1. Poll the stream
        match this.stream.poll_next(cx) {
            Poll::Ready(Some(item)) => {
                // Reset the timer for the next timeout period
                this.sleep
                    .set(Some(this.timer.sleep_future(*this.duration)));
                return Poll::Ready(Some(item));
            }
            Poll::Ready(None) => {
                *this.is_done = true;
                return Poll::Ready(None);
            }
            Poll::Pending => {
                // Stream is pending, check the timer
            }
        }

        // 2. Poll the timer
        if let Some(sleep) = this.sleep.as_mut().as_pin_mut() {
            match sleep.poll(cx) {
                Poll::Ready(_) => {
                    // Timer fired, emit error and terminate
                    *this.is_done = true;
                    Poll::Ready(Some(StreamItem::Error(FluxionError::timeout_error(
                        "Timeout",
                    ))))
                }
                Poll::Pending => Poll::Pending,
            }
        } else {
            unreachable!("sleep future should always be Some after initialization");
        }
    }
}
