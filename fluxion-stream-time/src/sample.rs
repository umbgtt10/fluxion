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

#[cfg(not(feature = "std"))]
#[allow(unused_imports)]
use alloc::boxed::Box;
use fluxion_core::{Fluxion, HasTimestamp, StreamItem};
use fluxion_runtime::runtime::Runtime;
use fluxion_runtime::timer::Timer;
use futures::Stream;
use pin_project::pin_project;

/// Extension trait providing the `sample` operator for streams.
///
/// This trait allows any stream of `StreamItem<T>` where `T: Fluxion` to sample emissions
/// at periodic intervals.
pub trait SampleExt<T, R>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Send + Sync + Ord + Unpin + 'static,
    T::Timestamp: Debug + Ord + Copy + 'static,
    R: Runtime,
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
    /// use fluxion_stream_time::{SampleExt, TokioTimestamped};
    /// # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
    /// use fluxion_runtime::impls::tokio::TokioTimer;
    /// # #[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
    /// use fluxion_runtime::timer::Timer;
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

#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std",
    feature = "runtime-embassy",
    feature = "runtime-wasm"
))]
impl<S, T> SampleExt<T, DefaultRuntime> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion<Timestamp = <DefaultRuntime as Runtime>::Instant>,
    T::Inner: Clone + Debug + Send + Sync + Ord + Unpin + 'static,
{
    fn sample(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> {
        Box::pin(SampleStream::<S, T, DefaultRuntime> {
            stream: self,
            duration,
            sleep: Some(<DefaultRuntime as Runtime>::Timer::default().sleep_future(duration)),
            pending_value: None,
            is_done: false,
        })
    }
}

#[pin_project]
struct SampleStream<S, T, R>
where
    S: Stream<Item = StreamItem<T>>,
    T: HasTimestamp<Timestamp = R::Instant>,
    R: Runtime,
{
    #[pin]
    stream: S,
    duration: Duration,
    #[pin]
    sleep: Option<<R::Timer as Timer>::Sleep>,
    pending_value: Option<StreamItem<T>>,
    is_done: bool,
}

impl<S, T, R> Stream for SampleStream<S, T, R>
where
    S: Stream<Item = StreamItem<T>>,
    T: HasTimestamp<Timestamp = R::Instant> + Clone,
    R: Runtime,
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
                        .set(Some(R::Timer::default().sleep_future(*this.duration)));

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
