// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Metric data domain type

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MetricData {
    pub timestamp: u64,
    pub metric_name: String,
    pub value: u64,
}
