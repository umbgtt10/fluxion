// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde::{Deserialize, Serialize};

/// User record from legacy database (JSON format)
#[derive(Debug, Clone, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}

/// Order event from legacy message queue (XML format)
#[derive(Debug, Clone, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq)]
pub struct Order {
    pub id: u64,
    pub user_id: u64,
    pub product_id: u64,
    pub quantity: u32,
    #[serde(default)]
    pub status: OrderStatus,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq)]
pub enum OrderStatus {
    #[default]
    Pending,
    Fulfilled,
    Failed,
}

/// Inventory update from legacy file watcher (CSV format)
#[derive(Debug, Clone, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq)]
pub struct Inventory {
    pub product_id: u64,
    pub product_name: String,
    pub quantity: u32,
}
