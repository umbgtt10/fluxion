// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde::Deserialize;

/// Configuration for a single sensor
#[derive(Debug, Clone, Deserialize)]
pub struct SensorConfig {
    pub delay_min_ms: u64,
    pub delay_max_ms: u64,
    pub value_min: u32,
    pub value_max: u32,
}

/// Complete dashboard configuration
#[derive(Debug, Clone, Deserialize)]
pub struct DashboardConfig {
    pub sensor1: SensorConfig,
    pub sensor2: SensorConfig,
    pub sensor3: SensorConfig,
}

impl DashboardConfig {
    /// Load configuration from embedded TOML file
    pub fn load() -> Result<Self, toml::de::Error> {
        let config_str = include_str!("../config.toml");
        toml::from_str(config_str)
    }
}
