// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

pub mod inventory_adapter;
pub mod order_adapter;
pub mod user_adapter;

pub use inventory_adapter::InventoryAdapter;
pub use order_adapter::OrderAdapter;
pub use user_adapter::UserAdapter;
