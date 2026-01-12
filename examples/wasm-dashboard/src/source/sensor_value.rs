// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{HasTimestamp, Timestamped};
use fluxion_runtime::impls::wasm::WasmInstant;

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
