// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::models::{Inventory, Order, User};

/// Unified event type that wraps all legacy data sources
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum UnifiedEvent {
    UserAdded(User),
    OrderReceived(Order),
    InventoryUpdated(Inventory),
}

impl std::fmt::Display for UnifiedEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
