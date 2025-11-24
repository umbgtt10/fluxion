// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Business logic processing using the aggregated repository state

use std::time::Duration;

use anyhow::Result;
use fluxion_core::stream_item::StreamItem;
use futures::{Stream, StreamExt};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::domain::{events::UnifiedEvent, repository::OrderAnalytics, TimestampedEvent};

pub async fn process_events_with_analytics(
    mut stream: impl Stream<Item = StreamItem<TimestampedEvent>> + Unpin,
    _analytics_stream: impl Stream<Item = TimestampedEvent> + Unpin,
    cancel: CancellationToken,
) -> Result<()> {
    let mut event_count = 0;
    let mut analytics = OrderAnalytics::default();

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                println!("\n📊 SHUTDOWN - FINAL ANALYTICS:");
                println!("   Total Orders: {}", analytics.total_orders);
                println!("   Total Units Ordered: {}", analytics.total_quantity);
                println!("   Unique Users: {}", analytics.orders_by_user.len());
                println!("   Products Ordered: {}", analytics.orders_by_product.len());
                println!("\n📊 Processed {} events before shutdown", event_count);
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

                        // Simulate some processing time
                        sleep(Duration::from_millis(100)).await;

                        // Stop after 20 events for demo purposes
                        if event_count >= 20 {
                            println!("\n📊 FINAL ANALYTICS:");
                            println!("   Total Orders: {}", analytics.total_orders);
                            println!("   Total Units Ordered: {}", analytics.total_quantity);
                            println!("   Unique Users: {}", analytics.orders_by_user.len());
                            println!("   Products Ordered: {}", analytics.orders_by_product.len());
                            println!("\n📊 Processed {} events, stopping demo", event_count);
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

#[allow(dead_code)]
pub async fn process_events(
    mut stream: impl Stream<Item = StreamItem<TimestampedEvent>> + Unpin,
    _cancel: CancellationToken,
) -> Result<()> {
    let mut event_count = 0;

    while let Some(stream_item) = stream.next().await {
        if let StreamItem::Value(timestamped_event) = stream_item {
            event_count += 1;
            let event = &timestamped_event.event;

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

                    // TODO: Check if we have sufficient inventory
                    // TODO: Check if user exists
                    // TODO: Update order status
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

            // Simulate some processing time
            sleep(Duration::from_millis(100)).await;

            // Stop after 20 events for demo purposes
            if event_count >= 20 {
                println!("\n📊 Processed {} events, stopping demo", event_count);
                break;
            }
        }
    }

    Ok(())
}
