// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_rx::prelude::{HasTimestamp, Timestamped};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SensorReading {
    pub timestamp: u64,
    pub sensor_id: String,
    pub temperature: i32, // Store as integer (e.g., temperature * 10)
}

impl HasTimestamp for SensorReading {
    type Timestamp = u64;

    fn timestamp(&self) -> Self::Timestamp {
        self.timestamp
    }
}

impl Timestamped for SensorReading {
    type Inner = Self;

    fn with_timestamp(inner: Self::Inner, timestamp: Self::Timestamp) -> Self {
        Self { timestamp, ..inner }
    }

    fn into_inner(self) -> Self::Inner {
        self
    }
}
