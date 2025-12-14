// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Sensor reading producer - simulates Queue 1

use crate::domain::SensorReading;
use tokio::select;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};
use tokio_util::sync::CancellationToken;

pub struct SensorProducer {
    cancel_token: CancellationToken,
    task_handle: Option<JoinHandle<()>>,
}

impl SensorProducer {
    pub fn new(cancel_token: CancellationToken) -> Self {
        Self {
            cancel_token,
            task_handle: None,
        }
    }

    pub fn start(&mut self, tx: mpsc::UnboundedSender<SensorReading>) {
        let cancel_token = self.cancel_token.clone();
        let handle = tokio::spawn(async move {
            Self::run(tx, cancel_token).await;
        });
        self.task_handle = Some(handle);
    }

    pub async fn stop(&mut self) {
        if let Some(handle) = self.task_handle.take() {
            self.cancel_token.cancel();
            let _ = handle.await;
        }
    }

    async fn run(tx: mpsc::UnboundedSender<SensorReading>, cancel_token: CancellationToken) {
        let mut ticker = interval(Duration::from_millis(300));
        let mut timestamp = 0u64;

        println!("🌡️  Sensor producer started");

        loop {
            select! {
                _ = ticker.tick() => {
                    timestamp += 1;

                    let temp_float = 20.0 + (timestamp % 10) as f64;
                    let reading = SensorReading {
                        timestamp,
                        sensor_id: "TEMP-001".to_string(),
                        temperature: (temp_float * 10.0) as i32,
                    };

                    if tx.send(reading).is_err() {
                        break;
                    }
                    println!("  [Producer<Sensor>] {:.1}°C @ ts {}", temp_float, timestamp);
                }
                _ = cancel_token.cancelled() => {
                    break;
                }
            }
        }

        println!("🌡️  Sensor producer stopped");
    }
}
