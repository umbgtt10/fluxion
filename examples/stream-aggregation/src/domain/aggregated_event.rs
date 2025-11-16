// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Aggregated event domain type

use fluxion_rx::Ordered;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AggregatedEvent {
    pub timestamp: u64,
    pub temperature: Option<i32>, // Store as integer (temp * 10)
    pub metric_value: Option<u64>,
    pub has_alert: bool,
}

impl Ordered for AggregatedEvent {
    type Inner = AggregatedEvent;

    fn order(&self) -> u64 {
        self.timestamp
    }

    fn get(&self) -> &Self::Inner {
        self
    }

    fn with_order(value: Self::Inner, _order: u64) -> Self {
        value
    }
}
