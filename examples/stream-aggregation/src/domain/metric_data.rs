// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Metric data domain type

use fluxion::prelude::Ordered;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MetricData {
    pub timestamp: u64,
    pub metric_name: String,
    pub value: u64,
}

impl Ordered for MetricData {
    type Inner = Self;

    fn get(&self) -> &Self::Inner {
        self
    }

    fn order(&self) -> u64 {
        self.timestamp
    }

    fn with_order(inner: Self::Inner, order: u64) -> Self {
        Self {
            timestamp: order,
            ..inner
        }
    }
}
