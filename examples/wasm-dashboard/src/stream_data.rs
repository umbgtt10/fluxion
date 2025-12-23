// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream_time::runtimes::wasm_implementation::WasmTimer;
use fluxion_stream_time::timer::Timer;
use fluxion_stream_time::InstantTimestamped;
use futures::channel::mpsc;
use gloo_timers::future::TimeoutFuture;

type WasmTimestamped<T> = InstantTimestamped<T, WasmTimer>;

/// Sensor data value
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct SensorValue {
    pub sensor_id: u8,
    pub value: f64,
    pub timestamp_ms: u64,
}

// Implement Eq and Ord for CombinedState compatibility
impl Eq for SensorValue {}

impl Ord for SensorValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Compare by timestamp, then sensor_id
        self.timestamp_ms
            .cmp(&other.timestamp_ms)
            .then_with(|| self.sensor_id.cmp(&other.sensor_id))
    }
}

/// Generate sensor data at specified frequency
pub async fn generate_sensor_stream(
    sensor_id: u8,
    hz: f64,
    tx: mpsc::UnboundedSender<WasmTimestamped<SensorValue>>,
    running: std::rc::Rc<std::cell::RefCell<bool>>,
) {
    web_sys::console::log_1(&format!("📊 Sensor {} ENTRY", sensor_id).into());

    let interval_ms = (1000.0 / hz) as u32;
    web_sys::console::log_1(&format!("📊 Sensor {} interval: {}ms", sensor_id, interval_ms).into());

    let timer = WasmTimer::new();
    web_sys::console::log_1(&format!("📊 Sensor {} timer created", sensor_id).into());

    let mut counter = 0u64;
    web_sys::console::log_1(&format!("📊 Sensor {} counter initialized", sensor_id).into());

    web_sys::console::log_1(&format!("📊 Sensor {} started at {}Hz", sensor_id, hz).into());

    // Test: Can we even execute past the first log?
    for i in 0..3 {
        web_sys::console::log_1(&format!("🔍 Sensor {} test log {}", sensor_id, i).into());
    }

    web_sys::console::log_1(
        &format!("🔍 Sensor {} about to check running flag...", sensor_id).into(),
    );

    let mut send_count = 0;
    web_sys::console::log_1(&format!("🔍 Sensor {} created send_count", sensor_id).into());
    let is_running = *running.borrow();
    web_sys::console::log_1(
        &format!(
            "🔍 Sensor {} borrowed running, value: {}",
            sensor_id, is_running
        )
        .into(),
    );
    web_sys::console::log_1(
        &format!("🔍 Sensor {} running flag: {}", sensor_id, is_running).into(),
    );

    while *running.borrow() {
        // Generate simulated sensor value (sine wave with some variance)
        let t = counter as f64 * 0.1;
        let base_value = ((t + sensor_id as f64).sin() + 1.0) / 2.0; // 0.0 to 1.0
        let noise = (js_sys::Math::random() - 0.5) * 0.1;
        let value = (base_value + noise).clamp(0.0, 1.0);

        let sensor_value = SensorValue {
            sensor_id,
            value,
            timestamp_ms: js_sys::Date::now() as u64,
        };

        // Send timestamped value
        let timestamped = WasmTimestamped::new(sensor_value, timer.now());
        if tx.unbounded_send(timestamped).is_err() {
            web_sys::console::log_1(&format!("⚠️ Sensor {} stream closed", sensor_id).into());
            break;
        }

        send_count += 1;
        if send_count % 50 == 0 {
            web_sys::console::log_1(
                &format!("📈 Sensor {} sent {} values", sensor_id, send_count).into(),
            );
        }

        counter += 1;
        TimeoutFuture::new(interval_ms).await;
    }
}

/// Combined sensor data from all sensors
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct CombinedSensorData {
    pub sensor1: f64,
    pub sensor2: f64,
    pub sensor3: f64,
    pub timestamp_ms: u64,
}
