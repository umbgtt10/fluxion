// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::HashMap;

use fluxion_core::stream_item::StreamItem;
use fluxion_stream::{FluxionStream, MergedStream};
use futures::Stream;

use super::{
    events::UnifiedEvent,
    models::{Inventory, Order, User},
    TimestampedEvent,
};

/// Aggregated state repository built from multiple legacy sources
#[derive(Debug, Default)]
pub struct Repository {
    pub users: HashMap<u64, User>,
    pub orders: HashMap<u64, Order>,
    pub inventory: HashMap<u64, Inventory>,
}

impl Repository {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Build the aggregated repository stream using merge_with
pub fn build_repository_stream(
    user_stream: impl Stream<Item = TimestampedEvent> + Send + Sync + Unpin + 'static,
    order_stream: impl Stream<Item = TimestampedEvent> + Send + Sync + Unpin + 'static,
    inventory_stream: impl Stream<Item = TimestampedEvent> + Send + Sync + Unpin + 'static,
) -> FluxionStream<impl Stream<Item = StreamItem<TimestampedEvent>>> {
    MergedStream::seed::<TimestampedEvent>(Repository::new())
        .merge_with(user_stream, |event, repo| {
            if let UnifiedEvent::UserAdded(user) = &event {
                repo.users.insert(user.id, user.clone());
            }
            event
        })
        .merge_with(order_stream, |event, repo| {
            if let UnifiedEvent::OrderReceived(order) = &event {
                repo.orders.insert(order.id, order.clone());
            }
            event
        })
        .merge_with(inventory_stream, |event, repo| {
            if let UnifiedEvent::InventoryUpdated(inventory) = &event {
                repo.inventory
                    .insert(inventory.product_id, inventory.clone());
            }
            event
        })
        .into_fluxion_stream()
}
