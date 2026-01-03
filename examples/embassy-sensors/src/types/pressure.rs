// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use fluxion_core::{HasTimestamp, Timestamped};
use fluxion_stream_time::runtimes::EmbassyInstant;

/// Pressure reading in hectopascals (hPa)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pressure {
    pub value_hpa: u32,
    pub timestamp: EmbassyInstant,
}

impl HasTimestamp for Pressure {
    type Timestamp = EmbassyInstant;

    fn timestamp(&self) -> Self::Timestamp {
        self.timestamp
    }
}

impl Timestamped for Pressure {
    type Inner = Self;

    fn with_timestamp(inner: Self::Inner, timestamp: Self::Timestamp) -> Self {
        Self { timestamp, ..inner }
    }

    fn into_inner(self) -> Self::Inner {
        self
    }
}

impl core::fmt::Display for Pressure {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} hPa", self.value_hpa)
    }
}
