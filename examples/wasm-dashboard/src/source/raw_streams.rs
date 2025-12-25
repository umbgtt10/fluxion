// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::sensor::Sensor;
use fluxion_core::CancellationToken;

/// Container for the three raw sensors (Phase 1)
///
/// Each sensor generates values at random frequencies (1-5 Hz) within
/// specific ranges:
/// - Sensor 1: Values 1-9
/// - Sensor 2: Values 10-90
/// - Sensor 3: Values 100-900
///
/// These streams emit plain sensor values without timestamps.
pub struct Sensors {
    pub sensor1: Sensor,
    pub sensor2: Sensor,
    pub sensor3: Sensor,
}

impl Sensors {
    /// Creates three independent sensor streams with predefined ranges.
    ///
    /// All sensors operate at 1-5 Hz (200-1000ms intervals) with natural
    /// timing variation.
    ///
    /// # Arguments
    ///
    /// * `cancel_token` - Cancellation token to stop all sensors
    pub fn new(cancel_token: CancellationToken) -> Self {
        Self {
            // Sensor 1: Values 1-9, Frequency 1-5 Hz
            sensor1: Sensor::new((200, 1000), (1, 9), cancel_token.clone()),
            // Sensor 2: Values 10-90, Frequency 1-5 Hz
            sensor2: Sensor::new((200, 1000), (10, 90), cancel_token.clone()),
            // Sensor 3: Values 100-900, Frequency 1-5 Hz
            sensor3: Sensor::new((200, 1000), (100, 900), cancel_token),
        }
    }

    /// Manually cancel all sensor tasks.
    ///
    /// This will stop all sensors from generating new values.
    pub fn cancel(&self) {
        self.sensor1.cancel();
        self.sensor2.cancel();
        self.sensor3.cancel();
    }
}
