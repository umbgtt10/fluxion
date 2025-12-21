// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! ⚠️ **DEPRECATED**: async-std has been discontinued and is no longer maintained.
//!
//! **Warning**: unmaintained
//! **Title**: async-std has been discontinued
//! **Date**: 2024-08-24
//! **Advisory**: RUSTSEC-2025-0052
//! **URL**: <https://rustsec.org/advisories/RUSTSEC-2025-0052>
//!
//! This implementation is kept for compatibility with existing projects using async-std,
//! but new projects should consider using tokio or smol runtimes instead.

#[cfg(all(feature = "runtime-async-std", not(target_arch = "wasm32")))]
pub mod async_std_implementation {
    use crate::timer::Timer;
    use async_io::Timer as AsyncIoTimer;
    use core::pin::Pin;
    use core::task::{Context, Poll};
    use std::time::{Duration, Instant};

    #[derive(Clone, Debug)]
    pub struct AsyncStdTimer;

    /// Wrapper for async-io Timer to implement Future
    pub struct AsyncStdSleep {
        timer: AsyncIoTimer,
    }

    impl AsyncStdSleep {
        fn new(duration: Duration) -> Self {
            Self {
                timer: AsyncIoTimer::after(duration),
            }
        }
    }

    impl core::future::Future for AsyncStdSleep {
        type Output = ();

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            Pin::new(&mut self.timer).poll(cx).map(|_| ())
        }
    }

    impl Timer for AsyncStdTimer {
        type Sleep = AsyncStdSleep;

        type Instant = Instant;

        fn sleep_future(&self, duration: Duration) -> Self::Sleep {
            AsyncStdSleep::new(duration)
        }

        fn now(&self) -> Self::Instant {
            Instant::now()
        }
    }
}
