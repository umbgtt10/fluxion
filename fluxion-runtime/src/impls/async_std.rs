// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[cfg(feature = "runtime-async-std")]
use std::sync::Arc;

#[cfg(feature = "runtime-async-std")]
use crate::{runtime::Runtime, timer::Timer};

#[cfg(feature = "runtime-async-std")]
#[derive(Debug)]
pub struct AsyncStdRuntime;

#[cfg(feature = "runtime-async-std")]
impl Runtime for AsyncStdRuntime {
    type Mutex<T: ?Sized> = Arc<parking_lot::Mutex<T>>;
    type Timer = AsyncStdTimer;
    type Instant = std::time::Instant;
}

#[cfg(feature = "runtime-async-std")]
#[derive(Clone, Debug, Default)]
pub struct AsyncStdTimer;

#[cfg(feature = "runtime-async-std")]
pub struct AsyncStdSleep {
    timer: async_io::Timer,
}

#[cfg(feature = "runtime-async-std")]
impl AsyncStdSleep {
    fn new(duration: std::time::Duration) -> Self {
        Self {
            timer: async_io::Timer::after(duration),
        }
    }
}

#[cfg(feature = "runtime-async-std")]
impl core::future::Future for AsyncStdSleep {
    type Output = ();

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        core::pin::Pin::new(&mut self.timer).poll(cx).map(|_| ())
    }
}

#[cfg(feature = "runtime-async-std")]
impl Timer for AsyncStdTimer {
    type Sleep = AsyncStdSleep;

    type Instant = std::time::Instant;

    fn sleep_future(&self, duration: std::time::Duration) -> Self::Sleep {
        AsyncStdSleep::new(duration)
    }

    fn now(&self) -> Self::Instant {
        std::time::Instant::now()
    }
}
