// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! RabbitMQ Event Aggregator - Real-time multi-source event processing
//!
//! This example demonstrates:
//! - Consuming from multiple RabbitMQ queues
//! - Ordered aggregation using timestamps
//! - Stream transformation and filtering
//! - Publishing aggregated results
//! - Graceful shutdown with Ctrl+C
//!
//! Architecture:
//! ```
//! Producer Tasks (3)      Aggregator Task          Consumer Task
//! ┌─────────────┐         ┌──────────────┐         ┌──────────┐
//! │  Queue 1    │────────▶│              │         │          │
//! │ (sensors)   │         │  FluxionStream│────────▶│  Queue 4 │
//! │  Queue 2    │────────▶│  combine &   │         │ (output) │
//! │ (metrics)   │         │  transform   │         │          │
//! │  Queue 3    │────────▶│              │         │          │
//! │ (events)    │         └──────────────┘         └──────────┘
//! └─────────────┘
//! ```
//!
//! Run with: `cargo run --example rabbitmq_aggregator`

mod aggregator;
mod consumer;
mod domain;
mod events_producer;
mod metrics_producer;
mod sensor_producer;

use aggregator::Aggregator;
use tokio::select;
use tokio::signal;
use tokio_util::sync::CancellationToken;

// ============================================================================
// Main - Single-threaded with parallel tasks
// ============================================================================

#[tokio::main(flavor = "current_thread")]
async fn main() {
    println!("\n🚀 RabbitMQ Event Aggregator Example");
    println!("=====================================\n");
    println!("Press Ctrl+C to stop\n");

    // Create cancellation token for graceful shutdown
    let cancel_token = CancellationToken::new();

    // Create and start aggregator (which manages all producers and consumer)
    let mut aggregator = Aggregator::new(cancel_token.clone());
    aggregator.start();

    // Wait for Ctrl+C
    select! {
        _ = signal::ctrl_c() => {
            println!("\n\n🛑 Shutting down gracefully...\n");
            aggregator.stop().await;
            println!("✅ All tasks stopped\n");
        }
    }
}
