// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[cfg(feature = "runtime-smol")]
use std::sync::Arc;

#[cfg(feature = "runtime-smol")]
use crate::{runtime::Runtime, timer::Timer};

#[cfg(feature = "runtime-smol")]
#[derive(Debug)]
pub struct SmolRuntime;

#[cfg(feature = "runtime-smol")]
impl Runtime for SmolRuntime {
    type Mutex<T: ?Sized> = Arc<parking_lot::Mutex<T>>;
    type Timer = SmolTimer;
    type Instant = std::time::Instant;
}

#[cfg(feature = "runtime-smol")]
#[derive(Clone, Debug, Default)]
pub struct SmolTimer;

#[cfg(feature = "runtime-smol")]
pub struct SmolSleep {
    timer: async_io::Timer,
}

#[cfg(feature = "runtime-smol")]
impl SmolSleep {
    fn new(duration: std::time::Duration) -> Self {
        Self {
            timer: async_io::Timer::after(duration),
        }
    }
}

#[cfg(feature = "runtime-smol")]
impl core::future::Future for SmolSleep {
    type Output = ();

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        core::pin::Pin::new(&mut self.timer).poll(cx).map(|_| ())
    }
}

#[cfg(feature = "runtime-smol")]
impl Timer for SmolTimer {
    type Sleep = SmolSleep;

    type Instant = std::time::Instant;

    fn sleep_future(&self, duration: std::time::Duration) -> Self::Sleep {
        SmolSleep::new(duration)
    }

    fn now(&self) -> Self::Instant {
        std::time::Instant::now()
    }
}
