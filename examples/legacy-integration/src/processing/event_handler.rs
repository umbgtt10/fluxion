// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Event handling and display logic

use crate::domain::{events::UnifiedEvent, repository::OrderAnalytics};

/// Process a single event and print appropriate output
pub fn process_event(event: &UnifiedEvent, event_count: u32, analytics: &OrderAnalytics) {
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
pub fn print_final_analytics(analytics: &OrderAnalytics, event_count: u32, shutdown: bool) {
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
