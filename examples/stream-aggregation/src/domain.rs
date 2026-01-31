// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

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
