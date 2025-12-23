// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Stream aggregation - combines multiple sensor streams

use crate::integration::WasmTimestamped;
use crate::raw_sensor_value::RawSensorValue;
use fluxion_core::StreamItem;
use fluxion_stream::prelude::*;
use fluxion_stream_time::prelude::*;
use futures::channel::mpsc;
use futures::stream::{Stream, StreamExt};
use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::time::Duration;
use wasm_bindgen_futures::spawn_local;

/// Aggregator that combines three sensor streams with its own task
pub struct StreamAggregator {
    running: Rc<RefCell<bool>>,
}

impl StreamAggregator {
    /// Create a new aggregator
    pub fn new() -> Self {
        Self {
            running: Rc::new(RefCell::new(false)),
        }
    }

    /// Start the aggregation task and return a receiver for the aggregated output
    pub fn start(
        &self,
        stream1: Pin<Box<dyn Stream<Item = WasmTimestamped<RawSensorValue>>>>,
        stream2: Pin<Box<dyn Stream<Item = WasmTimestamped<RawSensorValue>>>>,
        stream3: Pin<Box<dyn Stream<Item = WasmTimestamped<RawSensorValue>>>>,
        throttle_ms: u64,
    ) -> mpsc::UnboundedReceiver<f64> {
        let (tx, rx) = mpsc::unbounded();

        *self.running.borrow_mut() = true;

        let running = self.running.clone();

        spawn_local(async move {
            aggregate_streams(stream1, stream2, stream3, throttle_ms, tx, running).await;
        });

        web_sys::console::log_1(&"✅ StreamAggregator started".into());

        rx
    }

    /// Stop the aggregator gracefully
    pub fn stop(&self) {
        *self.running.borrow_mut() = false;
        web_sys::console::log_1(&"🛑 StreamAggregator stop requested".into());
    }

    /// Check if aggregator is running
    pub fn is_running(&self) -> bool {
        *self.running.borrow()
    }
}

/// Internal: Aggregate three sensor streams into averaged output
async fn aggregate_streams(
    stream1: Pin<Box<dyn Stream<Item = WasmTimestamped<RawSensorValue>>>>,
    stream2: Pin<Box<dyn Stream<Item = WasmTimestamped<RawSensorValue>>>>,
    stream3: Pin<Box<dyn Stream<Item = WasmTimestamped<RawSensorValue>>>>,
    throttle_ms: u64,
    tx: mpsc::UnboundedSender<f64>,
    running: Rc<RefCell<bool>>,
) {
    web_sys::console::log_1(&"📊 Aggregation task started".into());

    // Wrap in StreamItem for Fluxion operators
    let s1 = stream1.into_fluxion_stream();
    let s2 = stream2.into_fluxion_stream();
    let s3 = stream3.into_fluxion_stream();

    // Combine latest values from all three sensors
    let combined = s1
        .combine_latest(vec![s2, s3], |_| true)
        .filter_map(|stream_item| async move {
            if let StreamItem::Value(combined) = stream_item {
                // Extract values and calculate average
                let values = combined.values();
                let avg = (values[0].value + values[1].value + values[2].value) / 3.0;
                Some(StreamItem::Value(avg))
            } else {
                None
            }
        });

    // Throttle output
    let mut throttled = combined.throttle(Duration::from_millis(throttle_ms));

    // Process stream while running
    while *running.borrow() {
        if let Some(item) = throttled.next().await {
            if let StreamItem::Value(avg) = item {
                if tx.unbounded_send(avg).is_err() {
                    web_sys::console::log_1(&"⚠️ Aggregator channel closed".into());
                    break;
                }
            }
        }
    }

    web_sys::console::log_1(&"🛑 Aggregation task stopped".into());
}
