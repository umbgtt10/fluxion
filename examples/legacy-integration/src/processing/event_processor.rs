// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Business logic processing using the aggregated repository state

use crate::domain::{events::UnifiedEvent, repository::OrderAnalytics, TimestampedEvent};
use crate::processing::event_handler::{print_final_analytics, process_event};
use anyhow::Result;
use fluxion_core::stream_item::StreamItem;
use fluxion_exec::subscribe_async::SubscribeAsyncExt;
use futures::Stream;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

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
        stream: impl Stream<Item = StreamItem<TimestampedEvent>> + Unpin + Send + 'static,
        cancel: CancellationToken,
    ) -> Self {
        let task_handle = tokio::spawn(Self::process_events(stream, cancel));
        Self { task_handle }
    }

    /// Internal event processing loop using subscribe_async
    async fn process_events(
        stream: impl Stream<Item = StreamItem<TimestampedEvent>> + Unpin + Send + 'static,
        cancel: CancellationToken,
    ) -> Result<()> {
        let event_count = Arc::new(AtomicU32::new(0));
        let analytics = Arc::new(Mutex::new(OrderAnalytics::default()));

        let result = stream
            .subscribe_async(
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
                Some(cancel.clone()),
                None::<fn(ProcessingError)>,
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
