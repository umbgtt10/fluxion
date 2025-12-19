// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[cfg(all(feature = "time-tokio", not(target_arch = "wasm32")))]
pub mod tokio_implementation {
    use crate::timer::Timer;
    use std::time::{Duration, Instant};
    use tokio::time::sleep;

    #[derive(Clone, Debug)]
    pub struct TokioTimer;

    impl Timer for TokioTimer {
        type Sleep = tokio::time::Sleep;

        type Instant = Instant;

        fn sleep_future(&self, duration: Duration) -> Self::Sleep {
            sleep(duration)
        }

        fn now(&self) -> Self::Instant {
            Instant::now()
        }
    }
}
