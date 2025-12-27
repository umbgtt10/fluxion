// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::SensorStreams;

/// Source layer - Raw data generation
///
/// This layer represents the bottom of the architecture, responsible for
/// generating raw sensor data streams. It encapsulates the sensor creation
/// and provides access to the three independent sensor streams.
pub struct SourceLayer {
    sensor_streams: SensorStreams,
}

impl SourceLayer {
    /// Creates a new source layer from sensor streams
    ///
    /// # Arguments
    ///
    /// * `sensor_streams` - The sensor streams container
    pub fn new(sensor_streams: SensorStreams) -> Self {
        Self { sensor_streams }
    }

    /// Get access to the sensor streams for processing
    pub fn sensor_streams(&self) -> &SensorStreams {
        &self.sensor_streams
    }
}
