// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Business logic processing using the aggregated repository state

use crate::domain::{events::UnifiedEvent, repository::OrderAnalytics, TimestampedEvent};
use crate::processing::event_handler::{print_final_analytics, process_event};
use anyhow::Result;
use fluxion_core::{stream_item::StreamItem, CancellationToken};
use fluxion_exec::SubscribeExt;
use futures::lock::Mutex as FutureMutex;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::sleep;

#[derive(Debug, thiserror::Error)]
#[error("Processing error: {0}")]
struct ProcessingError(String);

/// Business logic processor that handles event processing with analytics
pub struct EventProcessor {
    pub task_handle: JoinHandle<Result<()>>,
}

impl EventProcessor {
    /// Create a new EventProcessor and start processing events
    pub fn start(
        stream: impl SubscribeExt<StreamItem<TimestampedEvent>> + Unpin + Send + 'static,
        cancel: CancellationToken,
    ) -> Self {
        let task_handle = tokio::spawn(Self::process_events(stream, cancel));
        Self { task_handle }
    }

    /// Internal event processing loop using subscribe
    async fn process_events(
        stream: impl SubscribeExt<StreamItem<TimestampedEvent>> + Unpin + Send + 'static,
        cancel: CancellationToken,
    ) -> Result<()> {
        let event_count = Arc::new(AtomicU32::new(0));
        let analytics = Arc::new(FutureMutex::new(OrderAnalytics::default()));

        let result = stream
            .subscribe(
                {
                    let event_count = event_count.clone();
                    let analytics = analytics.clone();
                    move |stream_item: StreamItem<TimestampedEvent>, cancel: CancellationToken| {
                        let event_count = event_count.clone();
                        let analytics = analytics.clone();
                        async move {
                            // Check for cancellation
                            if cancel.is_cancelled() {
                                return Err(ProcessingError("Cancelled".to_string()));
                            }

                            match stream_item {
                                StreamItem::Value(timestamped_event) => {
                                    let count = event_count.fetch_add(1, Ordering::SeqCst) + 1;
                                    let event = &timestamped_event.event;

                                    // Update analytics for order events
                                    if let UnifiedEvent::OrderReceived(order) = event {
                                        analytics.lock().await.add_order(order);
                                    }

                                    let analytics_snapshot = analytics.lock().await.clone();
                                    process_event(event, count, &analytics_snapshot);

                                    // Simulate some processing time
                                    sleep(Duration::from_millis(100)).await;
                                }
                                StreamItem::Error(_) => {
                                    // Handle errors if needed
                                }
                            }

                            Ok(())
                        }
                    }
                },
                |_| {},
                Some(cancel.clone()),
            )
            .await;

        // Handle the result
        let count = event_count.load(Ordering::SeqCst);
        let analytics_final = analytics.lock().await.clone();

        match result {
            Ok(()) => {
                println!("\n📊 Stream ended");
                print_final_analytics(&analytics_final, count, false);
                Ok(())
            }
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("Cancelled") {
                    print_final_analytics(&analytics_final, count, true);
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Processing failed: {}", error_msg))
                }
            }
        }
    }
}
