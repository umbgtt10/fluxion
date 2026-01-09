// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[cfg(feature = "runtime-tokio")]
use std::{sync::Arc, time::Duration};

#[cfg(feature = "runtime-tokio")]
use crate::{runtime::Runtime, timer::Timer};

#[cfg(feature = "runtime-tokio")]
pub struct TokioRuntime;

#[cfg(feature = "runtime-tokio")]
impl Runtime for TokioRuntime {
    type Mutex<T: ?Sized> = Arc<parking_lot::Mutex<T>>;
    type Timer = TokioTimer;
    type Instant = std::time::Instant;
}

#[cfg(feature = "runtime-tokio")]
#[derive(Clone, Debug)]
pub struct TokioTimer;

#[cfg(feature = "runtime-tokio")]
impl Timer for TokioTimer {
    type Sleep = tokio::time::Sleep;

    type Instant = std::time::Instant;

    fn sleep_future(&self, duration: Duration) -> Self::Sleep {
        tokio::time::sleep(duration)
    }

    fn now(&self) -> Self::Instant {
        std::time::Instant::now()
    }
}
