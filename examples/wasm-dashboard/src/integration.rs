// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::raw_sensor_value::RawSensorValue;
use fluxion_stream_time::prelude::*;
use fluxion_stream_time::timer::Timer;
use futures::channel::mpsc;
use futures::stream::{Stream, StreamExt};
use std::pin::Pin;
use std::rc::Rc;

pub type WasmTimestamped<T> = InstantTimestamped<T, WasmTimer>;

/// Augment raw sensor stream with timestamps
pub fn timestamped_sensor_stream(
    rx: mpsc::UnboundedReceiver<RawSensorValue>,
) -> Pin<Box<dyn Stream<Item = WasmTimestamped<RawSensorValue>>>> {
    let timer = Rc::new(WasmTimer::new());

    Box::pin(rx.map(move |value| {
        let t = timer.now();
        WasmTimestamped::new(value, t)
    }))
}
