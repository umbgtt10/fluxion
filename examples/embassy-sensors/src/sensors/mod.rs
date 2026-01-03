// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Sensor task implementations.

pub mod humidity;
pub mod pressure;
pub mod temperature;

pub use humidity::humidity_sensor;
pub use pressure::pressure_sensor;
pub use temperature::temperature_sensor;
