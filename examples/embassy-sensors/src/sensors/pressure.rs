// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Pressure sensor task implementation.

use crate::info;
use crate::types::Pressure;
use embassy_time::Duration;
use fluxion_core::CancellationToken;
use fluxion_runtime::impls::embassy::EmbassyTimer;
use fluxion_runtime::timer::Timer;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

/// Pressure sensor task with throttle → scan → distinct_until_changed pipeline
#[embassy_executor::task]
pub async fn pressure_sensor(tx: async_channel::Sender<Pressure>, cancel: CancellationToken) {
    info!("Pressure sensor task started");

    let timer = EmbassyTimer;
    let mut rng = ChaCha8Rng::seed_from_u64(67890);

    // Simulate sensor readings every 30ms (faster than temperature)
    loop {
        if cancel.is_cancelled() {
            break;
        }

        let pressure = Pressure {
            value_hpa: rng.random_range(950..=1050),
            timestamp: timer.now(),
        };

        info!("Sensor: {} hPa", pressure.value_hpa);
        if tx.send(pressure).await.is_err() {
            info!("Channel closed, stopping sensor");
            break;
        }

        let timeout = rng.random_range(100..=1000);
        embassy_time::Timer::after(Duration::from_millis(timeout)).await;
    }

    info!("Pressure sensor task stopped");
}
