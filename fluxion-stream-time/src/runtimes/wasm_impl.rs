// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[cfg(all(feature = "time-wasm", target_arch = "wasm32"))]
pub mod wasm_implementation {
    use crate::timer::Timer;
    use gloo_timers::future::TimeoutFuture;
    use std::time::Duration;

    /// A WASM-compatible instant using performance.now() via gloo
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct WasmInstant(u64);

    impl WasmInstant {
        fn now() -> Self {
            WasmInstant(js_sys::Date::now() as u64)
        }
    }

    impl std::ops::Sub for WasmInstant {
        type Output = Duration;

        fn sub(self, other: WasmInstant) -> Duration {
            Duration::from_millis(self.0.saturating_sub(other.0))
        }
    }

    impl std::ops::Add<Duration> for WasmInstant {
        type Output = WasmInstant;

        fn add(self, duration: Duration) -> WasmInstant {
            WasmInstant(self.0 + duration.as_millis() as u64)
        }
    }

    impl std::ops::Sub<Duration> for WasmInstant {
        type Output = WasmInstant;

        fn sub(self, duration: Duration) -> WasmInstant {
            WasmInstant(self.0.saturating_sub(duration.as_millis() as u64))
        }
    }

    #[derive(Clone, Debug)]
    pub struct WasmTimer;

    impl WasmTimer {
        #[must_use]
        pub const fn new() -> Self {
            Self
        }
    }

    impl Timer for WasmTimer {
        type Sleep = TimeoutFuture;
        type Instant = WasmInstant;

        fn sleep_future(&self, duration: Duration) -> Self::Sleep {
            TimeoutFuture::new(duration.as_millis() as u32)
        }

        fn now(&self) -> Self::Instant {
            WasmInstant::now()
        }
    }
}
