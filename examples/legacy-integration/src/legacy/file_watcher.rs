// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Simulates a legacy file watcher producing CSV inventory updates
//! In production, this would watch a directory for new CSV files

use crate::domain::models::Inventory;
use fluxion_core::CancellationToken;
use futures::{channel::mpsc::UnboundedSender, FutureExt};
use tokio::time::{sleep, Duration};

pub struct LegacyFileWatcher;

impl LegacyFileWatcher {
    pub fn new() -> Self {
        Self
    }

    /// Simulates watching a directory for new CSV inventory files
    /// In production: notify::Watcher on /var/legacy/inventory/*.csv
    pub async fn watch_inventory(self, tx: UnboundedSender<Inventory>, cancel: CancellationToken) {
        println!("  📁 Legacy File Watcher: Watching for CSV inventory files (every 4s)");

        let products = [
            (100, "Widget A"),
            (101, "Widget B"),
            (102, "Widget C"),
            (103, "Widget D"),
            (104, "Widget E"),
            (105, "Widget F"),
            (106, "Widget G"),
            (107, "Widget H"),
            (108, "Widget I"),
            (109, "Widget J"),
            (110, "Widget K"),
        ];

        loop {
            futures::select! {
                _ = cancel.cancelled().fuse() => {
                    println!("  📁 Legacy File Watcher: Shutting down");
                    break;
                }
                _ = sleep(Duration::from_secs(4)).fuse() => {
                    let (product_id, product_name) = products[fastrand::usize(0..products.len())];

                    let inventory = Inventory {
                        product_id,
                        product_name: product_name.to_string(),
                        quantity: fastrand::u32(10..100),
                    };

                    // Simulate CSV parsing (legacy file contains CSV data)
                    let _csv = format!("{},{},{}", inventory.product_id, inventory.product_name, inventory.quantity);

                    if tx.unbounded_send(inventory).is_err() {
                        break;
                    }
                }
            }
        }
    }
}
