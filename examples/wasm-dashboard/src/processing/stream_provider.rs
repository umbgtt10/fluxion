// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::processing::WasmStream;
use crate::source::SensorValue;
use fluxion_stream::fluxion_shared::SharedBoxStream;
use fluxion_stream_time::WasmTimestamped;

/// Trait for providing access to all dashboard streams
///
/// This abstraction defines the interface between the processing layer
/// and the orchestration layer (DashboardUpdater). It allows the updater
/// to remain independent of concrete stream implementations.
pub trait StreamProvider {
    /// Get a subscription to sensor 1 stream
    fn sensor1_stream(&self) -> SharedBoxStream<SensorValue>;

    /// Get a subscription to sensor 2 stream
    fn sensor2_stream(&self) -> SharedBoxStream<SensorValue>;

    /// Get a subscription to sensor 3 stream
    fn sensor3_stream(&self) -> SharedBoxStream<SensorValue>;

    /// Get a subscription to the combined stream
    fn combined_stream(&self) -> SharedBoxStream<WasmTimestamped<u32>>;

    /// Get a subscription to the debounced stream
    fn debounce_stream(&self) -> WasmStream<u32>;

    /// Get a subscription to the delayed stream
    fn delay_stream(&self) -> WasmStream<u32>;

    /// Get a subscription to the sampled stream
    fn sample_stream(&self) -> WasmStream<u32>;

    /// Get a subscription to the throttled stream
    fn throttle_stream(&self) -> WasmStream<u32>;

    /// Get a subscription to the timeout stream
    fn timeout_stream(&self) -> WasmStream<u32>;
}
