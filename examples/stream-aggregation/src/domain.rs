// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Domain types for the RabbitMQ aggregator example

mod aggregated_event;
mod data_event;
mod metric_data;
mod sensor_reading;
mod system_event;

pub use aggregated_event::AggregatedEvent;
pub use data_event::DataEvent;
pub use metric_data::MetricData;
pub use sensor_reading::SensorReading;
pub use system_event::SystemEvent;
