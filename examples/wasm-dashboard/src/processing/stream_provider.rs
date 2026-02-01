// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::processing::WasmStream;
use crate::source::SensorValue;
use fluxion_stream::share::SharedBoxStream;
use fluxion_stream_time::WasmTimestamped;

pub trait StreamProvider {
    fn sensor1_stream(&self) -> SharedBoxStream<SensorValue>;
    fn sensor2_stream(&self) -> SharedBoxStream<SensorValue>;
    fn sensor3_stream(&self) -> SharedBoxStream<SensorValue>;
    fn combined_stream(&self) -> SharedBoxStream<WasmTimestamped<u32>>;
    fn debounce_stream(&self) -> WasmStream<u32>;
    fn delay_stream(&self) -> WasmStream<u32>;
    fn sample_stream(&self) -> WasmStream<u32>;
    fn throttle_stream(&self) -> WasmStream<u32>;
    fn timeout_stream(&self) -> WasmStream<u32>;
}
