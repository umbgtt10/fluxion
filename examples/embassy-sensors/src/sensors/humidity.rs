// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Humidity sensor task implementation.

use crate::types::Humidity;
// Using println! for logging
use embassy_time::{Duration, Timer};
use fluxion_core::CancellationToken;
use fluxion_stream_time::timer::Timer as TimerTrait;
use fluxion_stream_time::EmbassyTimerImpl;
use rand::Rng;

/// Humidity sensor task with sample → delay → take pipeline
#[embassy_executor::task]
pub async fn humidity_sensor(tx: async_channel::Sender<Humidity>, cancel: CancellationToken) {
    println!("💧 Humidity sensor task started");

    let timer = EmbassyTimerImpl;
    let mut rng = rand::rng();

    // Simulate sensor readings every 20ms (fastest)
    loop {
        if cancel.is_cancelled() {
            break;
        }

        let humidity = Humidity {
            value_percent: rng.random_range(20..=80),
            timestamp: timer.now(),
        };

        println!("💧 Sensor: {}%", humidity.value_percent);
        if tx.send(humidity).await.is_err() {
            println!("💧 Channel closed, stopping sensor");
            break;
        }

        let timeout = rng.random_range(100..=1000);
        println!("🌡️  timeout: {} ms", timeout);
        Timer::after(Duration::from_millis(timeout)).await;
    }

    println!("💧 Humidity sensor task stopped");
}
