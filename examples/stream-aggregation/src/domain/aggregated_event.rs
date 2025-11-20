// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Aggregated event domain type

use fluxion_rx::Timestamped;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AggregatedEvent {
    pub timestamp: u64,
    pub temperature: Option<i32>, // Store as integer (temp * 10)
    pub metric_value: Option<u64>,
    pub has_alert: bool,
}

impl Timestamped for AggregatedEvent {
    type Inner = AggregatedEvent;

    fn timestamp(&self) -> u64 {
        self.timestamp
    }

    fn inner(&self) -> &Self::Inner {
        self
    }

    fn with_timestamp(value: Self::Inner, _timestamp: u64) -> Self {
        value
    }

    fn with_fresh_timestamp(value: Self) -> Self {
        value
    }
}
