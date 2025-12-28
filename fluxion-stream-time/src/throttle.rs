// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::timer::Timer;
use core::fmt::Debug;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::time::Duration;

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
use fluxion_core::{Fluxion, HasTimestamp, StreamItem};
use futures::Stream;
use pin_project::pin_project;

/// Extension trait providing the `throttle` operator for streams.
///
/// This trait allows any stream of `StreamItem<T>` where `T: Fluxion` to throttle emissions
/// by a specified duration.
pub trait ThrottleExt<T, TM>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
    TM: Timer<Instant = T::Timestamp>,
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
    /// ```rust,no_run
    /// # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
    /// use fluxion_stream_time::{ThrottleExt, TokioTimestamped, TokioTimer};
    /// use fluxion_stream_time::timer::Timer;
    /// use fluxion_core::StreamItem;
    /// # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
    /// use fluxion_test_utils::test_data::{person_alice, person_bob};
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
    /// let mut throttled = source.throttle(Duration::from_millis(100));
    ///
    /// // Alice and Bob emitted immediately. Bob should be throttled (dropped).
    /// tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now())).unwrap();
    /// tx.unbounded_send(TokioTimestamped::new(person_bob(), timer.now())).unwrap();
    ///
    /// // Only Alice should remain. Timer auto-selected!
    /// # }
    /// # #[cfg(not(all(feature = "runtime-tokio", not(target_arch = "wasm32"))))]
    /// # fn main() {}
    /// ```
    fn throttle(self, duration: Duration) -> impl Stream<Item = StreamItem<T>>;
}

// Feature-gated implementations - one per runtime

#[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
impl<S, T> ThrottleExt<T, crate::TokioTimer> for S
where
    S: Stream<Item = StreamItem<T>> + Send,
    T: Fluxion<Timestamp = std::time::Instant>,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn throttle(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> {
        Box::pin(ThrottleStream::<S, T, _> {
            stream: self,
            duration,
            timer: crate::TokioTimer,
            sleep: Some(crate::TokioTimer.sleep_future(duration)),
            throttling: false,
        })
    }
}

#[cfg(all(feature = "runtime-smol", not(feature = "runtime-tokio")))]
impl<S, T> ThrottleExt<T, crate::SmolTimer> for S
where
    S: Stream<Item = StreamItem<T>> + Send,
    T: Fluxion<Timestamp = std::time::Instant>,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn throttle(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> {
        Box::pin(ThrottleStream::<S, T, _> {
            stream: self,
            duration,
            timer: crate::SmolTimer,
            sleep: Some(crate::SmolTimer.sleep_future(duration)),
            throttling: false,
        })
    }
}

#[cfg(all(feature = "runtime-wasm", target_arch = "wasm32"))]
impl<S, T> ThrottleExt<T, crate::runtimes::wasm_implementation::WasmTimer> for S
where
    S: Stream<Item = StreamItem<T>> + Send,
    T: Fluxion<Timestamp = crate::runtimes::wasm_implementation::WasmInstant>,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn throttle(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> {
        let timer = crate::runtimes::wasm_implementation::WasmTimer::new();
        Box::pin(ThrottleStream::<S, T, _> {
            stream: self,
            duration,
            sleep: Some(timer.sleep_future(duration)),
            timer,
            throttling: false,
        })
    }
}

#[cfg(all(
    feature = "runtime-async-std",
    not(feature = "runtime-tokio"),
    not(feature = "runtime-smol")
))]
impl<S, T> ThrottleExt<T, crate::runtimes::AsyncStdTimer> for S
where
    S: Stream<Item = StreamItem<T>> + Send,
    T: Fluxion<Timestamp = std::time::Instant>,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn throttle(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> {
        Box::pin(ThrottleStream::<S, T, _> {
            stream: self,
            duration,
            timer: crate::runtimes::AsyncStdTimer,
            sleep: Some(crate::runtimes::AsyncStdTimer.sleep_future(duration)),
            throttling: false,
        })
    }
}

#[cfg(all(
    feature = "runtime-embassy",
    not(feature = "runtime-tokio"),
    not(feature = "runtime-smol"),
    not(feature = "runtime-async-std")
))]
impl<S, T> ThrottleExt<T, crate::runtimes::EmbassyTimerImpl> for S
where
    S: Stream<Item = StreamItem<T>> + Send,
    T: Fluxion<Timestamp = crate::runtimes::EmbassyInstant>,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn throttle(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> {
        Box::pin(ThrottleStream::<S, T, _> {
            stream: self,
            duration,
            timer: crate::runtimes::EmbassyTimerImpl,
            sleep: Some(crate::runtimes::EmbassyTimerImpl.sleep_future(duration)),
            throttling: false,
        })
    }
}

#[pin_project]
struct ThrottleStream<S, T, TM: Timer>
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
    throttling: bool,
}

impl<S, T, TM> Stream for ThrottleStream<S, T, TM>
where
    S: Stream<Item = StreamItem<T>>,
    T: HasTimestamp<Timestamp = TM::Instant>,
    TM: Timer,
{
    type Item = StreamItem<T>;

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
