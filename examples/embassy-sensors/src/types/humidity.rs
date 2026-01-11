// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{HasTimestamp, Timestamped};
use fluxion_runtime::impls::embassy::EmbassyInstant;

/// Humidity reading as percentage (0-100%)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Humidity {
    pub value_percent: u32,
    pub timestamp: EmbassyInstant,
}

impl HasTimestamp for Humidity {
    type Timestamp = EmbassyInstant;

    fn timestamp(&self) -> Self::Timestamp {
        self.timestamp
    }
}

impl Timestamped for Humidity {
    type Inner = Self;

    fn with_timestamp(inner: Self::Inner, timestamp: Self::Timestamp) -> Self {
        Self { timestamp, ..inner }
    }

    fn into_inner(self) -> Self::Inner {
        self
    }
}

impl core::fmt::Display for Humidity {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} %", self.value_percent)
    }
}
