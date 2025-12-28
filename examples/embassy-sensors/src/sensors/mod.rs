// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Sensor task implementations.

pub mod humidity;
pub mod pressure;
pub mod temperature;

pub use humidity::humidity_sensor;
pub use pressure::pressure_sensor;
pub use temperature::temperature_sensor;
