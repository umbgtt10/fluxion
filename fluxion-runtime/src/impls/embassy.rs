// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[cfg(feature = "runtime-embassy")]
use crate::{runtime::Runtime, timer::Timer};

#[cfg(feature = "runtime-embassy")]
extern crate alloc;

#[cfg(feature = "runtime-embassy")]
use crate::mutex::MutexLike;

#[cfg(feature = "runtime-embassy")]
impl<T: ?Sized> MutexLike<T> for alloc::sync::Arc<spin::Mutex<T>> {
    type Guard<'a>
        = spin::MutexGuard<'a, T>
    where
        Self: 'a,
        T: 'a;

    fn new(value: T) -> Self
    where
        T: Sized,
    {
        alloc::sync::Arc::new(spin::Mutex::new(value))
    }

    fn lock(&self) -> Self::Guard<'_> {
        self.as_ref().lock()
    }
}

#[cfg(feature = "runtime-embassy")]
pub struct EmbassyRuntime;

#[cfg(feature = "runtime-embassy")]
impl Runtime for EmbassyRuntime {
    type Mutex<T: ?Sized> = alloc::sync::Arc<spin::Mutex<T>>;
    type Timer = EmbassyTimer;
    type Instant = EmbassyInstant;
}

#[cfg(feature = "runtime-embassy")]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EmbassyInstant(embassy_time::Instant);

#[cfg(feature = "runtime-embassy")]
impl EmbassyInstant {
    pub fn from_embassy(instant: embassy_time::Instant) -> Self {
        Self(instant)
    }

    pub fn into_embassy(self) -> embassy_time::Instant {
        self.0
    }
}

#[cfg(feature = "runtime-embassy")]
fn to_embassy_duration(duration: core::time::Duration) -> embassy_time::Duration {
    embassy_time::Duration::from_micros(duration.as_micros() as u64)
}

#[cfg(feature = "runtime-embassy")]
fn to_core_duration(duration: embassy_time::Duration) -> core::time::Duration {
    core::time::Duration::from_micros(duration.as_micros())
}

#[cfg(feature = "runtime-embassy")]
impl core::ops::Add<core::time::Duration> for EmbassyInstant {
    type Output = EmbassyInstant;

    fn add(self, duration: core::time::Duration) -> Self::Output {
        EmbassyInstant(self.0 + to_embassy_duration(duration))
    }
}

#[cfg(feature = "runtime-embassy")]
impl core::ops::Sub<core::time::Duration> for EmbassyInstant {
    type Output = EmbassyInstant;

    fn sub(self, duration: core::time::Duration) -> Self::Output {
        EmbassyInstant(self.0 - to_embassy_duration(duration))
    }
}

#[cfg(feature = "runtime-embassy")]
impl core::ops::Sub<EmbassyInstant> for EmbassyInstant {
    type Output = core::time::Duration;

    fn sub(self, other: EmbassyInstant) -> Self::Output {
        to_core_duration(self.0 - other.0)
    }
}

#[cfg(feature = "runtime-embassy")]
impl core::ops::AddAssign<core::time::Duration> for EmbassyInstant {
    fn add_assign(&mut self, duration: core::time::Duration) {
        self.0 += to_embassy_duration(duration);
    }
}

#[cfg(feature = "runtime-embassy")]
impl core::ops::SubAssign<core::time::Duration> for EmbassyInstant {
    fn sub_assign(&mut self, duration: core::time::Duration) {
        self.0 -= to_embassy_duration(duration);
    }
}

#[cfg(feature = "runtime-embassy")]
#[derive(Clone, Debug)]
pub struct EmbassyTimer;

#[cfg(feature = "runtime-embassy")]
impl Timer for EmbassyTimer {
    type Sleep = embassy_time::Timer;
    type Instant = EmbassyInstant;

    fn sleep_future(&self, duration: core::time::Duration) -> Self::Sleep {
        embassy_time::Timer::after(to_embassy_duration(duration))
    }

    fn now(&self) -> Self::Instant {
        EmbassyInstant(embassy_time::Instant::now())
    }
}
