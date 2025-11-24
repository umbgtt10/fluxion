// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Business logic processing using the aggregated repository state

use crate::domain::{events::UnifiedEvent, repository::OrderAnalytics, TimestampedEvent};
use anyhow::Result;
use fluxion_core::stream_item::StreamItem;
use futures::{Stream, StreamExt};
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

/// Business logic processor that handles event processing with analytics
pub struct EventProcessor {
    task_handle: Option<JoinHandle<Result<()>>>,
}

impl EventProcessor {
    /// Create a new EventProcessor and start processing events
    pub fn start(
        stream: impl Stream<Item = StreamItem<TimestampedEvent>> + Unpin + Send + 'static,
        cancel: CancellationToken,
    ) -> Self {
        let task_handle = tokio::spawn(Self::process_events(stream, cancel));
        Self {
            task_handle: Some(task_handle),
        }
    }

    /// Wait for the processor to complete and return the result
    pub async fn wait(mut self) -> Result<()> {
        if let Some(handle) = self.task_handle.take() {
            handle.await??;
        }
        Ok(())
    }

    /// Internal event processing loop
    async fn process_events(
        mut stream: impl Stream<Item = StreamItem<TimestampedEvent>> + Unpin,
        cancel: CancellationToken,
    ) -> Result<()> {
        let mut event_count = 0;
        let mut analytics = OrderAnalytics::default();

        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    Self::print_final_analytics(&analytics, event_count, true);
                    break;
                }
                stream_item = stream.next() => {
                    match stream_item {
                        Some(StreamItem::Value(timestamped_event)) => {
                            event_count += 1;
                            let event = &timestamped_event.event;

                            // Update analytics for order events
                            if let UnifiedEvent::OrderReceived(order) = event {
                                analytics.add_order(order);
                            }

                            Self::process_event(event, event_count, &analytics);

                            // Simulate some processing time
                            sleep(Duration::from_millis(100)).await;

                            // Stop after 20 events for demo purposes
                            if event_count >= 20 {
                                Self::print_final_analytics(&analytics, event_count, false);
                                break;
                            }
                        }
                        Some(StreamItem::Error(_)) => {
                            // Handle errors if needed
                        }
                        None => {
                            println!("\n📊 Stream ended");
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Process a single event and print appropriate output
    fn process_event(event: &UnifiedEvent, event_count: u32, analytics: &OrderAnalytics) {
        match event {
            UnifiedEvent::UserAdded(user) => {
                println!(
                    "✅ [{:04}] NEW USER: {} ({})",
                    event_count, user.name, user.email
                );
            }
            UnifiedEvent::OrderReceived(order) => {
                println!(
                    "📦 [{:04}] NEW ORDER: #{} - User {} wants {} units of Product #{}",
                    event_count, order.id, order.user_id, order.quantity, order.product_id
                );

                // Display aggregated analytics after each order
                println!(
                    "   📊 Analytics: {} total orders, {} total units ordered",
                    analytics.total_orders, analytics.total_quantity
                );

                // Show top ordered product
                if let Some((product_id, count)) = analytics
                    .orders_by_product
                    .iter()
                    .max_by_key(|(_, &count)| count)
                {
                    println!(
                        "   🏆 Most ordered product: #{} ({} orders)",
                        product_id, count
                    );
                }
            }
            UnifiedEvent::InventoryUpdated(inventory) => {
                println!(
                    "📊 [{:04}] INVENTORY UPDATE: {} - {} units available",
                    event_count, inventory.product_name, inventory.quantity
                );

                // Alert if inventory is low
                if inventory.quantity < 20 {
                    println!(
                        "⚠️  [{:04}]   LOW INVENTORY ALERT for {}!",
                        event_count, inventory.product_name
                    );
                }
            }
        }
    }

    /// Print final analytics summary
    fn print_final_analytics(analytics: &OrderAnalytics, event_count: u32, shutdown: bool) {
        if shutdown {
            println!("\n📊 SHUTDOWN - FINAL ANALYTICS:");
        } else {
            println!("\n📊 FINAL ANALYTICS:");
        }
        println!("   Total Orders: {}", analytics.total_orders);
        println!("   Total Units Ordered: {}", analytics.total_quantity);
        println!("   Unique Users: {}", analytics.orders_by_user.len());
        println!("   Products Ordered: {}", analytics.orders_by_product.len());

        if shutdown {
            println!("\n📊 Processed {} events before shutdown", event_count);
        } else {
            println!("\n📊 Processed {} events, stopping demo", event_count);
        }
    }
}
