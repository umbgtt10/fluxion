// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::types::{Humidity, Pressure, Temperature};

/// Aggregated state of all sensor readings
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SensorAggregate {
    pub latest_temp: Option<Temperature>,
    pub latest_pressure: Option<Pressure>,
    pub latest_humidity: Option<Humidity>,
    pub humidity_delta: i32,
    pub update_count: u32,
}

impl SensorAggregate {
    pub fn new() -> Self {
        Self {
            latest_temp: None,
            latest_pressure: None,
            latest_humidity: None,
            humidity_delta: 0,
            update_count: 0,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.latest_temp.is_some()
            && self.latest_pressure.is_some()
            && self.latest_humidity.is_some()
    }
}

impl core::fmt::Display for SensorAggregate {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.is_complete() {
            write!(
                f,
                "Temp: {}K | Press: {}hPa | Hum: {}% | Delta: {} | Upd: {}",
                self.latest_temp.unwrap().value_kelvin,
                self.latest_pressure.unwrap().value_hpa,
                self.latest_humidity.unwrap().value_percent,
                self.humidity_delta,
                self.update_count
            )
        } else {
            write!(f, "Incomplete (Updates: {})", self.update_count)
        }
    }
}
