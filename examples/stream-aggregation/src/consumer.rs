// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Final consumer - consumes aggregated events from Queue 4

use crate::domain::AggregatedEvent;
use tokio::select;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

pub struct FinalConsumer {
    cancel_token: CancellationToken,
    task_handle: Option<JoinHandle<()>>,
}

impl FinalConsumer {
    /// Creates a new final consumer
    pub fn new(cancel_token: CancellationToken) -> Self {
        Self {
            cancel_token,
            task_handle: None,
        }
    }

    /// Starts the final consumer task
    pub fn start(&mut self, rx: mpsc::UnboundedReceiver<AggregatedEvent>) {
        let cancel_token = self.cancel_token.clone();
        let handle = tokio::spawn(async move {
            Self::run(rx, cancel_token).await;
        });
        self.task_handle = Some(handle);
    }

    /// Stops the final consumer task
    pub async fn stop(&mut self) {
        if let Some(handle) = self.task_handle.take() {
            self.cancel_token.cancel();
            let _ = handle.await;
        }
    }

    async fn run(
        mut rx: mpsc::UnboundedReceiver<AggregatedEvent>,
        cancel_token: CancellationToken,
    ) {
        println!("📥 Final consumer started\n");

        loop {
            select! {
                Some(event) = rx.recv() => {
                    let temp_display = event.temperature.map(|t| t as f64 / 10.0).unwrap_or(0.0);

                    println!(
                        "  [Consumer] seq {} - Temp: {:.1}°C, Alert: {}",
                        event.timestamp, temp_display, event.has_alert
                    );
                }
                _ = cancel_token.cancelled() => {
                    break;
                }
            }
        }

        println!("\n📥 Final consumer stopped");
    }
}
