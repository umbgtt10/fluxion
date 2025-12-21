// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Smol runtime implementation for Timer trait.
//!
//! This module provides a Timer implementation for the smol async runtime,
//! which is a small and fast async runtime built on async-executor.
//! Smol is well-maintained and supports both single-threaded and multi-threaded execution.

#[cfg(all(feature = "runtime-smol", not(target_arch = "wasm32")))]
pub mod smol_implementation {
    use crate::timer::Timer;
    use async_io::Timer as AsyncIoTimer;
    use core::pin::Pin;
    use std::task::{Context, Poll};
    use std::time::{Duration, Instant};

    #[derive(Clone, Debug)]
    pub struct SmolTimer;

    /// Wrapper for async-io Timer to implement Future
    pub struct SmolSleep {
        timer: AsyncIoTimer,
    }

    impl SmolSleep {
        fn new(duration: Duration) -> Self {
            Self {
                timer: AsyncIoTimer::after(duration),
            }
        }
    }

    impl std::future::Future for SmolSleep {
        type Output = ();

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            Pin::new(&mut self.timer).poll(cx).map(|_| ())
        }
    }

    impl Timer for SmolTimer {
        type Sleep = SmolSleep;

        type Instant = Instant;

        fn sleep_future(&self, duration: Duration) -> Self::Sleep {
            SmolSleep::new(duration)
        }

        fn now(&self) -> Self::Instant {
            Instant::now()
        }
    }
}
