// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::timer::Timer;
use crate::InstantTimestamped;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::time::Duration;

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
use fluxion_core::StreamItem;
use futures::Stream;
use pin_project::pin_project;

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
    /// ```rust,no_run
    /// # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
    /// use fluxion_stream_time::{ThrottleExt, InstantTimestamped, TokioTimer};
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
    /// let mut throttled = source.throttle_with_timer(Duration::from_millis(100), timer.clone());
    ///
    /// // Alice and Bob emitted immediately. Bob should be throttled (dropped).
    /// tx.unbounded_send(InstantTimestamped::new(person_alice(), timer.now())).unwrap();
    /// tx.unbounded_send(InstantTimestamped::new(person_bob(), timer.now())).unwrap();
    ///
    /// // Only Alice should remain (leading throttle)
    /// let item = throttled.next().await.unwrap().unwrap();
    /// assert_eq!(&*item, &person_alice());
    /// # }
    /// # #[cfg(not(all(feature = "runtime-tokio", not(target_arch = "wasm32"))))]
    /// # fn main() {}
    /// ```
    fn throttle_with_timer(
        self,
        duration: Duration,
        timer: TM,
    ) -> impl Stream<Item = StreamItem<InstantTimestamped<T, TM>>>;
}

impl<S, T, TM> ThrottleExt<T, TM> for S
where
    T: Send,
    TM: Timer,
    S: Stream<Item = StreamItem<InstantTimestamped<T, TM>>> + Send,
{
    fn throttle_with_timer(
        self,
        duration: Duration,
        timer: TM,
    ) -> impl Stream<Item = StreamItem<InstantTimestamped<T, TM>>> {
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

// =============================================================================
// Convenience extension trait with default timer
// =============================================================================

/// Extension trait for throttling with a default timer.
///
/// This trait provides a `throttle()` method that automatically uses the
/// appropriate timer for the active runtime feature.
pub trait ThrottleWithDefaultTimerExt<T>: Sized
where
    T: Send,
{
    /// Throttles the stream using the default timer for the active runtime.
    ///
    /// This convenience method is available when exactly one runtime feature is enabled.
    /// It automatically uses the correct timer without requiring an explicit timer parameter.
    fn throttle(self, duration: Duration) -> impl Stream<Item = StreamItem<Self::Timestamped>>;

    /// The timestamped type for this runtime.
    type Timestamped;
}

#[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
impl<S, T> ThrottleWithDefaultTimerExt<T> for S
where
    S: Stream<Item = StreamItem<crate::TokioTimestamped<T>>> + Send,
    T: Send,
{
    type Timestamped = crate::TokioTimestamped<T>;

    fn throttle(self, duration: Duration) -> impl Stream<Item = StreamItem<Self::Timestamped>> {
        ThrottleExt::throttle_with_timer(self, duration, crate::TokioTimer)
    }
}

#[cfg(all(feature = "runtime-smol", not(feature = "runtime-tokio")))]
impl<S, T> ThrottleWithDefaultTimerExt<T> for S
where
    S: Stream<Item = StreamItem<crate::SmolTimestamped<T>>> + Send,
    T: Send,
{
    type Timestamped = crate::SmolTimestamped<T>;

    fn throttle(self, duration: Duration) -> impl Stream<Item = StreamItem<Self::Timestamped>> {
        ThrottleExt::throttle_with_timer(self, duration, crate::SmolTimer)
    }
}

#[cfg(all(feature = "runtime-wasm", target_arch = "wasm32"))]
impl<S, T> ThrottleWithDefaultTimerExt<T> for S
where
    S: Stream<
            Item = StreamItem<
                InstantTimestamped<T, crate::runtimes::wasm_implementation::WasmTimer>,
            >,
        > + Send,
    T: Send,
{
    type Timestamped = InstantTimestamped<T, crate::runtimes::wasm_implementation::WasmTimer>;

    fn throttle(self, duration: Duration) -> impl Stream<Item = StreamItem<Self::Timestamped>> {
        ThrottleExt::throttle_with_timer(
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
impl<S, T> ThrottleWithDefaultTimerExt<T> for S
where
    S: Stream<Item = StreamItem<InstantTimestamped<T, crate::runtimes::AsyncStdTimer>>> + Send,
    T: Send,
{
    type Timestamped = InstantTimestamped<T, crate::runtimes::AsyncStdTimer>;

    fn throttle(self, duration: Duration) -> impl Stream<Item = StreamItem<Self::Timestamped>> {
        ThrottleExt::throttle_with_timer(self, duration, crate::runtimes::AsyncStdTimer)
    }
}

#[cfg(all(
    feature = "runtime-embassy",
    not(feature = "runtime-tokio"),
    not(feature = "runtime-smol"),
    not(feature = "runtime-async-std")
))]
impl<S, T> ThrottleWithDefaultTimerExt<T> for S
where
    S: Stream<Item = StreamItem<InstantTimestamped<T, crate::runtimes::EmbassyTimerImpl>>> + Send,
    T: Send,
{
    type Timestamped = InstantTimestamped<T, crate::runtimes::EmbassyTimerImpl>;

    fn throttle(self, duration: Duration) -> impl Stream<Item = StreamItem<Self::Timestamped>> {
        ThrottleExt::throttle_with_timer(self, duration, crate::runtimes::EmbassyTimerImpl)
    }
}
