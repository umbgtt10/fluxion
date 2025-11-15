// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Sensor reading domain type

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SensorReading {
    pub timestamp: u64,
    pub sensor_id: String,
    pub temperature: i32, // Store as integer (e.g., temperature * 10)
}
