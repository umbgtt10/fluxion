// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::gui::DashboardUI;
use crate::source::{SensorStreams, SensorValue};
use fluxion_core::{CancellationToken, FluxionTask, StreamItem};
use fluxion_stream::fluxion_shared::SharedBoxStream;
use futures::StreamExt;
use std::cell::RefCell;
use std::rc::Rc;

/// Manages subscriptions that wire streams to GUI updates
///
/// Creates and manages background tasks that consume stream data
/// and update the corresponding GUI windows.
pub struct DashboardUpdater {
    streams: Vec<SharedBoxStream<SensorValue>>,
    ui: Rc<RefCell<DashboardUI>>,
    cancel_token: CancellationToken,
}

impl DashboardUpdater {
    /// Creates a new updater (does not start tasks yet)
    ///
    /// # Arguments
    ///
    /// * `sensor_streams` - Shared sensor streams container
    /// * `ui` - Shared dashboard UI instance
    /// * `cancel_token` - Token to stop all update tasks
    pub fn new(
        sensor_streams: &SensorStreams,
        ui: Rc<RefCell<DashboardUI>>,
        cancel_token: CancellationToken,
    ) -> Self {
        // Get independent subscriptions for each sensor
        let streams = sensor_streams.subscribe();

        Self {
            streams,
            ui,
            cancel_token,
        }
    }

    /// Runs the dashboard updater, spawning tasks and blocking until cancellation
    ///
    /// This method spawns background tasks for each sensor stream and then
    /// blocks waiting for the cancellation token to be triggered (e.g., by
    /// the close button). When cancelled, it returns and all tasks are dropped.
    pub async fn run(self) {
        let mut streams = self.streams.into_iter();

        // Wire sensor 1
        if let Some(stream) = streams.next() {
            Self::wire_sensor1(stream, self.ui.clone(), self.cancel_token.clone());
        }

        // Wire sensor 2
        if let Some(stream) = streams.next() {
            Self::wire_sensor2(stream, self.ui.clone(), self.cancel_token.clone());
        }

        // Wire sensor 3
        if let Some(stream) = streams.next() {
            Self::wire_sensor3(stream, self.ui.clone(), self.cancel_token.clone());
        }

        // Block until cancellation token is triggered
        self.cancel_token.cancelled().await;

        web_sys::console::log_1(&"🛑 Dashboard shutting down...".into());

        // Tasks are automatically dropped here, triggering cleanup
    }

    fn wire_sensor1(
        mut stream: fluxion_stream::fluxion_shared::SharedBoxStream<SensorValue>,
        ui: Rc<RefCell<DashboardUI>>,
        cancel_token: CancellationToken,
    ) {
        wasm_bindgen_futures::spawn_local(async move {
            while !cancel_token.is_cancelled() {
                match stream.next().await {
                    Some(StreamItem::Value(sensor_value)) => {
                        ui.borrow_mut().update_sensor1(sensor_value.value);
                    }
                    Some(StreamItem::Error(e)) => {
                        web_sys::console::error_1(&format!("Sensor 1 error: {:?}", e).into());
                    }
                    None => break,
                }
            }
        });
    }

    fn wire_sensor2(
        mut stream: fluxion_stream::fluxion_shared::SharedBoxStream<SensorValue>,
        ui: Rc<RefCell<DashboardUI>>,
        cancel_token: CancellationToken,
    ) {
        wasm_bindgen_futures::spawn_local(async move {
            while !cancel_token.is_cancelled() {
                match stream.next().await {
                    Some(StreamItem::Value(sensor_value)) => {
                        ui.borrow_mut().update_sensor2(sensor_value.value);
                    }
                    Some(StreamItem::Error(e)) => {
                        web_sys::console::error_1(&format!("Sensor 2 error: {:?}", e).into());
                    }
                    None => break,
                }
            }
        });
    }

    fn wire_sensor3(
        mut stream: fluxion_stream::fluxion_shared::SharedBoxStream<SensorValue>,
        ui: Rc<RefCell<DashboardUI>>,
        cancel_token: CancellationToken,
    ) {
        wasm_bindgen_futures::spawn_local(async move {
            while !cancel_token.is_cancelled() {
                match stream.next().await {
                    Some(StreamItem::Value(sensor_value)) => {
                        ui.borrow_mut().update_sensor3(sensor_value.value);
                    }
                    Some(StreamItem::Error(e)) => {
                        web_sys::console::error_1(&format!("Sensor 3 error: {:?}", e).into());
                    }
                    None => break,
                }
            }
        });
    }
}
