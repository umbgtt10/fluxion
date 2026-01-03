// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::source::SensorStreams;
use fluxion_core::{HasTimestamp, Timestamped};
use fluxion_stream::fluxion_shared::SharedBoxStream;
use fluxion_stream::{CombineLatestExt, FluxionShared, MapOrderedExt, ShareExt};
use fluxion_stream_time::WasmTimestamped;

/// Combined and filtered stream from all three sensors.
///
/// Takes the three shared timestamped sensor streams, combines them with
/// `combine_latest`, filters for even sums only, and shares the result.
///
/// The shared output can be subscribed to multiple times (GUI + operators).
pub struct CombinedStream {
    combined: FluxionShared<WasmTimestamped<u32>>,
}

impl CombinedStream {
    /// Creates a combined stream from three sensor streams.
    ///
    /// # Process
    /// 1. Subscribe to all three shared sensor streams
    /// 2. Combine with `combine_latest` (emits when any sensor updates)
    /// 3. Filter to only pass even sums
    /// 4. Map to extract the sum value
    /// 5. Share for multiple subscriptions (1 GUI + 5 operators)
    ///
    /// # Arguments
    ///
    /// * `sensors` - The three shared timestamped sensor streams
    pub fn new(sensors: &SensorStreams) -> Self {
        let combined = sensors
            .sensor1()
            .subscribe()
            .unwrap()
            .combine_latest(
                vec![
                    sensors.sensor2().subscribe().unwrap(),
                    sensors.sensor3().subscribe().unwrap(),
                ],
                |state| {
                    // Only pass through even sums
                    let values = state.values();
                    let sum: u32 = values.iter().map(|v| v.value).sum();
                    sum % 3 == 0
                },
            )
            .map_ordered(|state| {
                let values = state.values();
                let sum: u32 = values.iter().map(|v| v.value).sum();
                WasmTimestamped::with_timestamp(sum, state.timestamp())
            })
            .share();

        Self { combined }
    }

    /// Returns a subscription to the combined stream.
    ///
    /// Multiple subscriptions can be created (for GUI and operators).
    pub fn subscribe(&self) -> SharedBoxStream<WasmTimestamped<u32>> {
        Box::pin(self.combined.subscribe().unwrap())
    }
}
