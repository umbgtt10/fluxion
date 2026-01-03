// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use fluxion_core::{HasTimestamp, Timestamped};
use fluxion_stream_time::runtimes::wasm_implementation::WasmInstant;

/// A sensor value with timestamp
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SensorValue {
    pub timestamp: WasmInstant,
    pub value: u32,
}

impl HasTimestamp for SensorValue {
    type Timestamp = WasmInstant;

    fn timestamp(&self) -> Self::Timestamp {
        self.timestamp
    }
}

impl Timestamped for SensorValue {
    type Inner = Self;

    fn with_timestamp(value: Self, timestamp: WasmInstant) -> Self {
        Self {
            timestamp,
            value: value.value,
        }
    }

    fn into_inner(self) -> Self {
        self
    }
}
