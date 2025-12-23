// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Raw sensor data generation without timestamping

use crate::raw_sensor_value::RawSensorValue;
use futures::channel::mpsc;
use gloo_timers::future::TimeoutFuture;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;

/// Sensor with its own data generation task and graceful shutdown
pub struct SensorValueSource {
    sensor_id: u8,
    frequency_hz: f64,
    running: Rc<RefCell<bool>>,
}

impl SensorValueSource {
    /// Create a new sensor
    pub fn new(sensor_id: u8, frequency_hz: f64) -> Self {
        Self {
            sensor_id,
            frequency_hz,
            running: Rc::new(RefCell::new(false)),
        }
    }

    /// Start the sensor task and return a receiver for the data stream
    pub fn start(&self) -> mpsc::UnboundedReceiver<RawSensorValue> {
        let (tx, rx) = mpsc::unbounded();

        *self.running.borrow_mut() = true;

        let sensor_id = self.sensor_id;
        let frequency_hz = self.frequency_hz;
        let running = self.running.clone();

        spawn_local(async move {
            generate_sensor_data(sensor_id, frequency_hz, tx, running).await;
        });

        web_sys::console::log_1(
            &format!(
                "✅ Sensor {} started at {}Hz",
                self.sensor_id, self.frequency_hz
            )
            .into(),
        );

        rx
    }

    /// Stop the sensor gracefully
    pub fn stop(&self) {
        *self.running.borrow_mut() = false;
        web_sys::console::log_1(&format!("🛑 Sensor {} stop requested", self.sensor_id).into());
    }

    /// Check if sensor is running
    pub fn is_running(&self) -> bool {
        *self.running.borrow()
    }
}

/// Internal: Generate raw sensor data at specified frequency
async fn generate_sensor_data(
    sensor_id: u8,
    hz: f64,
    tx: mpsc::UnboundedSender<RawSensorValue>,
    running: Rc<RefCell<bool>>,
) {
    let interval_ms = (1000.0 / hz) as u32;
    let mut counter = 0u64;

    web_sys::console::log_1(&format!("📡 Raw sensor {} started at {}Hz", sensor_id, hz).into());

    while *running.borrow() {
        // Generate simulated sensor value (sine wave with noise)
        let t = counter as f64 * 0.1;
        let base_value = ((t + sensor_id as f64).sin() + 1.0) / 2.0; // 0.0 to 1.0
        let noise = (js_sys::Math::random() - 0.5) * 0.1;
        let value = (base_value + noise).clamp(0.0, 1.0);

        let sensor_value = RawSensorValue { sensor_id, value };

        if tx.unbounded_send(sensor_value).is_err() {
            web_sys::console::log_1(&format!("⚠️ Sensor {} channel closed", sensor_id).into());
            break;
        }

        counter += 1;
        TimeoutFuture::new(interval_ms).await;
    }

    web_sys::console::log_1(&format!("🛑 Sensor {} stopped", sensor_id).into());
}
