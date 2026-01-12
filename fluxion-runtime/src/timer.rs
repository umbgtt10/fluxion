// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::clone::Clone;
use core::cmp::Ord;
use core::fmt::Debug;
use core::future::Future;
use core::marker::{Copy, Send, Sync};
use core::ops::{Add, Sub};
use core::time::Duration;

pub trait Timer: Clone + Send + Sync + Debug + 'static {
    type Sleep: Future<Output = ()>;

    type Instant: Copy
        + Debug
        + Ord
        + Send
        + Sync
        + Add<Duration, Output = Self::Instant>
        + Sub<Duration, Output = Self::Instant>
        + Sub<Self::Instant, Output = Duration>;

    fn sleep_future(&self, duration: Duration) -> Self::Sleep;

    fn now(&self) -> Self::Instant;
}
