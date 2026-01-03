// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::HashMap;

use fluxion_core::stream_item::StreamItem;
use fluxion_core::IntoStream;
use fluxion_stream::prelude::*;
use futures::{Stream, StreamExt};

use super::{
    events::UnifiedEvent,
    models::{Inventory, Order, User},
    TimestampedEvent,
};

/// Aggregated order analytics
#[derive(Debug, Clone, Default)]
pub struct OrderAnalytics {
    pub total_orders: u64,
    pub total_quantity: u64,
    pub orders_by_user: HashMap<u64, u64>,
    pub orders_by_product: HashMap<u64, u64>,
    pub quantity_by_product: HashMap<u64, u64>,
}

impl OrderAnalytics {
    pub fn add_order(&mut self, order: &Order) {
        self.total_orders += 1;
        self.total_quantity += order.quantity as u64;
        *self.orders_by_user.entry(order.user_id).or_insert(0) += 1;
        *self.orders_by_product.entry(order.product_id).or_insert(0) += 1;
        *self
            .quantity_by_product
            .entry(order.product_id)
            .or_insert(0) += order.quantity as u64;
    }
}

/// Aggregated state repository built from multiple legacy sources
pub struct Repository {
    pub users: HashMap<u64, User>,
    pub orders: HashMap<u64, Order>,
    pub inventory: HashMap<u64, Inventory>,
    #[allow(dead_code)]
    user_stream: Option<Box<dyn Stream<Item = TimestampedEvent> + Send + Sync + Unpin>>,
    #[allow(dead_code)]
    order_stream: Option<Box<dyn Stream<Item = TimestampedEvent> + Send + Sync + Unpin>>,
    #[allow(dead_code)]
    inventory_stream: Option<Box<dyn Stream<Item = TimestampedEvent> + Send + Sync + Unpin>>,
}

impl Repository {
    /// Create a new Repository with the given streams
    pub fn new(
        user_stream: impl Stream<Item = TimestampedEvent> + Send + Sync + Unpin + 'static,
        order_stream: impl Stream<Item = TimestampedEvent> + Send + Sync + Unpin + 'static,
        inventory_stream: impl Stream<Item = TimestampedEvent> + Send + Sync + Unpin + 'static,
    ) -> Self {
        Self {
            users: HashMap::new(),
            orders: HashMap::new(),
            inventory: HashMap::new(),
            user_stream: Some(Box::new(user_stream)),
            order_stream: Some(Box::new(order_stream)),
            inventory_stream: Some(Box::new(inventory_stream)),
        }
    }

    /// Create the aggregated FluxionStream from the stored streams
    pub fn create_stream(mut self) -> impl Stream<Item = StreamItem<TimestampedEvent>> {
        let user_stream = self
            .user_stream
            .take()
            .expect("user_stream already consumed");
        let order_stream = self
            .order_stream
            .take()
            .expect("order_stream already consumed");
        let inventory_stream = self
            .inventory_stream
            .take()
            .expect("inventory_stream already consumed");

        let initial_state = Repository {
            users: HashMap::new(),
            orders: HashMap::new(),
            inventory: HashMap::new(),
            user_stream: None,
            order_stream: None,
            inventory_stream: None,
        };

        MergedStream::seed::<TimestampedEvent>(initial_state)
            .merge_with(user_stream.map(StreamItem::Value), |event, repo| {
                if let UnifiedEvent::UserAdded(user) = &event {
                    repo.users.insert(user.id, user.clone());
                }
                event
            })
            .merge_with(order_stream.map(StreamItem::Value), |event, repo| {
                if let UnifiedEvent::OrderReceived(order) = &event {
                    repo.orders.insert(order.id, order.clone());
                }
                event
            })
            .merge_with(inventory_stream.map(StreamItem::Value), |event, repo| {
                if let UnifiedEvent::InventoryUpdated(inventory) = &event {
                    repo.inventory
                        .insert(inventory.product_id, inventory.clone());
                }
                event
            })
            .into_stream()
    }
}
