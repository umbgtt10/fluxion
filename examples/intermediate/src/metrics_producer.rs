// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Metrics data producer - simulates Queue 2

use crate::domain::MetricData;
use tokio::select;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};
use tokio_util::sync::CancellationToken;

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

    pub fn start(&mut self, tx: mpsc::UnboundedSender<MetricData>) {
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

    async fn run(tx: mpsc::UnboundedSender<MetricData>, cancel_token: CancellationToken) {
        let mut ticker = interval(Duration::from_millis(400));
        let mut timestamp = 100u64;

        println!("📊 Metrics producer started");

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

                    if tx.send(metric).is_err() {
                        break;
                    }
                    println!("  [Producer<Metrics>] {}% @ ts {}", value, timestamp);
                }
                _ = cancel_token.cancelled() => {
                    break;
                }
            }
        }

        println!("📊 Metrics producer stopped");
    }
}
