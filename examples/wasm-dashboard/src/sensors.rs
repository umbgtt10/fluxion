// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

/// Simulated sensor data generators
#[allow(dead_code)]
pub struct SensorSimulator {
    pub sensor1_hz: f64,
    pub sensor2_hz: f64,
    pub sensor3_hz: f64,
    pub debounce_enabled: bool,
    pub throttle_enabled: bool,
    pub delay_enabled: bool,
    pub sample_enabled: bool,
    pub timeout_enabled: bool,
}

#[allow(dead_code)]
impl SensorSimulator {
    pub fn new() -> Self {
        Self {
            sensor1_hz: 100.0,
            sensor2_hz: 50.0,
            sensor3_hz: 25.0,
            debounce_enabled: false,
            throttle_enabled: false,
            delay_enabled: false,
            sample_enabled: false,
            timeout_enabled: false,
        }
    }

    pub fn toggle_debounce(&mut self) {
        self.debounce_enabled = !self.debounce_enabled;
        web_sys::console::log_1(
            &format!(
                "Debounce: {}",
                if self.debounce_enabled { "ON" } else { "OFF" }
            )
            .into(),
        );
    }

    pub fn toggle_throttle(&mut self) {
        self.throttle_enabled = !self.throttle_enabled;
        web_sys::console::log_1(
            &format!(
                "Throttle: {}",
                if self.throttle_enabled { "ON" } else { "OFF" }
            )
            .into(),
        );
    }

    pub fn toggle_delay(&mut self) {
        self.delay_enabled = !self.delay_enabled;
        web_sys::console::log_1(
            &format!("Delay: {}", if self.delay_enabled { "ON" } else { "OFF" }).into(),
        );
    }

    pub fn toggle_sample(&mut self) {
        self.sample_enabled = !self.sample_enabled;
        web_sys::console::log_1(
            &format!("Sample: {}", if self.sample_enabled { "ON" } else { "OFF" }).into(),
        );
    }

    pub fn toggle_timeout(&mut self) {
        self.timeout_enabled = !self.timeout_enabled;
        web_sys::console::log_1(
            &format!(
                "Timeout: {}",
                if self.timeout_enabled { "ON" } else { "OFF" }
            )
            .into(),
        );
    }
}

impl Default for SensorSimulator {
    fn default() -> Self {
        Self::new()
    }
}
