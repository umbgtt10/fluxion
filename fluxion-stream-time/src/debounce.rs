// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::timer::Timer;
use crate::InstantTimestamped;
use core::pin::Pin;
use fluxion_core::StreamItem;
use futures::Stream;
use pin_project::pin_project;
use std::future::Future;
use std::task::{Context, Poll};
use std::time::Duration;

/// Extension trait providing the `debounce` operator for streams.
///
/// This trait allows any stream of `StreamItem<InstantTimestamped<T, TM>>` to debounce emissions
/// by a specified duration.
pub trait DebounceExt<T, TM>: Stream<Item = StreamItem<InstantTimestamped<T, TM>>> + Sized
where
    T: Send,
    TM: Timer,
{
    /// Debounces the stream by the specified duration.
    ///
    /// The debounce operator waits for a pause in the input stream of at least
    /// the given duration before emitting the latest value. If a new value
    /// arrives before the duration elapses, the timer is reset and only the
    /// newest value is eventually emitted.
    ///
    /// This implements **trailing debounce** semantics (Rx standard):
    /// - When a value arrives, start/restart the timer
    /// - If no new value arrives before the timer expires, emit the latest value
    /// - If a new value arrives, discard the pending value and restart the timer
    /// - When the stream ends, emit any pending value immediately
    ///
    /// Errors pass through immediately without debounce, to ensure timely
    /// error propagation.
    ///
    /// # Arguments
    ///
    /// * `duration` - The duration of required inactivity before emitting a value
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
    /// use fluxion_stream_time::{DebounceExt, TokioTimestamped, TokioTimer};
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
    /// let mut debounced = source.debounce_with_timer(Duration::from_millis(100), timer.clone());
    ///
    /// // Alice and Bob emitted immediately. Alice should be debounced (dropped).
    /// tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now())).unwrap();
    /// tx.unbounded_send(TokioTimestamped::new(person_bob(), timer.now())).unwrap();
    ///
    /// // Only Bob should remain (trailing debounce)
    /// let item = debounced.next().await.unwrap().unwrap();
    /// assert_eq!(&*item, &person_bob());
    /// # }
    /// ```
    fn debounce_with_timer(
        self,
        duration: Duration,
        timer: TM,
    ) -> impl Stream<Item = StreamItem<InstantTimestamped<T, TM>>>;
}

impl<S, T, TM> DebounceExt<T, TM> for S
where
    S: Stream<Item = StreamItem<InstantTimestamped<T, TM>>>,
    T: Send,
    TM: Timer,
{
    fn debounce_with_timer(
        self,
        duration: Duration,
        timer: TM,
    ) -> impl Stream<Item = StreamItem<InstantTimestamped<T, TM>>> {
        Box::pin(DebounceStream {
            stream: self,
            duration,
            timer,
            pending_value: None,
            sleep: None,
            stream_ended: false,
        })
    }
}

#[pin_project]
struct DebounceStream<S: Stream, TM: Timer> {
    #[pin]
    stream: S,
    duration: Duration,
    timer: TM,
    pending_value: Option<S::Item>,
    #[pin]
    sleep: Option<TM::Sleep>,
    stream_ended: bool,
}

impl<S, T, TM> Stream for DebounceStream<S, TM>
where
    S: Stream<Item = StreamItem<InstantTimestamped<T, TM>>>,
    T: Send,
    TM: Timer,
{
    type Item = StreamItem<InstantTimestamped<T, TM>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            // If stream ended and we have a pending value, emit it immediately
            if *this.stream_ended {
                if let Some(item) = this.pending_value.take() {
                    return Poll::Ready(Some(item));
                }
                return Poll::Ready(None);
            }

            // Check if we have a pending debounced value and its timer
            if this.pending_value.is_some() {
                if let Some(sleep) = this.sleep.as_mut().as_pin_mut() {
                    match sleep.poll(cx) {
                        Poll::Ready(_) => {
                            // Timer expired, emit the pending value
                            this.sleep.set(None);
                            let item = this.pending_value.take();
                            return Poll::Ready(item);
                        }
                        Poll::Pending => {
                            // Timer still running, check for new values
                        }
                    }
                }
            }

            // Poll the source stream for the next item
            match this.stream.as_mut().poll_next(cx) {
                Poll::Ready(Some(StreamItem::Value(value))) => {
                    // New value arrived - reset the debounce timer by creating a new sleep future
                    this.sleep
                        .set(Some(this.timer.sleep_future(*this.duration)));

                    // Replace any pending value with this new one
                    *this.pending_value = Some(StreamItem::Value(value));

                    // Continue polling to check the timer (it might be 0 duration)
                    continue;
                }
                Poll::Ready(Some(StreamItem::Error(err))) => {
                    // Errors pass through immediately, discarding any pending value
                    *this.pending_value = None;
                    this.sleep.set(None);
                    return Poll::Ready(Some(StreamItem::Error(err)));
                }
                Poll::Ready(None) => {
                    // Stream ended - mark it and loop to emit pending value if any
                    *this.stream_ended = true;
                    continue;
                }
                Poll::Pending => {
                    // No new values from source
                    // If we have a pending value, we're waiting for its timer
                    // Otherwise, we're waiting for the next source value
                    return Poll::Pending;
                }
            }
        }
    }
}

// =============================================================================
// Convenience extension trait with default timer
// =============================================================================

/// Extension trait for debouncing with a default timer.
///
/// This trait provides a `debounce()` method that automatically uses the
/// appropriate timer for the active runtime feature.
pub trait DebounceWithDefaultTimerExt<T>: Sized
where
    T: Send,
{
    /// Debounces the stream using the default timer for the active runtime.
    ///
    /// This convenience method is available when exactly one runtime feature is enabled.
    /// It automatically uses the correct timer (`TokioTimer`, `SmolTimer`, etc.)
    /// without requiring an explicit timer parameter.
    ///
    /// # Examples
    ///
    /// With the `runtime-tokio` feature:
    /// ```rust,no_run
    /// # #[cfg(not(target_arch = "wasm32"))]
    /// use fluxion_stream_time::prelude::*;
    /// # #[cfg(not(target_arch = "wasm32"))]
    /// use fluxion_stream_time::{TokioTimestamped, TokioTimer};
    /// use fluxion_stream_time::timer::Timer;
    /// use fluxion_core::StreamItem;
    /// use futures::stream::StreamExt;
    /// use std::time::Duration;
    ///
    /// # #[cfg(not(target_arch = "wasm32"))]
    /// # #[tokio::main]
    /// # async fn main() {
    /// # #[cfg(not(target_arch = "wasm32"))]
    /// # let timer = TokioTimer;
    /// # #[cfg(not(target_arch = "wasm32"))]
    /// # let source: futures::stream::Empty<StreamItem<TokioTimestamped<i32>>> = futures::stream::empty();
    /// // No timer parameter needed!
    /// let debounced = source.debounce(Duration::from_millis(100));
    /// # }
    /// ```
    fn debounce(self, duration: Duration) -> impl Stream<Item = StreamItem<Self::Timestamped>>;

    /// The timestamped type for this runtime.
    type Timestamped;
}

#[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
impl<S, T> DebounceWithDefaultTimerExt<T> for S
where
    S: Stream<Item = StreamItem<crate::TokioTimestamped<T>>>,
    T: Send,
{
    type Timestamped = crate::TokioTimestamped<T>;

    fn debounce(self, duration: Duration) -> impl Stream<Item = StreamItem<Self::Timestamped>> {
        DebounceExt::debounce_with_timer(self, duration, crate::TokioTimer)
    }
}

#[cfg(all(feature = "runtime-smol", not(feature = "runtime-tokio")))]
impl<S, T> DebounceWithDefaultTimerExt<T> for S
where
    S: Stream<Item = StreamItem<crate::SmolTimestamped<T>>>,
    T: Send,
{
    type Timestamped = crate::SmolTimestamped<T>;

    fn debounce(self, duration: Duration) -> impl Stream<Item = StreamItem<Self::Timestamped>> {
        DebounceExt::debounce_with_timer(self, duration, crate::SmolTimer)
    }
}

#[cfg(all(feature = "runtime-wasm", target_arch = "wasm32"))]
impl<S, T> DebounceWithDefaultTimerExt<T> for S
where
    S: Stream<
        Item = StreamItem<InstantTimestamped<T, crate::runtimes::wasm_implementation::WasmTimer>>,
    >,
    T: Send,
{
    type Timestamped = InstantTimestamped<T, crate::runtimes::wasm_implementation::WasmTimer>;

    fn debounce(self, duration: Duration) -> impl Stream<Item = StreamItem<Self::Timestamped>> {
        DebounceExt::debounce_with_timer(
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
impl<S, T> DebounceWithDefaultTimerExt<T> for S
where
    S: Stream<Item = StreamItem<InstantTimestamped<T, crate::runtimes::AsyncStdTimer>>>,
    T: Send,
{
    type Timestamped = InstantTimestamped<T, crate::runtimes::AsyncStdTimer>;

    fn debounce(self, duration: Duration) -> impl Stream<Item = StreamItem<Self::Timestamped>> {
        DebounceExt::debounce_with_timer(self, duration, crate::runtimes::AsyncStdTimer)
    }
}
