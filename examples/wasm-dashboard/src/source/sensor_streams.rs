// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::raw_streams::Sensors;
use super::sensor_value::SensorValue;
use fluxion_core::{StreamItem, Timestamped};
use fluxion_stream::fluxion_shared::SharedBoxStream;
use fluxion_stream::{IntoFluxionStream, ShareExt};
use fluxion_stream_time::runtimes::wasm_implementation::WasmTimer;
use fluxion_stream_time::timer::Timer;
use futures::StreamExt;

/// Container for three shared timestamped sensor streams (Phase 2)
///
/// Each stream wraps a raw sensor receiver, adds timestamps using WasmTimer,
/// and applies share() for multiple subscriptions.
pub struct SensorStreams {
    _sensors: Sensors, // Keep sensors alive (tasks run in background)
    sensor1: fluxion_stream::FluxionShared<SensorValue>,
    sensor2: fluxion_stream::FluxionShared<SensorValue>,
    sensor3: fluxion_stream::FluxionShared<SensorValue>,
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
        let timer = WasmTimer::new();

        // Create shared stream for sensor 1 (values 1-9)
        let timer1 = timer.clone();
        let sensor1 = sensors
            .sensor1
            .receiver()
            .into_fluxion_stream()
            .map(move |item| match item {
                StreamItem::Value(value) => {
                    StreamItem::Value(SensorValue::with_timestamp(value, timer1.now()))
                }
                StreamItem::Error(e) => StreamItem::Error(e),
            })
            .share();

        // Create shared stream for sensor 2 (values 10-90)
        let timer2 = timer.clone();
        let sensor2 = sensors
            .sensor2
            .receiver()
            .into_fluxion_stream()
            .map(move |item| match item {
                StreamItem::Value(value) => {
                    StreamItem::Value(SensorValue::with_timestamp(value, timer2.now()))
                }
                StreamItem::Error(e) => StreamItem::Error(e),
            })
            .share();

        // Create shared stream for sensor 3 (values 100-900)
        let timer3 = timer.clone();
        let sensor3 = sensors
            .sensor3
            .receiver()
            .into_fluxion_stream()
            .map(move |item| match item {
                StreamItem::Value(value) => {
                    StreamItem::Value(SensorValue::with_timestamp(value, timer3.now()))
                }
                StreamItem::Error(e) => StreamItem::Error(e),
            })
            .share();

        Self {
            _sensors: sensors,
            sensor1,
            sensor2,
            sensor3,
        }
    }

    /// Returns cloned streams for subscription
    ///
    /// Each call creates independent subscriptions to the shared streams.
    /// Use this to subscribe multiple consumers (e.g., GUI + combine_latest).
    ///
    /// # Returns
    ///
    /// Vector of three streams in order: [sensor1, sensor2, sensor3]
    pub fn subscribe(&self) -> Vec<SharedBoxStream<SensorValue>> {
        vec![
            self.sensor1.subscribe().unwrap(),
            self.sensor2.subscribe().unwrap(),
            self.sensor3.subscribe().unwrap(),
        ]
    }

    /// Returns a reference to sensor1 shared stream
    pub fn sensor1(&self) -> &fluxion_stream::FluxionShared<SensorValue> {
        &self.sensor1
    }

    /// Returns a reference to sensor2 shared stream
    pub fn sensor2(&self) -> &fluxion_stream::FluxionShared<SensorValue> {
        &self.sensor2
    }

    /// Returns a reference to sensor3 shared stream
    pub fn sensor3(&self) -> &fluxion_stream::FluxionShared<SensorValue> {
        &self.sensor3
    }
}
