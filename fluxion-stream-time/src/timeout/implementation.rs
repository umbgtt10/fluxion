// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

macro_rules! define_timeout_impl {
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
        use fluxion_core::{Fluxion, FluxionError, HasTimestamp, StreamItem};
        use fluxion_runtime::runtime::Runtime;
        use fluxion_runtime::timer::Timer;
        use futures::Stream;
        use pin_project::pin_project;

        /// Extension trait providing the `timeout` operator for streams.
        ///
        /// This trait allows any stream of `StreamItem<T>` where `T: Fluxion` to enforce a timeout
        /// between emissions.
        pub trait TimeoutExt<T, R>: Stream<Item = StreamItem<T>> + Sized
        where
            T: Fluxion,
            T::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
            T::Timestamp: Debug + Ord + Copy + 'static,
            R: Runtime,
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
            fn timeout(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> + $($bounds)*;
        }

        impl<S, T> TimeoutExt<T, DefaultRuntime> for S
        where
            S: Stream<Item = StreamItem<T>> + $($bounds)*,
            T: Fluxion<Timestamp = <DefaultRuntime as Runtime>::Instant> + $($bounds)*,
            T::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
        {
            fn timeout(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> + $($bounds)* {
                Box::pin(TimeoutStream::<S, T, DefaultRuntime> {
                    stream: self,
                    duration,
                    sleep: Some(<DefaultRuntime as Runtime>::Timer::default().sleep_future(duration)),
                    is_done: false,
                })
            }
        }

        #[pin_project]
        struct TimeoutStream<S, T, R>
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
            is_done: bool,
        }

        impl<S, T, R> Stream for TimeoutStream<S, T, R>
        where
            S: Stream<Item = StreamItem<T>>,
            T: HasTimestamp<Timestamp = R::Instant>,
            R: Runtime,
        {
            type Item = StreamItem<T>;

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
                            .set(Some(R::Timer::default().sleep_future(*this.duration)));
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
    };
}
