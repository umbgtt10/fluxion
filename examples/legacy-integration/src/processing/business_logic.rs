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

use crate::domain::{events::UnifiedEvent, TimestampedEvent};

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
