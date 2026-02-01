// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

macro_rules! define_throttle_impl {
    ($($bounds:tt)*) => {
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

        pub trait ThrottleExt<T, R>: Stream<Item = StreamItem<T>> + Sized
        where
            T: Fluxion,
            T::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
            T::Timestamp: Debug + Ord + Copy + 'static,
            R: Runtime,
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
            fn throttle(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> + $($bounds)*;
        }

        impl<S, T> ThrottleExt<T, DefaultRuntime> for S
        where
            S: Stream<Item = StreamItem<T>> + $($bounds)*,
            T: Fluxion<Timestamp = <DefaultRuntime as Runtime>::Instant> + $($bounds)*,
            T::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
        {
            fn throttle(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> + $($bounds)* {
                Box::pin(ThrottleStream::<S, T, DefaultRuntime> {
                    stream: self,
                    duration,
                    sleep: Some(<DefaultRuntime as Runtime>::Timer::default().sleep_future(duration)),
                    throttling: false,
                })
            }
        }

        #[pin_project]
        struct ThrottleStream<S, T, R>
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
            throttling: bool,
        }

        impl<S, T, R> Stream for ThrottleStream<S, T, R>
        where
            S: Stream<Item = StreamItem<T>>,
            T: HasTimestamp<Timestamp = R::Instant>,
            R: Runtime,
        {
            type Item = StreamItem<T>;

            fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                let mut this = self.project();

                loop {
                    if *this.throttling {
                        if let Some(sleep) = this.sleep.as_mut().as_pin_mut() {
                            match sleep.poll(cx) {
                                Poll::Ready(_) => {
                                    *this.throttling = false;
                                }
                                Poll::Pending => {}
                            }
                        }
                    }

                    match this.stream.as_mut().poll_next(cx) {
                        Poll::Ready(Some(StreamItem::Value(value))) => {
                            if !*this.throttling {
                                this.sleep
                                    .set(Some(R::Timer::default().sleep_future(*this.duration)));
                                *this.throttling = true;
                                return Poll::Ready(Some(StreamItem::Value(value)));
                            } else {
                                continue;
                            }
                        }
                        Poll::Ready(Some(StreamItem::Error(err))) => {
                            return Poll::Ready(Some(StreamItem::Error(err)));
                        }
                        Poll::Ready(None) => {
                            return Poll::Ready(None);
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
