// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Domain types for the RabbitMQ aggregator example

use fluxion::Ordered;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SensorReading {
    pub timestamp: u64,
    pub sensor_id: String,
    pub temperature: i32, // Store as integer (e.g., temperature * 10)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MetricData {
    pub timestamp: u64,
    pub metric_name: String,
    pub value: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SystemEvent {
    pub timestamp: u64,
    pub event_type: String,
    pub severity: String,
}

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
