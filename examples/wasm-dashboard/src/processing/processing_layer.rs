// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::{CombinedStream, ResultStreams, StreamProvider, WasmStream};
use crate::source::{SensorValue, SourceLayer};
use fluxion_stream::fluxion_shared::SharedBoxStream;
use fluxion_stream_time::WasmTimestamped;

/// Processing layer - Stream transformation and composition
///
/// This layer sits between the source layer (raw sensors) and the orchestration
/// layer (DashboardUpdater). It combines, filters, and applies time-based operators
/// to create the 9 output streams needed by the dashboard.
pub struct ProcessingLayer<'a> {
    source: &'a SourceLayer,
    combined_stream: CombinedStream,
}

impl<'a> ProcessingLayer<'a> {
    /// Creates a new processing layer from a source layer
    ///
    /// # Arguments
    ///
    /// * `source` - The source layer containing sensor streams
    pub fn new(source: &'a SourceLayer) -> Self {
        let combined_stream = CombinedStream::new(source.sensor_streams());

        Self {
            source,
            combined_stream,
        }
    }
}

impl<'a> StreamProvider for ProcessingLayer<'a> {
    fn sensor1_stream(&self) -> SharedBoxStream<SensorValue> {
        self.source
            .sensor_streams()
            .sensor1()
            .subscribe()
            .expect("Sensor 1 subject should not be closed")
    }

    fn sensor2_stream(&self) -> SharedBoxStream<SensorValue> {
        self.source
            .sensor_streams()
            .sensor2()
            .subscribe()
            .expect("Sensor 2 subject should not be closed")
    }

    fn sensor3_stream(&self) -> SharedBoxStream<SensorValue> {
        self.source
            .sensor_streams()
            .sensor3()
            .subscribe()
            .expect("Sensor 3 subject should not be closed")
    }

    fn combined_stream(&self) -> SharedBoxStream<WasmTimestamped<u32>> {
        self.combined_stream.subscribe()
    }

    fn debounce_stream(&self) -> WasmStream<u32> {
        let result_streams = ResultStreams::new(&self.combined_stream);
        result_streams.subscribe_debounce()
    }

    fn delay_stream(&self) -> WasmStream<u32> {
        let result_streams = ResultStreams::new(&self.combined_stream);
        result_streams.subscribe_delay()
    }

    fn sample_stream(&self) -> WasmStream<u32> {
        let result_streams = ResultStreams::new(&self.combined_stream);
        result_streams.subscribe_sample()
    }

    fn throttle_stream(&self) -> WasmStream<u32> {
        let result_streams = ResultStreams::new(&self.combined_stream);
        result_streams.subscribe_throttle()
    }

    fn timeout_stream(&self) -> WasmStream<u32> {
        let result_streams = ResultStreams::new(&self.combined_stream);
        result_streams.subscribe_timeout()
    }
}
