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

use std::time::Duration;

use crate::adapters::{InventoryAdapter, OrderAdapter, UserAdapter};
use crate::domain::repository::Repository;
use crate::processing::event_processor::EventProcessor;
use anyhow::Result;
use tokio::time::sleep;
use tokio::{select, signal};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Legacy Integration Demo Starting...\n");

    // Cancellation token for graceful shutdown
    let cancel_token = CancellationToken::new();

    // Create and start adapters, getting streams directly
    println!("📊 Starting legacy data source adapters...");

    let mut user_adapter = UserAdapter::new();
    let mut order_adapter = OrderAdapter::new();
    let mut inventory_adapter = InventoryAdapter::new();

    let user_stream = user_adapter.start(cancel_token.clone());
    let order_stream = order_adapter.start(cancel_token.clone());
    let inventory_stream = inventory_adapter.start(cancel_token.clone());

    let aggregated_stream =
        Repository::new(user_stream, order_stream, inventory_stream).create_stream();

    // Process business logic
    println!("Demo will run for 20 seconds or press Ctrl+C to stop...\n");

    let mut event_processor = EventProcessor::start(aggregated_stream, cancel_token.clone());

    select! {
        _ = signal::ctrl_c() => {
            println!("\n\n🛑 Ctrl+C received, shutting down gracefully...");
            cancel_token.cancel();
            // Wait for processor to finish
            match event_processor.task_handle.await {
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
        _ = sleep(Duration::from_secs(20)) => {
            println!("\n\n⏱️  20 seconds elapsed, shutting down gracefully...");
            cancel_token.cancel();
            // Wait for processor to finish
            match event_processor.task_handle.await {
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
        result = &mut event_processor.task_handle => {
            // Processor completed normally (all events processed)
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
    }

    // Shutdown adapters
    println!("🧹 Shutting down adapters...");
    user_adapter.shutdown().await;
    order_adapter.shutdown().await;
    inventory_adapter.shutdown().await;

    println!("✅ All tasks completed, exiting.");

    Ok(())
}
