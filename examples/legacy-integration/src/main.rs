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

use crate::adapters::{InventoryAdapter, OrderAdapter, UserAdapter};
use crate::domain::repository::Repository;
use crate::processing::business_logic;
use anyhow::Result;
use futures::future::ready;
use futures::StreamExt;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Legacy Integration Demo Starting...\n");

    // Cancellation token for graceful shutdown
    let cancel_token = CancellationToken::new();

    // Setup Ctrl+C handler
    let cancel_token_ctrlc = cancel_token.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
        println!("\n\n🛑 Ctrl+C received, shutting down gracefully...");
        cancel_token_ctrlc.cancel();
    });

    // Create and start adapters, getting streams directly
    println!("📊 Starting legacy data source adapters...");

    let mut user_adapter = UserAdapter::new();
    let mut order_adapter = OrderAdapter::new();
    let mut inventory_adapter = InventoryAdapter::new();

    let user_stream = user_adapter.start(cancel_token.clone());
    let order_stream = order_adapter.start(cancel_token.clone());
    let inventory_stream = inventory_adapter.start(cancel_token.clone());

    println!("🔄 Building timestamped streams...");

    // Split streams for repository and analytics using broadcast

    let (event_tx, _) = broadcast::channel(100);

    let event_tx_clone = event_tx.clone();
    let user_stream_broadcast = user_stream.map(move |e| {
        let _ = event_tx_clone.send(e.clone());
        e
    });

    let event_tx_clone = event_tx.clone();
    let order_stream_broadcast = order_stream.map(move |e| {
        let _ = event_tx_clone.send(e.clone());
        e
    });

    let event_tx_clone = event_tx.clone();
    let inventory_stream_broadcast = inventory_stream.map(move |e| {
        let _ = event_tx_clone.send(e.clone());
        e
    });

    // Aggregate using merge_with into a unified repository
    println!("🗄️  Building aggregated repository and analytics streams...\n");

    let aggregated_stream = Repository::new(
        user_stream_broadcast,
        order_stream_broadcast,
        inventory_stream_broadcast,
    )
    .create_stream();

    // Get analytics stream from broadcast
    let analytics_rx = event_tx.subscribe();
    let analytics_events = BroadcastStream::new(analytics_rx).filter_map(|r| ready(r.ok()));

    // Process business logic
    println!("💼 Processing business logic with analytics...\n");
    println!("Press Ctrl+C to stop...\n");
    println!("{}", "=".repeat(80));

    let business_logic_task = tokio::spawn(business_logic::process_events_with_analytics(
        aggregated_stream,
        analytics_events,
        cancel_token.clone(),
    ));

    // Wait for either business logic completion or cancellation
    tokio::select! {
        result = business_logic_task => {
            match result {
                Ok(Ok(())) => {
                    println!("\n{}", "=".repeat(80));
                    println!("✅ Demo completed successfully!");
                }
                Ok(Err(e)) => {
                    println!("\n{}", "=".repeat(80));
                    println!("❌ Business logic error: {}", e);
                }
                Err(e) => {
                    println!("\n{}", "=".repeat(80));
                    println!("❌ Task join error: {}", e);
                }
            }
        }
        _ = cancel_token.cancelled() => {
            println!("\n{}", "=".repeat(80));
            println!("🛑 Shutdown requested, cleaning up...");
            user_adapter.shutdown().await;
            order_adapter.shutdown().await;
            inventory_adapter.shutdown().await;
            println!("✅ Adapters shut down gracefully.");
        }
    }

    println!("✅ All tasks completed, exiting.");

    Ok(())
}
