// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_rx::prelude::{HasTimestamp, Timestamped};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MetricData {
    pub timestamp: u64,
    pub metric_name: String,
    pub value: u64,
}

impl HasTimestamp for MetricData {
    type Timestamp = u64;

    fn timestamp(&self) -> Self::Timestamp {
        self.timestamp
    }
}

impl Timestamped for MetricData {
    type Inner = Self;

    fn with_timestamp(inner: Self::Inner, timestamp: Self::Timestamp) -> Self {
        Self { timestamp, ..inner }
    }

    fn into_inner(self) -> Self::Inner {
        self
    }
}
