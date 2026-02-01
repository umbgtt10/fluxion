// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

macro_rules! define_debounce_impl {
    ($($bounds:tt)*) => {
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
        use crate::DefaultRuntime;

        pub trait DebounceExt<T, R>: Stream<Item = StreamItem<T>> + Sized
        where
            T: Fluxion,
            T::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
            T::Timestamp: Debug + Ord + Copy + 'static,
            R: Runtime,
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
            fn debounce(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> + $($bounds)*;
        }

        impl<S, T> DebounceExt<T, DefaultRuntime> for S
        where
            S: Stream<Item = StreamItem<T>> + $($bounds)*,
            T: Fluxion<Timestamp = <DefaultRuntime as Runtime>::Instant> + $($bounds)*,
            T::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
        {
            fn debounce(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> + $($bounds)* {
                Box::pin(DebounceStream::<S, T, DefaultRuntime> {
                    stream: self,
                    duration,
                    pending_value: None,
                    sleep: None,
                    stream_ended: false,
                })
            }
        }

        #[pin_project]
        struct DebounceStream<S, T, R>
        where
            S: Stream<Item = StreamItem<T>>,
            T: HasTimestamp<Timestamp = R::Instant>,
            R: Runtime,
            R::Timer: Timer,
        {
            #[pin]
            stream: S,
            duration: Duration,
            pending_value: Option<StreamItem<T>>,
            #[pin]
            sleep: Option<<R::Timer as Timer>::Sleep>,
            stream_ended: bool,
        }

        impl<S, T, R> Stream for DebounceStream<S, T, R>
        where
            S: Stream<Item = StreamItem<T>>,
            T: HasTimestamp<Timestamp = R::Instant>,
            R: Runtime,
        {
            type Item = StreamItem<T>;

            fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                let mut this = self.project();

                loop {
                    if *this.stream_ended {
                        if let Some(item) = this.pending_value.take() {
                            return Poll::Ready(Some(item));
                        }
                        return Poll::Ready(None);
                    }

                    if this.pending_value.is_some() {
                        if let Some(sleep) = this.sleep.as_mut().as_pin_mut() {
                            match sleep.poll(cx) {
                                Poll::Ready(_) => {
                                    this.sleep.set(None);
                                    let item = this.pending_value.take();
                                    return Poll::Ready(item);
                                }
                                Poll::Pending => {}
                            }
                        }
                    }

                    match this.stream.as_mut().poll_next(cx) {
                        Poll::Ready(Some(StreamItem::Value(value))) => {
                            let timer = R::Timer::default();
                            this.sleep.set(Some(timer.sleep_future(*this.duration)));

                            *this.pending_value = Some(StreamItem::Value(value));

                            continue;
                        }
                        Poll::Ready(Some(StreamItem::Error(err))) => {
                            *this.pending_value = None;
                            this.sleep.set(None);
                            return Poll::Ready(Some(StreamItem::Error(err)));
                        }
                        Poll::Ready(None) => {
                            *this.stream_ended = true;
                            continue;
                        }
                        Poll::Pending => {
                            return Poll::Pending;
                        }
                    }
                }
            }
        }
    };
}
