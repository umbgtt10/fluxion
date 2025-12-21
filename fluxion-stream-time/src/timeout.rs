// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::timer::Timer;
use crate::InstantTimestamped;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use fluxion_core::{FluxionError, StreamItem};
use futures::Stream;
use pin_project::pin_project;
use std::time::Duration;

/// Extension trait providing the `timeout_with_timer` operator for streams.
///
/// This trait allows any stream of `StreamItem<InstantTimestamped<T>>` to enforce a timeout
/// between emissions.
pub trait TimeoutExt<T, TM>: Stream<Item = StreamItem<InstantTimestamped<T, TM>>> + Sized
where
    TM: Timer,
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
    /// use fluxion_stream_time::{TimeoutExt, InstantTimestamped, TokioTimer};
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
    /// let mut timed_out = source.timeout_with_timer(Duration::from_millis(100), timer.clone());
    ///
    /// tx.unbounded_send(InstantTimestamped::new(person_alice(), timer.now())).unwrap();
    ///
    /// let item = timed_out.next().await.unwrap().unwrap();
    /// assert_eq!(&*item, &person_alice());
    /// # }
    /// ```
    fn timeout_with_timer(
        self,
        duration: Duration,
        timer: TM,
    ) -> impl Stream<Item = StreamItem<InstantTimestamped<T, TM>>>;
}

impl<S, T, TM> TimeoutExt<T, TM> for S
where
    TM: Timer,
    S: Stream<Item = StreamItem<InstantTimestamped<T, TM>>>,
{
    fn timeout_with_timer(
        self,
        duration: Duration,
        timer: TM,
    ) -> impl Stream<Item = StreamItem<InstantTimestamped<T, TM>>> {
        Box::pin(TimeoutStream {
            stream: self,
            duration,
            timer: timer.clone(),
            sleep: Some(timer.sleep_future(duration)),
            is_done: false,
        })
    }
}

#[pin_project]
struct TimeoutStream<S, TM: Timer> {
    #[pin]
    stream: S,
    duration: Duration,
    timer: TM,
    #[pin]
    sleep: Option<TM::Sleep>,
    is_done: bool,
}

impl<S, T, TM> Stream for TimeoutStream<S, TM>
where
    TM: Timer,
    S: Stream<Item = StreamItem<InstantTimestamped<T, TM>>>,
{
    type Item = StreamItem<InstantTimestamped<T, TM>>;

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

// =============================================================================
// Convenience extension trait with default timer
// =============================================================================

/// Extension trait for timeout with a default timer.
///
/// This trait provides a `timeout()` method that automatically uses the
/// appropriate timer for the active runtime feature.
pub trait TimeoutWithDefaultTimerExt<T>: Sized {
    /// Errors if the stream does not emit any value within the specified duration,
    /// using the default timer for the active runtime.
    ///
    /// This convenience method is available when exactly one runtime feature is enabled.
    /// It automatically uses the correct timer without requiring an explicit timer parameter.
    fn timeout(self, duration: Duration) -> impl Stream<Item = StreamItem<Self::Timestamped>>;

    /// The timestamped type for this runtime.
    type Timestamped;
}

#[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
impl<S, T> TimeoutWithDefaultTimerExt<T> for S
where
    S: Stream<Item = StreamItem<crate::TokioTimestamped<T>>>,
{
    type Timestamped = crate::TokioTimestamped<T>;

    fn timeout(self, duration: Duration) -> impl Stream<Item = StreamItem<Self::Timestamped>> {
        TimeoutExt::timeout_with_timer(self, duration, crate::TokioTimer)
    }
}

#[cfg(all(feature = "runtime-smol", not(feature = "runtime-tokio")))]
impl<S, T> TimeoutWithDefaultTimerExt<T> for S
where
    S: Stream<Item = StreamItem<crate::SmolTimestamped<T>>>,
{
    type Timestamped = crate::SmolTimestamped<T>;

    fn timeout(self, duration: Duration) -> impl Stream<Item = StreamItem<Self::Timestamped>> {
        TimeoutExt::timeout_with_timer(self, duration, crate::SmolTimer)
    }
}

#[cfg(all(feature = "runtime-wasm", target_arch = "wasm32"))]
impl<S, T> TimeoutWithDefaultTimerExt<T> for S
where
    S: Stream<
        Item = StreamItem<InstantTimestamped<T, crate::runtimes::wasm_implementation::WasmTimer>>,
    >,
{
    type Timestamped = InstantTimestamped<T, crate::runtimes::wasm_implementation::WasmTimer>;

    fn timeout(self, duration: Duration) -> impl Stream<Item = StreamItem<Self::Timestamped>> {
        TimeoutExt::timeout_with_timer(
            self,
            duration,
            crate::runtimes::wasm_implementation::WasmTimer::new(),
        )
    }
}

#[cfg(all(
    feature = "runtime-async-std",
    not(feature = "runtime-tokio"),
    not(feature = "runtime-smol")
))]
impl<S, T> TimeoutWithDefaultTimerExt<T> for S
where
    S: Stream<Item = StreamItem<InstantTimestamped<T, crate::runtimes::AsyncStdTimer>>>,
{
    type Timestamped = InstantTimestamped<T, crate::runtimes::AsyncStdTimer>;

    fn timeout(self, duration: Duration) -> impl Stream<Item = StreamItem<Self::Timestamped>> {
        TimeoutExt::timeout_with_timer(self, duration, crate::runtimes::AsyncStdTimer)
    }
}
