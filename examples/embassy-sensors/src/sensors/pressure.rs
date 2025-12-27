// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Pressure sensor task implementation.

use crate::types::Pressure;
use embassy_time::{Duration, Timer};
use fluxion_core::CancellationToken;
use fluxion_stream_time::timer::Timer as TimerTrait;
use fluxion_stream_time::EmbassyTimerImpl;
use rand::Rng;

/// Pressure sensor task with throttle → scan → distinct_until_changed pipeline
#[embassy_executor::task]
pub async fn pressure_sensor(tx: async_channel::Sender<Pressure>, cancel: CancellationToken) {
    println!("📊 Pressure sensor task started");

    let timer = EmbassyTimerImpl;
    let mut rng = rand::rng();

    // Simulate sensor readings every 30ms (faster than temperature)
    loop {
        if cancel.is_cancelled() {
            break;
        }

        let pressure = Pressure {
            value: (1000.0 + rng.random::<f32>() * 20.0) as i32,
            timestamp: timer.now(),
        };

        println!("📊 Sensor: {} hPa", pressure.value);
        if tx.send(pressure).await.is_err() {
            println!("📊 Channel closed, stopping sensor");
            break;
        }

        Timer::after(Duration::from_millis(30)).await;
    }

    println!("📊 Pressure sensor task stopped");
}
