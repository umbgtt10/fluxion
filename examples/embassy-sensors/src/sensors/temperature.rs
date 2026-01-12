// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Temperature sensor task implementation.

use crate::info;
use crate::types::Temperature;
use embassy_time::Duration;
use fluxion_core::CancellationToken;
use fluxion_runtime::impls::embassy::EmbassyTimer;
use fluxion_runtime::timer::Timer;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

/// Temperature sensor task with debounce → map → filter pipeline
#[embassy_executor::task]
pub async fn temperature_sensor(tx: async_channel::Sender<Temperature>, cancel: CancellationToken) {
    info!("Temperature sensor task started");

    let timer = EmbassyTimer;
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

        info!("Sensor: {} C", temperature.value_kelvin);
        if tx.send(temperature).await.is_err() {
            info!("Channel closed, stopping sensor");
            break;
        }

        let timeout = rng.random_range(100..=1000);
        embassy_time::Timer::after(Duration::from_millis(timeout)).await;
    }

    info!("Temperature sensor task stopped");
}
