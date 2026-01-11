// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std",
    feature = "runtime-embassy",
    feature = "runtime-wasm"
))]
use crate::DefaultRuntime;
use core::fmt::Debug;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::time::Duration;
use fluxion_core::{Fluxion, HasTimestamp, StreamItem};
use fluxion_runtime::runtime::Runtime;
use fluxion_runtime::timer::Timer;
use futures::stream::FuturesOrdered;
use futures::{Stream, StreamExt};
use pin_project::pin_project;

/// Extension trait providing the `delay` operator for streams.
///
/// This trait allows any stream of `StreamItem<T>` where `T: Fluxion` to delay emissions
/// by a specified duration.
pub trait DelayExt<T, R>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Send + Sync + Ord + Unpin + 'static,
    T::Timestamp: Debug + Ord + Copy + 'static,
    R: Runtime,
{
    /// Delays each emission by the specified duration.
    ///
    /// Each item is delayed independently - the delay is applied to each item
    /// as it arrives. Errors are passed through without delay to ensure timely
    /// error propagation.
    ///
    /// Timer is automatically selected based on runtime features.
    ///
    /// # Arguments
    ///
    /// * `duration` - The duration by which to delay each emission
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
    /// use fluxion_stream_time::{DelayExt, TokioTimestamped};
    /// # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
    /// use fluxion_runtime::impls::tokio::TokioTimer;
    /// # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
    /// use fluxion_runtime::timer::Timer;
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
    /// # let timer = TokioTimer;
    /// let (mut tx, rx) = mpsc::unbounded();
    /// let source = rx.map(StreamItem::Value);
    ///
    /// let mut delayed = source.delay(Duration::from_millis(10));
    ///
    /// # tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now())).unwrap();
    /// // Timer auto-selected based on timestamp type!
    /// # }
    /// # #[cfg(not(all(feature = "runtime-tokio", not(target_arch = "wasm32"))))]
    /// # fn main() {}
    /// ```
    fn delay(self, duration: Duration) -> impl Stream<Item = StreamItem<T>>;
}

// Feature-gated implementations - one per runtime

#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std",
    feature = "runtime-embassy",
    feature = "runtime-wasm"
))]
impl<S, T> DelayExt<T, DefaultRuntime> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion<Timestamp = <DefaultRuntime as Runtime>::Instant>,
    T::Inner: Clone + Debug + Send + Sync + Ord + Unpin + 'static,
{
    fn delay(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> {
        DelayStream::<S, T, DefaultRuntime> {
            stream: self,
            duration,
            in_flight: FuturesOrdered::new(),
            upstream_done: false,
        }
    }
}

#[pin_project]
struct DelayFuture<T, R>
where
    T: HasTimestamp<Timestamp = R::Instant>,
    R: Runtime,
{
    #[pin]
    delay: <R::Timer as Timer>::Sleep,
    value: Option<T>,
}

impl<T, R: Runtime> Future for DelayFuture<T, R>
where
    T: HasTimestamp<Timestamp = R::Instant>,
{
    type Output = StreamItem<T>;

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
struct DelayStream<S, T, R>
where
    S: Stream<Item = StreamItem<T>>,
    T: HasTimestamp<Timestamp = R::Instant>,
    R: Runtime,
    R::Timer: Timer,
{
    #[pin]
    stream: S,
    duration: Duration,
    in_flight: FuturesOrdered<DelayFuture<T, R>>,
    upstream_done: bool,
}

impl<S, T, R> Stream for DelayStream<S, T, R>
where
    S: Stream<Item = StreamItem<T>>,
    T: HasTimestamp<Timestamp = R::Instant>,
    R: Runtime,
    R::Timer: Timer,
{
    type Item = StreamItem<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        // 1. Poll upstream for new items if not done
        if !*this.upstream_done {
            loop {
                match this.stream.as_mut().poll_next(cx) {
                    Poll::Ready(Some(StreamItem::Value(value))) => {
                        let future = DelayFuture {
                            delay: R::Timer::default().sleep_future(*this.duration),
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
