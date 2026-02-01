// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::source::SensorStreams;
use fluxion_core::{HasTimestamp, Timestamped};
use fluxion_stream::share::SharedBoxStream;
use fluxion_stream::{CombineLatestExt, FluxionShared, MapOrderedExt, ShareExt};
use fluxion_stream_time::WasmTimestamped;

pub struct CombinedStream {
    combined: FluxionShared<WasmTimestamped<u32>>,
}

impl CombinedStream {
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
                    let values = state.values();
                    let sum: u32 = values.iter().map(|v| v.value).sum();
                    sum.is_multiple_of(3)
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

    pub fn subscribe(&self) -> SharedBoxStream<WasmTimestamped<u32>> {
        Box::pin(self.combined.subscribe().unwrap())
    }
}
