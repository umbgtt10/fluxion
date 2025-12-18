// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Aggregated event domain type

use fluxion_rx::{HasTimestamp, Timestamped};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AggregatedEvent {
    pub timestamp: u64,
    pub temperature: Option<i32>, // Store as integer (temp * 10)
    pub metric_value: Option<u64>,
    pub has_alert: bool,
}

impl HasTimestamp for AggregatedEvent {
    type Timestamp = u64;

    fn timestamp(&self) -> Self::Timestamp {
        self.timestamp
    }
}

impl Timestamped for AggregatedEvent {
    type Inner = AggregatedEvent;

    fn with_timestamp(value: Self::Inner, _timestamp: Self::Timestamp) -> Self {
        value
    }

    fn into_inner(self) -> Self::Inner {
        self
    }
}
