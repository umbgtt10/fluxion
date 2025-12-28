// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Sensor data types for the fusion system.

use fluxion_core::{HasTimestamp, Timestamped};
use fluxion_stream_time::runtimes::EmbassyInstant;

/// Temperature reading in degrees Celsius
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Temperature {
    pub value_kelvin: u32,
    pub timestamp: EmbassyInstant,
}

impl HasTimestamp for Temperature {
    type Timestamp = EmbassyInstant;

    fn timestamp(&self) -> Self::Timestamp {
        self.timestamp
    }
}

impl Timestamped for Temperature {
    type Inner = Self;

    fn with_timestamp(inner: Self::Inner, timestamp: Self::Timestamp) -> Self {
        Self { timestamp, ..inner }
    }

    fn into_inner(self) -> Self::Inner {
        self
    }
}

impl core::fmt::Display for Temperature {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} K", self.value_kelvin)
    }
}

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
