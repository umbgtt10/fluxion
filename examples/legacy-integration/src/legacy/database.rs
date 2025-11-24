// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Simulates a legacy database that produces JSON user records
//! In production, this would poll a real database table

use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;

use crate::domain::models::User;

pub struct LegacyDatabase {
    user_id_counter: AtomicU64,
}

impl LegacyDatabase {
    pub fn new() -> Self {
        Self {
            user_id_counter: AtomicU64::new(1000),
        }
    }

    /// Simulates polling a database for new user records
    /// In production: SELECT * FROM users WHERE processed = 0
    pub async fn poll_users(self, tx: UnboundedSender<User>, cancel: CancellationToken) {
        println!("  🗄️  Legacy Database: Polling for new users (every 3s)");

        let names = ["Alice Smith", "Bob Jones", "Carol Davis", "David Wilson", "Eve Martinez"];

        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    println!("  🗄️  Legacy Database: Shutting down");
                    break;
                }
                _ = sleep(Duration::from_secs(3)) => {
                    let user_id = self.user_id_counter.fetch_add(1, Ordering::SeqCst);
                    let name = names[user_id as usize % names.len()];

                    let user = User {
                        id: user_id,
                        name: name.to_string(),
                        email: format!("{}@legacy.com", name.to_lowercase().replace(' ', ".")),
                    };

                    // Simulate JSON serialization (legacy DB returns JSON strings)
                    let _json = serde_json::to_string(&user).unwrap();

                    if tx.send(user).is_err() {
                        break;
                    }
                }
            }
        }
    }
}
