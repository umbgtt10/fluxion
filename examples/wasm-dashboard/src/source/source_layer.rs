// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::SensorStreams;

pub struct SourceLayer {
    sensor_streams: SensorStreams,
}

impl SourceLayer {
    pub fn new(sensor_streams: SensorStreams) -> Self {
        Self { sensor_streams }
    }

    pub fn sensor_streams(&self) -> &SensorStreams {
        &self.sensor_streams
    }
}
