// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Event aggregation logic

use crate::domain::{AggregatedEvent, DataEvent};
use fluxion_rx::prelude::*;
use fluxion_rx::OrderedWrapper;

/// Creates an aggregated event from combined stream state
pub fn create_aggregated_event(
    combined: OrderedWrapper<CombinedState<DataEvent>>,
) -> AggregatedEvent {
    let inner = combined.into_inner();
    let state = inner.values();

    // Extract latest of each type - state is Vec<DataEvent>
    let sensor = state
        .iter()
        .rev()
        .find_map(|e| match e {
            DataEvent::Sensor(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap();

    let metric = state
        .iter()
        .rev()
        .find_map(|e| match e {
            DataEvent::Metric(m) => Some(m.clone()),
            _ => None,
        })
        .unwrap();

    let sys_event = state
        .iter()
        .rev()
        .find_map(|e| match e {
            DataEvent::SystemEvent(e) => Some(e.clone()),
            _ => None,
        })
        .unwrap();

    AggregatedEvent {
        timestamp: sensor.timestamp,
        temperature: Some(sensor.temperature), // Already in integer format (temp * 10)
        metric_value: Some(metric.value),
        has_alert: sys_event.event_type == "ALERT",
    }
}
