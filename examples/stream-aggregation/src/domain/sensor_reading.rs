// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Sensor reading domain type

use fluxion::prelude::Ordered;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SensorReading {
    pub timestamp: u64,
    pub sensor_id: String,
    pub temperature: i32, // Store as integer (e.g., temperature * 10)
}

impl Ordered for SensorReading {
    type Inner = Self;

    fn get(&self) -> &Self::Inner {
        self
    }

    fn order(&self) -> u64 {
        self.timestamp
    }

    fn with_order(inner: Self::Inner, order: u64) -> Self {
        Self {
            timestamp: order,
            ..inner
        }
    }
}
