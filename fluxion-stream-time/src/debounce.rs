// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::timer::Timer;
use core::fmt::Debug;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::time::Duration;

#[cfg(not(feature = "std"))]
#[allow(unused_imports)]
use alloc::boxed::Box;
use fluxion_core::{Fluxion, HasTimestamp, StreamItem};
use futures::Stream;
use pin_project::pin_project;

/// Extension trait providing the `debounce` operator for streams.
///
/// This trait allows any stream of `StreamItem<T>` where `T: Fluxion` to debounce emissions
/// by a specified duration.
pub trait DebounceExt<T, TM>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
    TM: Timer<Instant = T::Timestamp>,
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
    /// let mut debounced = source.debounce(Duration::from_millis(100));
    ///
    /// // Alice and Bob emitted immediately. Alice should be debounced (dropped).
    /// tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now())).unwrap();
    /// tx.unbounded_send(TokioTimestamped::new(person_bob(), timer.now())).unwrap();
    ///
    /// // Only Bob should remain (trailing debounce). Timer auto-selected!
    /// # }
    /// # #[cfg(not(all(feature = "runtime-tokio", not(target_arch = "wasm32"))))]
    /// # fn main() {}
    /// ```
    fn debounce(self, duration: Duration) -> impl Stream<Item = StreamItem<T>>;
}

// Feature-gated implementations - one per runtime

#[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
impl<S, T> DebounceExt<T, crate::TokioTimer> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion<Timestamp = std::time::Instant>,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn debounce(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> {
        Box::pin(DebounceStream::<S, T, _> {
            stream: self,
            duration,
            timer: crate::TokioTimer,
            pending_value: None,
            sleep: None,
            stream_ended: false,
        })
    }
}

#[cfg(all(feature = "runtime-smol", not(feature = "runtime-tokio")))]
impl<S, T> DebounceExt<T, crate::SmolTimer> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion<Timestamp = std::time::Instant>,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn debounce(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> {
        Box::pin(DebounceStream::<S, T, _> {
            stream: self,
            duration,
            timer: crate::SmolTimer,
            pending_value: None,
            sleep: None,
            stream_ended: false,
        })
    }
}

#[cfg(all(feature = "runtime-wasm", target_arch = "wasm32"))]
impl<S, T> DebounceExt<T, crate::runtimes::wasm_implementation::WasmTimer> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion<Timestamp = crate::runtimes::wasm_implementation::WasmInstant>,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn debounce(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> {
        Box::pin(DebounceStream::<S, T, _> {
            stream: self,
            duration,
            timer: crate::runtimes::wasm_implementation::WasmTimer::new(),
            pending_value: None,
            sleep: None,
            stream_ended: false,
        })
    }
}

#[cfg(all(
    feature = "runtime-async-std",
    not(feature = "runtime-tokio"),
    not(feature = "runtime-smol")
))]
impl<S, T> DebounceExt<T, crate::runtimes::AsyncStdTimer> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion<Timestamp = std::time::Instant>,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn debounce(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> {
        Box::pin(DebounceStream::<S, T, _> {
            stream: self,
            duration,
            timer: crate::runtimes::AsyncStdTimer,
            pending_value: None,
            sleep: None,
            stream_ended: false,
        })
    }
}

#[cfg(all(
    feature = "runtime-embassy",
    not(feature = "runtime-tokio"),
    not(feature = "runtime-smol"),
    not(feature = "runtime-async-std")
))]
impl<S, T> DebounceExt<T, crate::runtimes::EmbassyTimerImpl> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion<Timestamp = crate::runtimes::EmbassyInstant>,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn debounce(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> {
        Box::pin(DebounceStream::<S, T, _> {
            stream: self,
            duration,
            timer: crate::runtimes::EmbassyTimerImpl,
            pending_value: None,
            sleep: None,
            stream_ended: false,
        })
    }
}

#[pin_project]
struct DebounceStream<S, T, TM: Timer>
where
    S: Stream<Item = StreamItem<T>>,
    T: HasTimestamp<Timestamp = TM::Instant>,
{
    #[pin]
    stream: S,
    duration: Duration,
    timer: TM,
    pending_value: Option<StreamItem<T>>,
    #[pin]
    sleep: Option<TM::Sleep>,
    stream_ended: bool,
}

impl<S, T, TM> Stream for DebounceStream<S, T, TM>
where
    S: Stream<Item = StreamItem<T>>,
    T: HasTimestamp<Timestamp = TM::Instant>,
    TM: Timer,
{
    type Item = StreamItem<T>;

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
