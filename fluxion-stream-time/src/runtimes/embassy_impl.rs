// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[cfg(feature = "runtime-embassy")]
pub mod embassy_implementation {
    use crate::timer::Timer;
    use core::ops::{Add, Sub};
    use core::time::Duration as CoreDuration;
    use embassy_time::{
        Duration as EmbassyDuration, Instant as RawEmbassyInstant, Timer as EmbassyTimer,
    };

    #[derive(Clone, Debug)]
    pub struct EmbassyTimerImpl;

    /// Wrapper around embassy_time::Instant that implements arithmetic with core::time::Duration
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct EmbassyInstant(RawEmbassyInstant);

    impl EmbassyInstant {
        pub fn from_embassy(instant: RawEmbassyInstant) -> Self {
            Self(instant)
        }

        pub fn into_embassy(self) -> RawEmbassyInstant {
            self.0
        }
    }

    // Convert core::time::Duration to embassy_time::Duration
    fn to_embassy_duration(duration: CoreDuration) -> EmbassyDuration {
        EmbassyDuration::from_micros(duration.as_micros() as u64)
    }

    // Convert embassy_time::Duration to core::time::Duration
    fn to_core_duration(duration: EmbassyDuration) -> CoreDuration {
        CoreDuration::from_micros(duration.as_micros())
    }

    impl Add<CoreDuration> for EmbassyInstant {
        type Output = EmbassyInstant;

        fn add(self, duration: CoreDuration) -> Self::Output {
            EmbassyInstant(self.0 + to_embassy_duration(duration))
        }
    }

    impl Sub<CoreDuration> for EmbassyInstant {
        type Output = EmbassyInstant;

        fn sub(self, duration: CoreDuration) -> Self::Output {
            EmbassyInstant(self.0 - to_embassy_duration(duration))
        }
    }

    impl Sub<EmbassyInstant> for EmbassyInstant {
        type Output = CoreDuration;

        fn sub(self, other: EmbassyInstant) -> Self::Output {
            to_core_duration(self.0 - other.0)
        }
    }

    impl Timer for EmbassyTimerImpl {
        type Sleep = EmbassyTimer;
        type Instant = EmbassyInstant;

        fn sleep_future(&self, duration: CoreDuration) -> Self::Sleep {
            EmbassyTimer::after(to_embassy_duration(duration))
        }

        fn now(&self) -> Self::Instant {
            EmbassyInstant(RawEmbassyInstant::now())
        }
    }
}
