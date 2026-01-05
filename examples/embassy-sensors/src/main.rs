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

#![no_std]
#![no_main]

extern crate alloc;

mod aggregate;
mod fusion;
mod logging;
mod sensors;
mod time_driver;
mod types;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use fluxion_core::CancellationToken;
use fusion::fusion_task;
use sensors::{humidity_sensor, pressure_sensor, temperature_sensor};

// Required for panic handling
use panic_semihosting as _;

// Global allocator for heap allocations (64KB heap)
use embedded_alloc::LlffHeap as Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize heap (64KB)
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 64 * 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(core::ptr::addr_of_mut!(HEAP_MEM) as usize, HEAP_SIZE) }
    }

    info!("🚀 Embassy Sensor Fusion System Starting");
    info!("Runtime: 30 seconds");

    // Initialize Time Driver (SysTick) for QEMU
    let mut p = cortex_m::Peripherals::take().unwrap();
    time_driver::init(&mut p.SYST);
    info!("Time driver initialized");

    let cancel = CancellationToken::new();

    // Create channels for sensor fusion
    let (temp_tx, temp_rx) = async_channel::unbounded();
    let (pressure_tx, pressure_rx) = async_channel::unbounded();
    let (humidity_tx, humidity_rx) = async_channel::unbounded();

    // Spawn sensor tasks
    spawner.spawn(temperature_sensor(temp_tx, cancel.clone()).expect("Failed to spawn task"));
    spawner.spawn(pressure_sensor(pressure_tx, cancel.clone()).expect("Failed to spawn task"));
    spawner.spawn(humidity_sensor(humidity_tx, cancel.clone()).expect("Failed to spawn task"));

    // Spawn fusion task
    spawner.spawn(
        fusion_task(temp_rx, pressure_rx, humidity_rx, cancel.clone())
            .expect("Failed to spawn task"),
    );

    // Run for 30 seconds
    Timer::after(Duration::from_secs(30)).await;

    info!(" Timeout reached - initiating shutdown");
    cancel.cancel();

    // Wait for graceful shutdown
    Timer::after(Duration::from_millis(500)).await;
    info!(" System shutdown complete");

    // Exit QEMU
    use cortex_m_semihosting::debug::{self, EXIT_SUCCESS};
    debug::exit(EXIT_SUCCESS);
}
