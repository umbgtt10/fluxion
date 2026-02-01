// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::raw_streams::Sensors;
use super::sensor_value::SensorValue;
use fluxion_runtime::impls::wasm::WasmTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream::{FluxionShared, IntoFluxionStream, ShareExt};

pub struct SensorStreams {
    _sensors: Sensors,
    sensor1: FluxionShared<SensorValue>,
    sensor2: FluxionShared<SensorValue>,
    sensor3: FluxionShared<SensorValue>,
}

impl SensorStreams {
    pub fn new(sensors: Sensors) -> Self {
        let timer = WasmTimer;

        let timer1 = timer.clone();
        let sensor1 = sensors
            .sensor1
            .receiver()
            .into_fluxion_stream_map(move |value| SensorValue {
                timestamp: timer1.now(),
                value,
            })
            .share();

        let timer2 = timer.clone();
        let sensor2 = sensors
            .sensor2
            .receiver()
            .into_fluxion_stream_map(move |value| SensorValue {
                timestamp: timer2.now(),
                value,
            })
            .share();

        let timer3 = timer.clone();
        let sensor3 = sensors
            .sensor3
            .receiver()
            .into_fluxion_stream_map(move |value| SensorValue {
                timestamp: timer3.now(),
                value,
            })
            .share();

        Self {
            _sensors: sensors,
            sensor1,
            sensor2,
            sensor3,
        }
    }

    pub fn sensor1(&self) -> &FluxionShared<SensorValue> {
        &self.sensor1
    }

    pub fn sensor2(&self) -> &FluxionShared<SensorValue> {
        &self.sensor2
    }

    pub fn sensor3(&self) -> &FluxionShared<SensorValue> {
        &self.sensor3
    }
}
