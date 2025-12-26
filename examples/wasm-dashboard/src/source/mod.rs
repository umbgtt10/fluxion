// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Source layer - Raw sensor stream generation.
//!
//! This module provides the foundational data sources for the dashboard.
//! Three independent sensors generate random values at random frequencies
//! (1-5 Hz) without timestamps.

mod combined_stream;
mod raw_streams;
mod result_streams;
mod sensor;
mod sensor_streams;
mod sensor_value;

pub use combined_stream::CombinedStream;
pub use raw_streams::Sensors;
pub use result_streams::{ResultStreams, WasmStream};
pub use sensor_streams::SensorStreams;
pub use sensor_value::SensorValue;
