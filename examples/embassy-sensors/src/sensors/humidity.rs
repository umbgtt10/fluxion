// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Humidity sensor task implementation.

use crate::info;
use crate::types::Humidity;
use embassy_time::Duration;
use fluxion_core::CancellationToken;
use fluxion_runtime::impls::embassy::EmbassyTimer;
use fluxion_runtime::timer::Timer;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

/// Humidity sensor task with sample → delay → take pipeline
#[embassy_executor::task]
pub async fn humidity_sensor(tx: async_channel::Sender<Humidity>, cancel: CancellationToken) {
    info!("Humidity sensor task started");

    let timer = EmbassyTimer;
    let mut rng = ChaCha8Rng::seed_from_u64(11111);

    // Simulate sensor readings every 20ms (fastest)
    loop {
        if cancel.is_cancelled() {
            break;
        }

        let humidity = Humidity {
            value_percent: rng.random_range(20..=80),
            timestamp: timer.now(),
        };

        info!("Sensor: {}%", humidity.value_percent);
        if tx.send(humidity).await.is_err() {
            info!("Channel closed, stopping sensor");
            break;
        }

        let timeout = rng.random_range(100..=1000);
        embassy_time::Timer::after(Duration::from_millis(timeout)).await;
    }

    info!("Humidity sensor task stopped");
}
