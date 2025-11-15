// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Unified data event enum

use fluxion::Ordered;

use super::{MetricData, SensorReading, SystemEvent};

/// Unified event type for combining different data sources
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataEvent {
    Sensor(SensorReading),
    Metric(MetricData),
    SystemEvent(SystemEvent),
}

impl Ordered for DataEvent {
    type Inner = DataEvent;

    fn order(&self) -> u64 {
        match self {
            DataEvent::Sensor(s) => s.timestamp,
            DataEvent::Metric(m) => m.timestamp,
            DataEvent::SystemEvent(e) => e.timestamp,
        }
    }

    fn get(&self) -> &Self::Inner {
        self
    }

    fn with_order(value: Self::Inner, _order: u64) -> Self {
        value // Just return the value since it already has the timestamp
    }
}
