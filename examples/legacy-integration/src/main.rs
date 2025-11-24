// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Legacy System Integration Demo
//!
//! Demonstrates integrating 3 legacy data sources using Fluxion's wrapper pattern:
//! - Legacy Database (JSON) -> User records
//! - Legacy Message Queue (XML) -> Order events
//! - Legacy File Watcher (CSV) -> Inventory updates
//!
//! All sources lack intrinsic timestamps, so adapters add them at the boundary.
//! Data is aggregated using `merge_with` into a unified repository.

mod adapters;
mod domain;
mod legacy;
mod processing;

use crate::adapters::{inventory_adapter, order_adapter, user_adapter};
use crate::domain::repository::build_repository_stream;
use crate::legacy::{
    database::LegacyDatabase, file_watcher::LegacyFileWatcher, message_queue::LegacyMessageQueue,
};
use crate::processing::business_logic;
use anyhow::Result;
use tokio::spawn;
use tokio::sync::mpsc::unbounded_channel;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Legacy Integration Demo Starting...\n");

    // Cancellation token for graceful shutdown
    let cancel_token = CancellationToken::new();

    // Channels for legacy data sources (unwrapped, no timestamps)
    let (user_tx, user_rx) = unbounded_channel();
    let (order_tx, order_rx) = unbounded_channel();
    let (inventory_tx, inventory_rx) = unbounded_channel();

    // Spawn legacy data source simulators
    println!("📊 Starting legacy data sources...");

    let db = LegacyDatabase::new();

    spawn(db.poll_users(user_tx, cancel_token.clone()));
    let cancel_token_orders = cancel_token.clone();
    spawn(async move {
        let mq = LegacyMessageQueue::new();
        mq.consume_orders(order_tx, cancel_token_orders).await;
    });
    let cancel_token_inventory = cancel_token.clone();
    spawn(async move {
        let fw = LegacyFileWatcher::new();
        fw.watch_inventory(inventory_tx, cancel_token_inventory)
            .await;
    });

    // Wrap legacy sources with timestamps using adapters
    println!("🔄 Creating timestamped adapters...");

    let user_stream = user_adapter::wrap_users(user_rx);
    let order_stream = order_adapter::wrap_orders(order_rx);
    let inventory_stream = inventory_adapter::wrap_inventory(inventory_rx);

    // Aggregate using merge_with into a unified repository
    println!("🗄️  Building aggregated repository stream...\n");

    let aggregated_stream = build_repository_stream(user_stream, order_stream, inventory_stream);

    // Process business logic
    println!("💼 Processing business logic...\n");
    println!("Press Ctrl+C to stop...\n");
    println!("{}", "=".repeat(80));

    business_logic::process_events(aggregated_stream, cancel_token.clone()).await?;

    println!("\n{}", "=".repeat(80));
    println!("✅ Demo completed successfully!");

    Ok(())
}
