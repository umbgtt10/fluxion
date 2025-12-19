// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[cfg(feature = "time-wasm")]
pub mod wasm_implementation {
    use crate::timer::Timer;
    use std::time::{Duration, Instant};
    use gloo_timers::future::TimeoutFuture;

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
        type Instant = Instant;

        fn sleep_future(&self, duration: Duration) -> Self::Sleep {
            TimeoutFuture::new(duration.as_millis() as u32)
        }

        fn now(&self) -> Self::Instant {
            Instant::now()
        }
    }
}
