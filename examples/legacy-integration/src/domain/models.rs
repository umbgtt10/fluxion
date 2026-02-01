// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq)]
pub struct Inventory {
    pub product_id: u64,
    pub product_name: String,
    pub quantity: u32,
}
