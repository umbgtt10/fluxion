// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! System events producer - simulates Queue 3

use crate::domain::SystemEvent;
use async_channel::Sender;
use fluxion_core::CancellationToken;
use tokio::select;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};

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

    pub fn start(&mut self, tx: Sender<SystemEvent>) {
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

    async fn run(tx: Sender<SystemEvent>, cancel_token: CancellationToken) {
        let mut ticker = interval(Duration::from_millis(500));
        let mut timestamp = 200u64;

        println!("? Events producer started");

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

                    if tx.try_send(event.clone()).is_err() {
                        break;
                    }
                    println!("  [Producer<Events>] {} ({}) @ ts {}", event.event_type, event.severity, timestamp);
                }
                _ = cancel_token.cancelled() => {
                    break;
                }
            }
        }

        println!("? Events producer stopped");
    }
}
