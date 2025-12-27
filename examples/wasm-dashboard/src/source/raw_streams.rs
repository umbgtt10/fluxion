// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::sensor::Sensor;
use crate::config::DashboardConfig;
use fluxion_core::CancellationToken;

/// Container for the three raw sensors (Phase 1)
///
/// Each sensor generates values at random frequencies (1-5 Hz) within
/// specific ranges:
/// - Sensor 1: Values 1-9
/// - Sensor 2: Values 10-90
/// - Sensor 3: Values 100-900
///
/// These streams emit plain sensor values without timestamps.
pub struct Sensors {
    pub sensor1: Sensor,
    pub sensor2: Sensor,
    pub sensor3: Sensor,
}

impl Sensors {
    /// Creates three independent sensor streams with predefined ranges.
    /// Creates three independent sensor streams with configuration from file.
    ///
    /// # Arguments
    ///
    /// * `config` - Dashboard configuration with sensor parameters
    /// * `cancel_token` - Cancellation token to stop all sensors
    pub fn new(config: DashboardConfig, cancel_token: CancellationToken) -> Self {
        Self {
            sensor1: Sensor::new(
                (config.sensor1.delay_min_ms, config.sensor1.delay_max_ms),
                (config.sensor1.value_min, config.sensor1.value_max),
                cancel_token.clone(),
            ),
            sensor2: Sensor::new(
                (config.sensor2.delay_min_ms, config.sensor2.delay_max_ms),
                (config.sensor2.value_min, config.sensor2.value_max),
                cancel_token.clone(),
            ),
            sensor3: Sensor::new(
                (config.sensor3.delay_min_ms, config.sensor3.delay_max_ms),
                (config.sensor3.value_min, config.sensor3.value_max),
                cancel_token,
            ),
        }
    }
}
