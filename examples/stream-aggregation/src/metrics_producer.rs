// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Metrics data producer - simulates Queue 2

use crate::domain::MetricData;
use async_channel::Sender;
use fluxion_core::CancellationToken;
use tokio::select;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};

pub struct MetricsProducer {
    cancel_token: CancellationToken,
    task_handle: Option<JoinHandle<()>>,
}

impl MetricsProducer {
    pub fn new(cancel_token: CancellationToken) -> Self {
        Self {
            cancel_token,
            task_handle: None,
        }
    }

    pub fn start(&mut self, tx: Sender<MetricData>) {
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

    async fn run(tx: Sender<MetricData>, cancel_token: CancellationToken) {
        let mut ticker = interval(Duration::from_millis(400));
        let mut timestamp = 100u64;

        println!("?? Metrics producer started");

        loop {
            select! {
                _ = ticker.tick() => {
                    timestamp += 1;

                    let value = 30 + (timestamp % 50);
                    let metric = MetricData {
                        timestamp,
                        metric_name: "cpu_usage".to_string(),
                        value,
                    };

                    if tx.try_send(metric).is_err() {
                        break;
                    }
                    println!("  [Producer<Metrics>] {}% @ ts {}", value, timestamp);
                }
                _ = cancel_token.cancelled() => {
                    break;
                }
            }
        }

        println!("?? Metrics producer stopped");
    }
}
