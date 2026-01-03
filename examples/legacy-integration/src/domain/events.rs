// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::models::{Inventory, Order, User};

/// Unified event type that wraps all legacy data sources
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum UnifiedEvent {
    UserAdded(User),
    OrderReceived(Order),
    InventoryUpdated(Inventory),
}

impl core::fmt::Display for UnifiedEvent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            UnifiedEvent::UserAdded(u) => write!(f, "User({}, {})", u.id, u.name),
            UnifiedEvent::OrderReceived(o) => {
                write!(
                    f,
                    "Order({}, user={}, product={}, qty={})",
                    o.id, o.user_id, o.product_id, o.quantity
                )
            }
            UnifiedEvent::InventoryUpdated(i) => {
                write!(
                    f,
                    "Inventory({}, {}, qty={})",
                    i.product_id, i.product_name, i.quantity
                )
            }
        }
    }
}
