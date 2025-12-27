// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Temperature sensor task implementation.

use crate::types::Temperature;
use embassy_time::{Duration, Timer};
use fluxion_core::CancellationToken;
use fluxion_stream_time::timer::Timer as TimerTrait;
use fluxion_stream_time::EmbassyTimerImpl;
use rand::Rng;

/// Temperature sensor task with debounce → map → filter pipeline
#[embassy_executor::task]
pub async fn temperature_sensor(tx: async_channel::Sender<Temperature>, cancel: CancellationToken) {
    println!("🌡️  Temperature sensor task started");

    let timer = EmbassyTimerImpl;
    let mut rng = rand::rng();

    // Simulate sensor readings every 50ms
    loop {
        if cancel.is_cancelled() {
            break;
        }

        let temperature = Temperature {
            value: (20.0 + rng.random::<f32>() * 15.0 * 100.0) as i32,
            timestamp: timer.now(),
        };

        println!("🌡️  Sensor: {}°C", temperature.value);
        if tx.send(temperature).await.is_err() {
            println!("🌡️  Channel closed, stopping sensor");
            break;
        }

        Timer::after(Duration::from_millis(50)).await;
    }

    println!("🌡️  Temperature sensor task stopped");
}
