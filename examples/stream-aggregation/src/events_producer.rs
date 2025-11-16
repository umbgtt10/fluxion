// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! System events producer - simulates Queue 3

use crate::domain::SystemEvent;
use tokio::select;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};
use tokio_util::sync::CancellationToken;

pub struct EventsProducer {
    cancel_token: CancellationToken,
    task_handle: Option<JoinHandle<()>>,
}

impl EventsProducer {
    pub fn new(cancel_token: CancellationToken) -> Self {
        Self {
            cancel_token,
            task_handle: None,
        }
    }

    pub fn start(&mut self, tx: mpsc::UnboundedSender<SystemEvent>) {
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

    async fn run(tx: mpsc::UnboundedSender<SystemEvent>, cancel_token: CancellationToken) {
        let mut ticker = interval(Duration::from_millis(500));
        let mut timestamp = 200u64;

        println!("⚡ Events producer started");

        loop {
            select! {
                _ = ticker.tick() => {
                    timestamp += 1;

                    let event = SystemEvent {
                        timestamp,
                        event_type: if timestamp.is_multiple_of(3) {
                            "ALERT".to_string()
                        } else {
                            "INFO".to_string()
                        },
                        severity: "LOW".to_string(),
                    };

                    if tx.send(event.clone()).is_err() {
                        break;
                    }
                    println!("  [Producer<Events>] {} ({}) @ ts {}", event.event_type, event.severity, timestamp);
                }
                _ = cancel_token.cancelled() => {
                    break;
                }
            }
        }

        println!("⚡ Events producer stopped");
    }
}
