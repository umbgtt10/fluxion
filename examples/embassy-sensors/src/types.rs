// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Sensor data types for the fusion system.

use fluxion_core::{HasTimestamp, Timestamped};
use fluxion_stream_time::runtimes::EmbassyInstant;

/// Temperature reading in degrees Celsius
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Temperature {
    pub value: i32, // Temperature * 100
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

/// Pressure reading in hectopascals (hPa)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pressure {
    pub value: i32, // Pressure * 100
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

/// Humidity reading as percentage (0-100%)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Humidity {
    pub value: i32, // Humidity * 100
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

/// Unified sensor reading enum for combining different sensor types
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SensorReading {
    Temperature(Temperature),
    Pressure(Pressure),
    Humidity(Humidity),
}

impl HasTimestamp for SensorReading {
    type Timestamp = EmbassyInstant;

    fn timestamp(&self) -> Self::Timestamp {
        match self {
            SensorReading::Temperature(t) => t.timestamp(),
            SensorReading::Pressure(p) => p.timestamp(),
            SensorReading::Humidity(h) => h.timestamp(),
        }
    }
}

impl Timestamped for SensorReading {
    type Inner = Self;

    fn with_timestamp(value: Self::Inner, timestamp: Self::Timestamp) -> Self {
        match value {
            SensorReading::Temperature(t) => {
                SensorReading::Temperature(Timestamped::with_timestamp(t, timestamp))
            }
            SensorReading::Pressure(p) => {
                SensorReading::Pressure(Timestamped::with_timestamp(p, timestamp))
            }
            SensorReading::Humidity(h) => {
                SensorReading::Humidity(Timestamped::with_timestamp(h, timestamp))
            }
        }
    }

    fn into_inner(self) -> Self::Inner {
        self
    }
}
