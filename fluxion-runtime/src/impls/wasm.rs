// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[cfg(feature = "runtime-wasm")]
use std::{sync::Arc, time::Duration};

#[cfg(feature = "runtime-wasm")]
use crate::{runtime::Runtime, timer::Timer};

#[cfg(feature = "runtime-wasm")]
pub struct WasmRuntime;

#[cfg(feature = "runtime-wasm")]
impl Runtime for WasmRuntime {
    type Mutex<T: ?Sized> = Arc<parking_lot::Mutex<T>>;
    type Timer = WasmTimer;
    type Instant = WasmInstant;
}

#[cfg(feature = "runtime-wasm")]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WasmInstant(u64);

#[cfg(feature = "runtime-wasm")]
impl WasmInstant {
    fn now() -> Self {
        WasmInstant(js_sys::Date::now() as u64)
    }
}

#[cfg(feature = "runtime-wasm")]
impl std::ops::Sub for WasmInstant {
    type Output = Duration;

    fn sub(self, other: WasmInstant) -> Duration {
        Duration::from_millis(self.0.saturating_sub(other.0))
    }
}

#[cfg(feature = "runtime-wasm")]
impl std::ops::Add<Duration> for WasmInstant {
    type Output = WasmInstant;

    fn add(self, duration: Duration) -> WasmInstant {
        WasmInstant(self.0 + duration.as_millis() as u64)
    }
}

#[cfg(feature = "runtime-wasm")]
impl std::ops::Sub<Duration> for WasmInstant {
    type Output = WasmInstant;

    fn sub(self, duration: Duration) -> WasmInstant {
        WasmInstant(self.0.saturating_sub(duration.as_millis() as u64))
    }
}

#[cfg(feature = "runtime-wasm")]
impl std::ops::AddAssign<Duration> for WasmInstant {
    fn add_assign(&mut self, duration: Duration) {
        self.0 += duration.as_millis() as u64;
    }
}

#[cfg(feature = "runtime-wasm")]
impl std::ops::SubAssign<Duration> for WasmInstant {
    fn sub_assign(&mut self, duration: Duration) {
        self.0 = self.0.saturating_sub(duration.as_millis() as u64);
    }
}

#[cfg(feature = "runtime-wasm")]
#[derive(Clone, Debug)]
pub struct WasmTimer;

#[cfg(feature = "runtime-wasm")]
impl Timer for WasmTimer {
    type Sleep = gloo_timers::future::TimeoutFuture;
    type Instant = WasmInstant;

    fn sleep_future(&self, duration: Duration) -> Self::Sleep {
        gloo_timers::future::TimeoutFuture::new(duration.as_millis() as u32)
    }

    fn now(&self) -> Self::Instant {
        WasmInstant::now()
    }
}
