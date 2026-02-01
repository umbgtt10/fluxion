// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::sensor::Sensor;
use fluxion_core::CancellationToken;

pub struct Sensors {
    pub sensor1: Sensor,
    pub sensor2: Sensor,
    pub sensor3: Sensor,
}

impl Sensors {
    pub fn new(cancel_token: CancellationToken) -> Self {
        Self {
            sensor1: Sensor::new((200, 1000), (1, 9), cancel_token.clone()),
            sensor2: Sensor::new((200, 1000), (10, 90), cancel_token.clone()),
            sensor3: Sensor::new((200, 1000), (100, 900), cancel_token),
        }
    }
}
