// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Embassy Sensor Fusion Example
//!
//! Demonstrates Fluxion reactive streams on embedded systems with Embassy runtime.
//!
//! # Architecture
//!
//! Three concurrent sensor tasks, each with a reactive processing pipeline:
//! - Temperature: debounce → map → filter
//! - Pressure: throttle → scan → distinct_until_changed
//! - Humidity: sample → delay → take
//!
//! Streams are fused with combine_latest, filtered for alert conditions,
//! and logged via defmt.
//!
//! # Features Demonstrated
//!
//! - ✅ Multi-task Embassy spawning
//! - ✅ All 5 time operators (debounce, throttle, sample, delay, timeout implied via cancel)
//! - ✅ Stream transformations (map, filter, scan, distinct_until_changed, take)
//! - ✅ Sensor fusion with combine_latest
//! - ✅ Graceful shutdown with CancellationToken
//! - ✅ defmt logging for embedded
//!
//! # Runtime
//!
//! Uses Embassy executor with arch-std for demonstration. In production,
//! replace with embassy-executor hardware-specific features (e.g., embassy-stm32).

mod aggregate;
mod fusion;
mod sensors;
mod types;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use fluxion_core::CancellationToken;
use fusion::fusion_task;
use sensors::{humidity_sensor, pressure_sensor, temperature_sensor};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    println!("🚀 Embassy Sensor Fusion System Starting");
    println!("Runtime: 30 seconds");

    let cancel = CancellationToken::new();

    // Create channels for sensor fusion
    let (temp_tx, temp_rx) = async_channel::unbounded();
    let (pressure_tx, pressure_rx) = async_channel::unbounded();
    let (humidity_tx, humidity_rx) = async_channel::unbounded();

    // Spawn sensor tasks
    spawner
        .spawn(temperature_sensor(temp_tx, cancel.clone()))
        .unwrap();
    spawner
        .spawn(pressure_sensor(pressure_tx, cancel.clone()))
        .unwrap();
    spawner
        .spawn(humidity_sensor(humidity_tx, cancel.clone()))
        .unwrap();

    // Spawn fusion task
    spawner
        .spawn(fusion_task(
            temp_rx,
            pressure_rx,
            humidity_rx,
            cancel.clone(),
        ))
        .unwrap();

    // Run for 30 seconds
    Timer::after(Duration::from_secs(30)).await;

    println!("⏱️  Timeout reached - initiating shutdown");
    cancel.cancel();

    // Wait for graceful shutdown
    Timer::after(Duration::from_millis(500)).await;
    println!("✅ System shutdown complete");

    // Exit the process (Embassy executor doesn't stop automatically)
    std::process::exit(0);
}

// Note: For actual embedded hardware, you would need:
// - Proper panic handler for your target
// - Embassy HAL for your specific MCU (e.g., embassy-stm32)
// - Hardware-specific timer driver
// - Real sensor drivers (I2C, SPI, etc.)
