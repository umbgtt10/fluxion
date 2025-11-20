// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Unified data event enum

use fluxion_rx::{CompareByInner, Timestamped};

use super::{MetricData, SensorReading, SystemEvent};

/// Unified event type for combining different data sources
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataEvent {
    Sensor(SensorReading),
    Metric(MetricData),
    SystemEvent(SystemEvent),
}

impl Timestamped for DataEvent {
    type Inner = DataEvent;
    type Timestamp = u64;

    fn timestamp(&self) -> Self::Timestamp {
        match self {
            DataEvent::Sensor(s) => s.timestamp,
            DataEvent::Metric(m) => m.timestamp,
            DataEvent::SystemEvent(e) => e.timestamp,
        }
    }

    fn inner(&self) -> &Self::Inner {
        self
    }

    fn with_timestamp(value: Self::Inner, _timestamp: Self::Timestamp) -> Self {
        value // Just return the value since it already has the timestamp
    }

    fn with_fresh_timestamp(value: Self) -> Self {
        value
    }
}

impl CompareByInner for DataEvent {
    fn cmp_inner(&self, other: &Self) -> std::cmp::Ordering {
        self.inner().cmp(other.inner())
    }
}
