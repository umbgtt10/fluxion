// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::gui::DashboardUI;
use crate::processing::WasmStream;
use crate::source::{SensorStreams, SensorValue};
use crate::CombinedStream;
use fluxion_core::{CancellationToken, FluxionTask, StreamItem};
use fluxion_exec::SubscribeExt;
use fluxion_stream::fluxion_shared::SharedBoxStream;
use fluxion_stream_time::WasmTimestamped;
use std::cell::RefCell;
use std::rc::Rc;

/// Manages subscriptions that wire streams to GUI updates
///
/// Creates and manages background tasks that consume stream data
/// and update the corresponding GUI windows.
pub struct DashboardUpdater {
    streams: Vec<SharedBoxStream<SensorValue>>,
    combined_stream: SharedBoxStream<WasmTimestamped<u32>>,
    debounce_stream: WasmStream<u32>,
    delay_stream: WasmStream<u32>,
    sample_stream: WasmStream<u32>,
    throttle_stream: WasmStream<u32>,
    timeout_stream: WasmStream<u32>,
    ui: Rc<RefCell<DashboardUI>>,
    cancel_token: CancellationToken,
    tasks: Vec<FluxionTask>,
}

impl DashboardUpdater {
    /// Creates a new updater (does not start tasks yet)
    ///
    /// # Arguments
    ///
    /// * `sensor_streams` - Shared sensor streams container
    /// * `combined_stream` - Subscription to the combined/filtered stream
    /// * `ui` - Shared dashboard UI instance
    /// * `cancel_token` - Token to stop all update tasks
    pub fn new(
        sensor_streams: &SensorStreams,
        combined_stream: &CombinedStream,
        debounce_stream: WasmStream<u32>,
        delay_stream: WasmStream<u32>,
        sample_stream: WasmStream<u32>,
        throttle_stream: WasmStream<u32>,
        timeout_stream: WasmStream<u32>,
        ui: Rc<RefCell<DashboardUI>>,
        cancel_token: CancellationToken,
    ) -> Self {
        // Get independent subscriptions for each sensor stream and combined stream
        let streams = sensor_streams.subscribe();
        let combined_stream = combined_stream.subscribe();

        Self {
            streams,
            combined_stream,
            debounce_stream,
            delay_stream,
            sample_stream,
            throttle_stream,
            timeout_stream,
            ui,
            cancel_token,
            tasks: Vec::new(),
        }
    }

    /// Runs the dashboard updater, spawning tasks and blocking until cancellation
    ///
    /// This method spawns background tasks for each sensor stream and then
    /// blocks waiting for the cancellation token to be triggered (e.g., by
    /// the close button). When cancelled, it returns and all tasks are dropped.
    pub async fn run(mut self) {
        let mut streams = self.streams.into_iter();

        // Wire sensor 1
        if let Some(stream) = streams.next() {
            let task = Self::wire_sensor1(stream, self.ui.clone(), self.cancel_token.clone());
            self.tasks.push(task);
        }

        // Wire sensor 2
        if let Some(stream) = streams.next() {
            let task = Self::wire_sensor2(stream, self.ui.clone(), self.cancel_token.clone());
            self.tasks.push(task);
        }

        // Wire sensor 3
        if let Some(stream) = streams.next() {
            let task = Self::wire_sensor3(stream, self.ui.clone(), self.cancel_token.clone());
            self.tasks.push(task);
        }

        // Wire combined stream
        let task = Self::wire_combined(
            self.combined_stream,
            self.ui.clone(),
            self.cancel_token.clone(),
        );
        self.tasks.push(task);

        let task = Self::wire_debounce(
            self.debounce_stream,
            self.ui.clone(),
            self.cancel_token.clone(),
        );
        self.tasks.push(task);

        let task = Self::wire_delay(
            self.delay_stream,
            self.ui.clone(),
            self.cancel_token.clone(),
        );
        self.tasks.push(task);

        let task = Self::wire_sample(
            self.sample_stream,
            self.ui.clone(),
            self.cancel_token.clone(),
        );
        self.tasks.push(task);

        let task = Self::wire_throttle(
            self.throttle_stream,
            self.ui.clone(),
            self.cancel_token.clone(),
        );
        self.tasks.push(task);

        let task = Self::wire_timeout(
            self.timeout_stream,
            self.ui.clone(),
            self.cancel_token.clone(),
        );
        self.tasks.push(task);

        // Block until cancellation token is triggered
        self.cancel_token.cancelled().await;

        web_sys::console::log_1(&"🛑 Dashboard shutting down...".into());

        // Tasks are automatically cancelled and dropped here
        drop(self.tasks);
    }

    fn wire_sensor1(
        stream: SharedBoxStream<SensorValue>,
        ui: Rc<RefCell<DashboardUI>>,
        cancel_token: CancellationToken,
    ) -> FluxionTask {
        FluxionTask::spawn(move |_task_cancel| async move {
            let _ = stream
                .subscribe(
                    move |item, _token| {
                        let ui = ui.clone();
                        async move {
                            match item {
                                StreamItem::Value(sensor_value) => {
                                    ui.borrow_mut().update_sensor1(sensor_value.value);
                                }
                                StreamItem::Error(e) => {
                                    web_sys::console::error_1(
                                        &format!("Sensor 1 stream error: {:?}", e).into(),
                                    );
                                }
                            }
                            Ok::<_, std::convert::Infallible>(())
                        }
                    },
                    |err| web_sys::console::error_1(&format!("Sensor 1 error: {:?}", err).into()),
                    Some(cancel_token),
                )
                .await;
        })
    }

    fn wire_sensor2(
        stream: SharedBoxStream<SensorValue>,
        ui: Rc<RefCell<DashboardUI>>,
        cancel_token: CancellationToken,
    ) -> FluxionTask {
        FluxionTask::spawn(move |_task_cancel| async move {
            let _ = stream
                .subscribe(
                    move |item, _token| {
                        let ui = ui.clone();
                        async move {
                            match item {
                                StreamItem::Value(sensor_value) => {
                                    ui.borrow_mut().update_sensor2(sensor_value.value);
                                }
                                StreamItem::Error(e) => {
                                    web_sys::console::error_1(
                                        &format!("Sensor 2 stream error: {:?}", e).into(),
                                    );
                                }
                            }
                            Ok::<_, std::convert::Infallible>(())
                        }
                    },
                    |err| web_sys::console::error_1(&format!("Sensor 2 error: {:?}", err).into()),
                    Some(cancel_token),
                )
                .await;
        })
    }

    fn wire_sensor3(
        stream: SharedBoxStream<SensorValue>,
        ui: Rc<RefCell<DashboardUI>>,
        cancel_token: CancellationToken,
    ) -> FluxionTask {
        FluxionTask::spawn(move |_task_cancel| async move {
            let _ = stream
                .subscribe(
                    move |item, _token| {
                        let ui = ui.clone();
                        async move {
                            match item {
                                StreamItem::Value(sensor_value) => {
                                    ui.borrow_mut().update_sensor3(sensor_value.value);
                                }
                                StreamItem::Error(e) => {
                                    web_sys::console::error_1(
                                        &format!("Sensor 3 stream error: {:?}", e).into(),
                                    );
                                }
                            }
                            Ok::<_, std::convert::Infallible>(())
                        }
                    },
                    |err| web_sys::console::error_1(&format!("Sensor 3 error: {:?}", err).into()),
                    Some(cancel_token),
                )
                .await;
        })
    }

    fn wire_combined(
        stream: SharedBoxStream<WasmTimestamped<u32>>,
        ui: Rc<RefCell<DashboardUI>>,
        cancel_token: CancellationToken,
    ) -> FluxionTask {
        FluxionTask::spawn(move |_task_cancel| async move {
            let _ = stream
                .subscribe(
                    move |item, _token| {
                        let ui = ui.clone();
                        async move {
                            match item {
                                StreamItem::Value(sum) => {
                                    ui.borrow_mut().update_combined(sum.value);
                                }
                                StreamItem::Error(e) => {
                                    web_sys::console::error_1(
                                        &format!("Combined stream error: {:?}", e).into(),
                                    );
                                }
                            }
                            Ok::<_, std::convert::Infallible>(())
                        }
                    },
                    |err| {
                        web_sys::console::error_1(
                            &format!("Combined stream error: {:?}", err).into(),
                        )
                    },
                    Some(cancel_token),
                )
                .await;
        })
    }

    fn wire_debounce(
        stream: WasmStream<u32>,
        ui: Rc<RefCell<DashboardUI>>,
        cancel_token: CancellationToken,
    ) -> FluxionTask {
        FluxionTask::spawn(move |_task_cancel| async move {
            let _ = stream
                .subscribe(
                    move |item, _token| {
                        let ui = ui.clone();
                        async move {
                            match item {
                                StreamItem::Value(value) => {
                                    ui.borrow_mut().update_debounce(value);
                                }
                                StreamItem::Error(e) => {
                                    web_sys::console::error_1(
                                        &format!("Debounce stream error: {:?}", e).into(),
                                    );
                                }
                            }
                            Ok::<_, std::convert::Infallible>(())
                        }
                    },
                    |err| {
                        web_sys::console::error_1(
                            &format!("Debounce stream error: {:?}", err).into(),
                        )
                    },
                    Some(cancel_token),
                )
                .await;
        })
    }

    pub fn wire_delay(
        stream: WasmStream<u32>,
        ui: Rc<RefCell<DashboardUI>>,
        cancel_token: CancellationToken,
    ) -> FluxionTask {
        FluxionTask::spawn(move |_task_cancel| async move {
            let _ = stream
                .subscribe(
                    move |item, _token| {
                        let ui = ui.clone();
                        async move {
                            match item {
                                StreamItem::Value(value) => {
                                    ui.borrow_mut().update_delay(value);
                                }
                                StreamItem::Error(e) => {
                                    web_sys::console::error_1(
                                        &format!("Delay stream error: {:?}", e).into(),
                                    );
                                }
                            }
                            Ok::<_, std::convert::Infallible>(())
                        }
                    },
                    |err| {
                        web_sys::console::error_1(&format!("Delay stream error: {:?}", err).into())
                    },
                    Some(cancel_token),
                )
                .await;
        })
    }

    pub fn wire_sample(
        stream: WasmStream<u32>,
        ui: Rc<RefCell<DashboardUI>>,
        cancel_token: CancellationToken,
    ) -> FluxionTask {
        FluxionTask::spawn(move |_task_cancel| async move {
            let _ = stream
                .subscribe(
                    move |item, _token| {
                        let ui = ui.clone();
                        async move {
                            match item {
                                StreamItem::Value(value) => {
                                    ui.borrow_mut().update_sample(value);
                                }
                                StreamItem::Error(e) => {
                                    web_sys::console::error_1(
                                        &format!("Sample stream error: {:?}", e).into(),
                                    );
                                }
                            }
                            Ok::<_, std::convert::Infallible>(())
                        }
                    },
                    |err| {
                        web_sys::console::error_1(&format!("Sample stream error: {:?}", err).into())
                    },
                    Some(cancel_token),
                )
                .await;
        })
    }

    pub fn wire_throttle(
        stream: WasmStream<u32>,
        ui: Rc<RefCell<DashboardUI>>,
        cancel_token: CancellationToken,
    ) -> FluxionTask {
        FluxionTask::spawn(move |_task_cancel| async move {
            let _ = stream
                .subscribe(
                    move |item, _token| {
                        let ui = ui.clone();
                        async move {
                            match item {
                                StreamItem::Value(value) => {
                                    ui.borrow_mut().update_throttle(value);
                                }
                                StreamItem::Error(e) => {
                                    web_sys::console::error_1(
                                        &format!("Throttle stream error: {:?}", e).into(),
                                    );
                                }
                            }
                            Ok::<_, std::convert::Infallible>(())
                        }
                    },
                    |err| {
                        web_sys::console::error_1(
                            &format!("Throttle stream error: {:?}", err).into(),
                        )
                    },
                    Some(cancel_token),
                )
                .await;
        })
    }

    pub fn wire_timeout(
        stream: WasmStream<u32>,
        ui: Rc<RefCell<DashboardUI>>,
        cancel_token: CancellationToken,
    ) -> FluxionTask {
        FluxionTask::spawn(move |_task_cancel| async move {
            let _ = stream
                .subscribe(
                    move |item, _token| {
                        let ui = ui.clone();
                        async move {
                            match item {
                                StreamItem::Value(value) => {
                                    ui.borrow_mut().update_timeout(value);
                                }
                                StreamItem::Error(e) => {
                                    web_sys::console::error_1(
                                        &format!("Timeout stream error: {:?}", e).into(),
                                    );
                                    ui.borrow_mut().update_timeout_error(&format!("{:?}", e));
                                }
                            }
                            Ok::<_, std::convert::Infallible>(())
                        }
                    },
                    |err| {
                        web_sys::console::error_1(
                            &format!("Timeout stream error: {:?}", err).into(),
                        )
                    },
                    Some(cancel_token),
                )
                .await;
        })
    }
}
