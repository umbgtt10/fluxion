// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Temperature sensor task implementation.

use crate::types::Temperature;
use embassy_time::{Duration, Timer};
use fluxion_core::CancellationToken;
use fluxion_stream_time::timer::Timer as TimerTrait;
use fluxion_stream_time::EmbassyTimerImpl;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

/// Temperature sensor task with debounce → map → filter pipeline
#[embassy_executor::task]
pub async fn temperature_sensor(tx: async_channel::Sender<Temperature>, cancel: CancellationToken) {
    println!("🌡️  Temperature sensor task started");

    let timer = EmbassyTimerImpl;
    let mut rng = ChaCha8Rng::seed_from_u64(12345);

    // Simulate sensor readings every 50ms
    loop {
        if cancel.is_cancelled() {
            break;
        }

        let temperature = Temperature {
            value_kelvin: rng.random_range(263..=313),
            timestamp: timer.now(),
        };

        println!("🌡️  Sensor: {}°C", temperature.value_kelvin);
        if tx.send(temperature).await.is_err() {
            println!("🌡️  Channel closed, stopping sensor");
            break;
        }

        let timeout = rng.random_range(100..=1000);
        println!("🌡️  timeout: {} ms", timeout);
        Timer::after(Duration::from_millis(timeout)).await;
    }

    println!("🌡️  Temperature sensor task stopped");
}
