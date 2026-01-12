// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::gui::DashboardSink;
use crate::processing::{StreamProvider, WasmStream};
use crate::source::SensorValue;
use fluxion_core::{CancellationToken, FluxionTask, StreamItem};
use fluxion_exec::SubscribeExt;
use fluxion_stream::share::SharedBoxStream;
use fluxion_stream_time::WasmTimestamped;
use std::cell::RefCell;
use std::convert::Infallible;
use std::rc::Rc;
use web_sys::console;

/// Pure orchestrator - Wires streams to UI updates
///
/// This component knows nothing about concrete implementations. It only
/// knows how to:
/// - Get streams from a StreamProvider
/// - Update a DashboardSink
/// - Spawn FluxionTasks to wire them together
///
/// Generic over both the stream source and the UI target, making it
/// completely testable and reusable.
pub struct DashboardOrchestrator<P: StreamProvider, S: DashboardSink + 'static> {
    provider: P,
    sink: Rc<RefCell<S>>,
    stop_token: CancellationToken,
    tasks: Vec<FluxionTask>,
}

impl<P: StreamProvider, S: DashboardSink + 'static> DashboardOrchestrator<P, S> {
    /// Creates a new orchestrator
    ///
    /// # Arguments
    ///
    /// * `provider` - Stream provider implementing StreamProvider trait
    /// * `sink` - UI sink implementing DashboardSink trait
    /// * `stop_token` - Token to stop all update tasks
    pub fn new(provider: P, sink: Rc<RefCell<S>>, stop_token: CancellationToken) -> Self {
        Self {
            provider,
            sink,
            stop_token,
            tasks: Vec::new(),
        }
    }

    /// Runs the dashboard updater, spawning tasks and blocking until cancellation
    ///
    /// This method gets streams from the provider, wires them to the sink,
    /// and then blocks waiting for the cancellation token. When cancelled,
    /// it returns and all tasks are dropped.
    pub async fn run(mut self) {
        // Wire all 9 streams to their corresponding sink methods
        self.tasks.push(Self::wire_sensor1(
            self.provider.sensor1_stream(),
            self.sink.clone(),
            self.stop_token.clone(),
        ));
        self.tasks.push(Self::wire_sensor2(
            self.provider.sensor2_stream(),
            self.sink.clone(),
            self.stop_token.clone(),
        ));
        self.tasks.push(Self::wire_sensor3(
            self.provider.sensor3_stream(),
            self.sink.clone(),
            self.stop_token.clone(),
        ));
        self.tasks.push(Self::wire_combined(
            self.provider.combined_stream(),
            self.sink.clone(),
            self.stop_token.clone(),
        ));
        self.tasks.push(Self::wire_debounce(
            self.provider.debounce_stream(),
            self.sink.clone(),
            self.stop_token.clone(),
        ));
        self.tasks.push(Self::wire_delay(
            self.provider.delay_stream(),
            self.sink.clone(),
            self.stop_token.clone(),
        ));
        self.tasks.push(Self::wire_sample(
            self.provider.sample_stream(),
            self.sink.clone(),
            self.stop_token.clone(),
        ));
        self.tasks.push(Self::wire_throttle(
            self.provider.throttle_stream(),
            self.sink.clone(),
            self.stop_token.clone(),
        ));
        self.tasks.push(Self::wire_timeout(
            self.provider.timeout_stream(),
            self.sink.clone(),
            self.stop_token.clone(),
        ));

        // Block until cancellation token is triggered
        self.stop_token.cancelled().await;

        console::log_1(&"🛑 Dashboard shutting down...".into());

        // Tasks are automatically cancelled and dropped here
        drop(self.tasks);
    }

    fn wire_sensor1(
        stream: SharedBoxStream<SensorValue>,
        sink: Rc<RefCell<S>>,
        stop_token: CancellationToken,
    ) -> FluxionTask {
        FluxionTask::spawn(move |_| async move {
            let _ = stream
                .subscribe(
                    move |item, _| {
                        let sink = sink.clone();
                        async move {
                            match item {
                                StreamItem::Value(sensor_value) => {
                                    sink.borrow_mut().update_sensor1(sensor_value.value);
                                }
                                StreamItem::Error(e) => {
                                    console::error_1(
                                        &format!("Sensor 1 stream error: {:?}", e).into(),
                                    );
                                }
                            }
                            Ok::<_, Infallible>(())
                        }
                    },
                    |err| console::error_1(&format!("Sensor 1 error: {:?}", err).into()),
                    Some(stop_token),
                )
                .await;
        })
    }

    fn wire_sensor2(
        stream: SharedBoxStream<SensorValue>,
        sink: Rc<RefCell<S>>,
        stop_token: CancellationToken,
    ) -> FluxionTask {
        FluxionTask::spawn(move |_| async move {
            let _ = stream
                .subscribe(
                    move |item, _| {
                        let sink = sink.clone();
                        async move {
                            match item {
                                StreamItem::Value(sensor_value) => {
                                    sink.borrow_mut().update_sensor2(sensor_value.value);
                                }
                                StreamItem::Error(e) => {
                                    console::error_1(
                                        &format!("Sensor 2 stream error: {:?}", e).into(),
                                    );
                                }
                            }
                            Ok::<_, Infallible>(())
                        }
                    },
                    |err| console::error_1(&format!("Sensor 2 error: {:?}", err).into()),
                    Some(stop_token),
                )
                .await;
        })
    }

    fn wire_sensor3(
        stream: SharedBoxStream<SensorValue>,
        sink: Rc<RefCell<S>>,
        stop_token: CancellationToken,
    ) -> FluxionTask {
        FluxionTask::spawn(move |_| async move {
            let _ = stream
                .subscribe(
                    move |item, _| {
                        let sink = sink.clone();
                        async move {
                            match item {
                                StreamItem::Value(sensor_value) => {
                                    sink.borrow_mut().update_sensor3(sensor_value.value);
                                }
                                StreamItem::Error(e) => {
                                    console::error_1(
                                        &format!("Sensor 3 stream error: {:?}", e).into(),
                                    );
                                }
                            }
                            Ok::<_, Infallible>(())
                        }
                    },
                    |err| console::error_1(&format!("Sensor 3 error: {:?}", err).into()),
                    Some(stop_token),
                )
                .await;
        })
    }

    fn wire_combined(
        stream: SharedBoxStream<WasmTimestamped<u32>>,
        sink: Rc<RefCell<S>>,
        stop_token: CancellationToken,
    ) -> FluxionTask {
        FluxionTask::spawn(move |_| async move {
            let _ = stream
                .subscribe(
                    move |item, _| {
                        let sink = sink.clone();
                        async move {
                            match item {
                                StreamItem::Value(sum) => {
                                    sink.borrow_mut().update_combined(sum.value);
                                }
                                StreamItem::Error(e) => {
                                    console::error_1(
                                        &format!("Combined stream error: {:?}", e).into(),
                                    );
                                }
                            }
                            Ok::<_, Infallible>(())
                        }
                    },
                    |err| console::error_1(&format!("Combined stream error: {:?}", err).into()),
                    Some(stop_token),
                )
                .await;
        })
    }

    fn wire_debounce(
        stream: WasmStream<u32>,
        sink: Rc<RefCell<S>>,
        stop_token: CancellationToken,
    ) -> FluxionTask {
        FluxionTask::spawn(move |_| async move {
            let _ = stream
                .subscribe(
                    move |item, _| {
                        let sink = sink.clone();
                        async move {
                            match item {
                                StreamItem::Value(value) => {
                                    sink.borrow_mut().update_debounce(value);
                                }
                                StreamItem::Error(e) => {
                                    console::error_1(
                                        &format!("Debounce stream error: {:?}", e).into(),
                                    );
                                }
                            }
                            Ok::<_, Infallible>(())
                        }
                    },
                    |err| console::error_1(&format!("Debounce stream error: {:?}", err).into()),
                    Some(stop_token),
                )
                .await;
        })
    }

    fn wire_delay(
        stream: WasmStream<u32>,
        sink: Rc<RefCell<S>>,
        stop_token: CancellationToken,
    ) -> FluxionTask {
        FluxionTask::spawn(move |_| async move {
            let _ = stream
                .subscribe(
                    move |item, _| {
                        let sink = sink.clone();
                        async move {
                            match item {
                                StreamItem::Value(value) => {
                                    sink.borrow_mut().update_delay(value);
                                }
                                StreamItem::Error(e) => {
                                    console::error_1(
                                        &format!("Delay stream error: {:?}", e).into(),
                                    );
                                }
                            }
                            Ok::<_, Infallible>(())
                        }
                    },
                    |err| console::error_1(&format!("Delay stream error: {:?}", err).into()),
                    Some(stop_token),
                )
                .await;
        })
    }

    fn wire_sample(
        stream: WasmStream<u32>,
        sink: Rc<RefCell<S>>,
        stop_token: CancellationToken,
    ) -> FluxionTask {
        FluxionTask::spawn(move |_| async move {
            let _ = stream
                .subscribe(
                    move |item, _| {
                        let sink = sink.clone();
                        async move {
                            match item {
                                StreamItem::Value(value) => {
                                    sink.borrow_mut().update_sample(value);
                                }
                                StreamItem::Error(e) => {
                                    console::error_1(
                                        &format!("Sample stream error: {:?}", e).into(),
                                    );
                                }
                            }
                            Ok::<_, Infallible>(())
                        }
                    },
                    |err| console::error_1(&format!("Sample stream error: {:?}", err).into()),
                    Some(stop_token),
                )
                .await;
        })
    }

    fn wire_throttle(
        stream: WasmStream<u32>,
        sink: Rc<RefCell<S>>,
        stop_token: CancellationToken,
    ) -> FluxionTask {
        FluxionTask::spawn(move |_| async move {
            let _ = stream
                .subscribe(
                    move |item, _| {
                        let sink = sink.clone();
                        async move {
                            match item {
                                StreamItem::Value(value) => {
                                    sink.borrow_mut().update_throttle(value);
                                }
                                StreamItem::Error(e) => {
                                    console::error_1(
                                        &format!("Throttle stream error: {:?}", e).into(),
                                    );
                                }
                            }
                            Ok::<_, Infallible>(())
                        }
                    },
                    |err| console::error_1(&format!("Throttle stream error: {:?}", err).into()),
                    Some(stop_token),
                )
                .await;
        })
    }

    fn wire_timeout(
        stream: WasmStream<u32>,
        sink: Rc<RefCell<S>>,
        stop_token: CancellationToken,
    ) -> FluxionTask {
        FluxionTask::spawn(move |_| async move {
            let _ = stream
                .subscribe(
                    move |item, _| {
                        let sink = sink.clone();
                        async move {
                            match item {
                                StreamItem::Value(value) => {
                                    sink.borrow_mut().update_timeout(value);
                                }
                                StreamItem::Error(e) => {
                                    console::error_1(
                                        &format!("Timeout stream error: {:?}", e).into(),
                                    );
                                    sink.borrow_mut().show_timeout_error(&format!("{:?}", e));
                                }
                            }
                            Ok::<_, Infallible>(())
                        }
                    },
                    |err| console::error_1(&format!("Timeout stream error: {:?}", err).into()),
                    Some(stop_token),
                )
                .await;
        })
    }
}
