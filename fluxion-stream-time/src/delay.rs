// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::timer::Timer;
use crate::InstantTimestamped;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::time::Duration;
use fluxion_core::StreamItem;
use futures::stream::FuturesOrdered;
use futures::{Stream, StreamExt};
use pin_project::pin_project;

/// Extension trait providing the `delay` operator for streams.
///
/// This trait allows any stream of `StreamItem<InstantTimestamped<T>>` to delay emissions
/// by a specified duration.
pub trait DelayExt<T, TM>: Stream<Item = StreamItem<InstantTimestamped<T, TM>>> + Sized
where
    T: Send,
    TM: Timer,
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
    /// ```rust,no_run
    /// # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
    /// use fluxion_stream_time::{DelayExt, InstantTimestamped, TokioTimer};
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
    /// let mut delayed = source.delay_with_timer(Duration::from_millis(10), timer.clone());
    ///
    /// tx.unbounded_send(InstantTimestamped::new(person_alice(), timer.now())).unwrap();
    ///
    /// let item = delayed.next().await.unwrap().unwrap();
    /// assert_eq!(&*item, &person_alice());
    /// # }
    /// ```
    fn delay_with_timer(
        self,
        duration: Duration,
        timer: TM,
    ) -> impl Stream<Item = StreamItem<InstantTimestamped<T, TM>>>;
}

impl<S, T, TM> DelayExt<T, TM> for S
where
    T: Send,
    TM: Timer,
    S: Stream<Item = StreamItem<InstantTimestamped<T, TM>>>,
{
    fn delay_with_timer(
        self,
        duration: Duration,
        timer: TM,
    ) -> impl Stream<Item = StreamItem<InstantTimestamped<T, TM>>> {
        DelayStream {
            stream: self,
            duration,
            timer,
            in_flight: FuturesOrdered::new(),
            upstream_done: false,
        }
    }
}

#[pin_project]
struct DelayFuture<T, TM: Timer> {
    #[pin]
    delay: TM::Sleep,
    value: Option<InstantTimestamped<T, TM>>,
}

impl<T, TM: Timer> Future for DelayFuture<T, TM> {
    type Output = StreamItem<InstantTimestamped<T, TM>>;

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
struct DelayStream<S, T, TM: Timer> {
    #[pin]
    stream: S,
    duration: Duration,
    timer: TM,
    in_flight: FuturesOrdered<DelayFuture<T, TM>>,
    upstream_done: bool,
}

impl<S, T, TM> Stream for DelayStream<S, T, TM>
where
    S: Stream<Item = StreamItem<InstantTimestamped<T, TM>>>,
    T: Send,
    TM: Timer,
{
    type Item = StreamItem<InstantTimestamped<T, TM>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        // 1. Poll upstream for new items if not done
        if !*this.upstream_done {
            loop {
                match this.stream.as_mut().poll_next(cx) {
                    Poll::Ready(Some(StreamItem::Value(value))) => {
                        let future = DelayFuture {
                            delay: this.timer.sleep_future(*this.duration),
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

// =============================================================================
// Convenience extension trait with default timer
// =============================================================================

/// Extension trait for delaying with a default timer.
///
/// This trait provides a `delay()` method that automatically uses the
/// appropriate timer for the active runtime feature.
pub trait DelayWithDefaultTimerExt<T>: Sized
where
    T: Send,
{
    /// Delays each emission using the default timer for the active runtime.
    ///
    /// This convenience method is available when exactly one runtime feature is enabled.
    /// It automatically uses the correct timer without requiring an explicit timer parameter.
    fn delay(self, duration: Duration) -> impl Stream<Item = StreamItem<Self::Timestamped>>;

    /// The timestamped type for this runtime.
    type Timestamped;
}

#[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
impl<S, T> DelayWithDefaultTimerExt<T> for S
where
    S: Stream<Item = StreamItem<crate::TokioTimestamped<T>>>,
    T: Send,
{
    type Timestamped = crate::TokioTimestamped<T>;

    fn delay(self, duration: Duration) -> impl Stream<Item = StreamItem<Self::Timestamped>> {
        DelayExt::delay_with_timer(self, duration, crate::TokioTimer)
    }
}

#[cfg(all(feature = "runtime-smol", not(feature = "runtime-tokio")))]
impl<S, T> DelayWithDefaultTimerExt<T> for S
where
    S: Stream<Item = StreamItem<crate::SmolTimestamped<T>>>,
    T: Send,
{
    type Timestamped = crate::SmolTimestamped<T>;

    fn delay(self, duration: Duration) -> impl Stream<Item = StreamItem<Self::Timestamped>> {
        DelayExt::delay_with_timer(self, duration, crate::SmolTimer)
    }
}

#[cfg(all(feature = "runtime-wasm", target_arch = "wasm32"))]
impl<S, T> DelayWithDefaultTimerExt<T> for S
where
    S: Stream<
        Item = StreamItem<InstantTimestamped<T, crate::runtimes::wasm_implementation::WasmTimer>>,
    >,
    T: Send,
{
    type Timestamped = InstantTimestamped<T, crate::runtimes::wasm_implementation::WasmTimer>;

    fn delay(self, duration: Duration) -> impl Stream<Item = StreamItem<Self::Timestamped>> {
        DelayExt::delay_with_timer(
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
impl<S, T> DelayWithDefaultTimerExt<T> for S
where
    S: Stream<Item = StreamItem<InstantTimestamped<T, crate::runtimes::AsyncStdTimer>>>,
    T: Send,
{
    type Timestamped = InstantTimestamped<T, crate::runtimes::AsyncStdTimer>;

    fn delay(self, duration: Duration) -> impl Stream<Item = StreamItem<Self::Timestamped>> {
        DelayExt::delay_with_timer(self, duration, crate::runtimes::AsyncStdTimer)
    }
}
