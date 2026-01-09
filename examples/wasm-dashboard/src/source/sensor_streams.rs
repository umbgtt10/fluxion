// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::raw_streams::Sensors;
use super::sensor_value::SensorValue;
use fluxion_runtime::impls::wasm::WasmTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream::{FluxionShared, IntoFluxionStream, ShareExt};

/// Container for three shared timestamped sensor streams (Phase 2)
///
/// Each stream wraps a raw sensor receiver, adds timestamps using WasmTimer,
/// and applies share() for multiple subscriptions.
pub struct SensorStreams {
    _sensors: Sensors, // Keep sensors alive (tasks run in background)
    sensor1: FluxionShared<SensorValue>,
    sensor2: FluxionShared<SensorValue>,
    sensor3: FluxionShared<SensorValue>,
}

impl SensorStreams {
    /// Creates timestamped shared streams from raw sensors
    ///
    /// # Arguments
    ///
    /// * `sensors` - Raw sensor collection from Phase 1
    ///
    /// # Example
    ///
    /// ```ignore
    /// let cancel = CancellationToken::new();
    /// let sensors = Sensors::new(cancel.clone());
    /// let streams = SensorStreams::new(sensors);
    /// ```
    pub fn new(sensors: Sensors) -> Self {
        let timer = WasmTimer;

        // Create shared stream for sensor 1 (values 1-9)
        let timer1 = timer.clone();
        let sensor1 = sensors
            .sensor1
            .receiver()
            .into_fluxion_stream_map(move |value| SensorValue {
                timestamp: timer1.now(),
                value,
            })
            .share();

        // Create shared stream for sensor 2 (values 10-90)
        let timer2 = timer.clone();
        let sensor2 = sensors
            .sensor2
            .receiver()
            .into_fluxion_stream_map(move |value| SensorValue {
                timestamp: timer2.now(),
                value,
            })
            .share();

        // Create shared stream for sensor 3 (values 100-900)
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

    /// Returns a reference to sensor1 shared stream
    pub fn sensor1(&self) -> &FluxionShared<SensorValue> {
        &self.sensor1
    }

    /// Returns a reference to sensor2 shared stream
    pub fn sensor2(&self) -> &FluxionShared<SensorValue> {
        &self.sensor2
    }

    /// Returns a reference to sensor3 shared stream
    pub fn sensor3(&self) -> &FluxionShared<SensorValue> {
        &self.sensor3
    }
}
