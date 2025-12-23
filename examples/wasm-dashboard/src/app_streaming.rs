// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::chart::Chart;
use crate::sensors::SensorSimulator;
use crate::stream_data::{generate_sensor_stream, SensorValue};
use crate::ui::UI;
use fluxion_core::StreamItem;
use fluxion_stream::prelude::*;
use fluxion_stream_time::prelude::*;
use fluxion_stream_time::runtimes::wasm_implementation::WasmTimer;
use fluxion_stream_time::timer::Timer;
use fluxion_stream_time::InstantTimestamped;
use futures::channel::mpsc;
use futures::stream::StreamExt;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Document, HtmlCanvasElement, Window};

type WasmTimestamped<T> = InstantTimestamped<T, WasmTimer>;

/// Main dashboard application with real streaming
pub struct Dashboard {
    _window: Window,
    document: Document,
    _ui: UI,
    _simulator: SensorSimulator,
    charts: Vec<Chart>,
    output_chart: Chart,
    running: Rc<RefCell<bool>>,
    metrics: Rc<RefCell<Metrics>>,
}

#[derive(Debug, Clone, Default)]
struct Metrics {
    total_received: u32,
    total_emitted: u32,
    last_value: f64,
}

impl Dashboard {
    pub fn new(window: Window, document: Document) -> Result<Self, JsValue> {
        let ui = UI::new(&window, &document)?;
        let simulator = SensorSimulator::new();

        // Create charts for each sensor
        let mut charts = Vec::new();
        for i in 1..=3 {
            let canvas_id = format!("sensor{}Chart", i);
            let canvas = document
                .get_element_by_id(&canvas_id)
                .ok_or_else(|| JsValue::from_str(&format!("Canvas {} not found", canvas_id)))?
                .dyn_into::<HtmlCanvasElement>()?;
            charts.push(Chart::new(canvas)?);
        }

        // Create output chart
        let output_canvas = document
            .get_element_by_id("outputChart")
            .ok_or_else(|| JsValue::from_str("Output canvas not found"))?
            .dyn_into::<HtmlCanvasElement>()?;
        let output_chart = Chart::new(output_canvas)?;

        web_sys::console::log_1(&"✅ Dashboard initialized".into());

        Ok(Self {
            _window: window,
            document,
            _ui: ui,
            _simulator: simulator,
            charts,
            output_chart,
            running: Rc::new(RefCell::new(false)),
            metrics: Rc::new(RefCell::new(Metrics::default())),
        })
    }

    pub async fn run(&mut self) -> Result<(), JsValue> {
        web_sys::console::log_1(&"▶️ Dashboard starting...".into());

        // Setup UI controls with streaming logic
        self.setup_controls()?;

        web_sys::console::log_1(&"✅ Dashboard ready - click 'Start Sensors' to begin".into());

        // Log button availability for debugging
        if let Some(btn) = self.document.get_element_by_id("startBtn") {
            web_sys::console::log_1(&format!("✅ Start button found: {:?}", btn.tag_name()).into());
        } else {
            web_sys::console::log_1(&"❌ Start button NOT found!".into());
        }

        Ok(())
    }

    fn setup_controls(&mut self) -> Result<(), JsValue> {
        let document = self.document.clone();
        let running = self.running.clone();
        let metrics = self.metrics.clone();
        let metrics2 = self.metrics.clone(); // Clone for reset button
        let charts = self.charts.clone();
        let output_chart = self.output_chart.clone();

        // Start button
        let start_btn = document
            .get_element_by_id("startBtn")
            .ok_or("Start button not found")?;

        web_sys::console::log_1(&"🔧 Setting up start button handler...".into());

        let running_clone = running.clone();
        let start_closure = Closure::wrap(Box::new(move || {
            web_sys::console::log_1(&"👆 Start button clicked!".into());

            if *running_clone.borrow() {
                web_sys::console::log_1(&"⚠️ Already running".into());
                return;
            }

            web_sys::console::log_1(&"🚀 Starting sensors...".into());
            *running_clone.borrow_mut() = true;

            // Create sensor streams
            let (tx1, rx1) = mpsc::unbounded();
            let (tx2, rx2) = mpsc::unbounded();
            let (tx3, rx3) = mpsc::unbounded();

            // Spawn sensor generators
            spawn_local(generate_sensor_stream(1, 100.0, tx1, running_clone.clone()));
            spawn_local(generate_sensor_stream(2, 50.0, tx2, running_clone.clone()));
            spawn_local(generate_sensor_stream(3, 25.0, tx3, running_clone.clone()));

            // Start processing pipeline
            let running_clone2 = running_clone.clone();
            let metrics_clone = metrics.clone();
            let charts_clone = charts.clone();
            let output_clone = output_chart.clone();

            spawn_local(async move {
                Self::process_streams(
                    rx1,
                    rx2,
                    rx3,
                    running_clone2,
                    metrics_clone,
                    charts_clone,
                    output_clone,
                )
                .await;
            });
        }) as Box<dyn FnMut()>);

        start_btn
            .add_event_listener_with_callback("click", start_closure.as_ref().unchecked_ref())?;
        start_closure.forget();

        // Stop button
        let stop_btn = document
            .get_element_by_id("stopBtn")
            .ok_or("Stop button not found")?;

        let running_clone = running.clone();
        let stop_closure = Closure::wrap(Box::new(move || {
            web_sys::console::log_1(&"⏹️ Stopping sensors...".into());
            *running_clone.borrow_mut() = false;
        }) as Box<dyn FnMut()>);

        stop_btn
            .add_event_listener_with_callback("click", stop_closure.as_ref().unchecked_ref())?;
        stop_closure.forget();

        // Reset button
        let reset_btn = document
            .get_element_by_id("resetBtn")
            .ok_or("Reset button not found")?;

        let charts_clone = self.charts.clone();
        let output_clone = self.output_chart.clone();

        let reset_closure = Closure::wrap(Box::new(move || {
            web_sys::console::log_1(&"🔄 Resetting...".into());
            *metrics2.borrow_mut() = Metrics::default();

            // Clear charts
            for chart in &charts_clone {
                chart.clear().ok();
            }
            output_clone.clear().ok();
        }) as Box<dyn FnMut()>);

        reset_btn
            .add_event_listener_with_callback("click", reset_closure.as_ref().unchecked_ref())?;
        reset_closure.forget();

        Ok(())
    }

    async fn process_streams(
        rx1: mpsc::UnboundedReceiver<WasmTimestamped<SensorValue>>,
        rx2: mpsc::UnboundedReceiver<WasmTimestamped<SensorValue>>,
        rx3: mpsc::UnboundedReceiver<WasmTimestamped<SensorValue>>,
        running: Rc<RefCell<bool>>,
        metrics: Rc<RefCell<Metrics>>,
        charts: Vec<Chart>,
        mut output_chart: Chart,
    ) {
        web_sys::console::log_1(&"🔄 process_streams started".into());

        // Clone charts for use in closures
        let chart1 = charts[0].clone();
        let chart2 = charts[1].clone();
        let chart3 = charts[2].clone();

        // Wrap receivers in StreamItem::Value and tap to update charts
        let stream1 = rx1.map(move |timestamped| {
            chart1.add_point(timestamped.value.value);
            let _ = chart1.render();
            StreamItem::Value(timestamped)
        });

        let stream2 = rx2.map(move |timestamped| {
            chart2.add_point(timestamped.value.value);
            let _ = chart2.render();
            StreamItem::Value(timestamped)
        });

        let stream3 = rx3.map(move |timestamped| {
            chart3.add_point(timestamped.value.value);
            let _ = chart3.render();
            StreamItem::Value(timestamped)
        });

        web_sys::console::log_1(&"✅ Streams wrapped, starting combine_latest".into());

        // Combine latest from all sensors using fluxion's API
        let combined_stream = stream1
            .combine_latest(vec![stream2, stream3], |_| true)
            .filter_map(|stream_item| async move {
                // Extract CombinedState from StreamItem
                if let StreamItem::Value(combined) = stream_item {
                    // Extract values from combined state (combine_latest unwraps InstantTimestamped)
                    let values = combined.values();
                    let avg = (values[0].value + values[1].value + values[2].value) / 3.0;

                    // Create timestamped output
                    let timer = WasmTimer::new();
                    Some(StreamItem::Value(InstantTimestamped::new(avg, timer.now())))
                } else {
                    web_sys::console::log_1(&"⚠️ Got non-Value StreamItem".into());
                    None
                }
            });

        web_sys::console::log_1(&"✅ Combined stream created, applying throttle".into());

        // Apply time operators (using convenience methods - they automatically use WasmTimer)
        // Use throttle instead of debounce - throttle emits at most once per period
        // while debounce waits for quiet periods (which never happen with constant sensor data!)
        let mut processed = combined_stream.throttle(Duration::from_millis(100));

        web_sys::console::log_1(&"✅ Throttle applied, starting processing loop".into());

        // Process stream and update UI
        web_sys::console::log_1(&"🔄 Starting stream processing loop".into());
        let mut count = 0;

        while *running.borrow() {
            if let Some(item) = processed.next().await {
                count += 1;
                if count == 1 {
                    web_sys::console::log_1(&"🎉 First item received!".into());
                }
                if count % 5 == 0 {
                    web_sys::console::log_1(&format!("📊 Processed {} items", count).into());
                }

                // Extract value from StreamItem
                if let StreamItem::Value(timestamped) = item {
                    let avg = timestamped.value;

                    // Update metrics
                    {
                        let mut m = metrics.borrow_mut();
                        m.total_received += 1;
                        m.total_emitted += 1;
                        m.last_value = avg;
                    }

                    // Update output chart
                    output_chart.add_point(avg);
                    if let Err(e) = output_chart.render() {
                        web_sys::console::log_1(&format!("❌ Chart render error: {:?}", e).into());
                    }

                    // Update metrics display
                    Self::update_metrics_display(&metrics.borrow());
                }
            }
        }

        web_sys::console::log_1(&"✅ Stream processing stopped".into());
    }

    fn update_metrics_display(metrics: &Metrics) {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                // Update received count
                if let Some(elem) = document.get_element_by_id("eventsReceived") {
                    elem.set_inner_html(&metrics.total_received.to_string());
                }

                // Update emitted count
                if let Some(elem) = document.get_element_by_id("eventsEmitted") {
                    elem.set_inner_html(&metrics.total_emitted.to_string());
                }

                // Calculate drop rate
                let drop_rate = if metrics.total_received > 0 {
                    let dropped = metrics.total_received.saturating_sub(metrics.total_emitted);
                    (dropped as f64 / metrics.total_received as f64 * 100.0).round()
                } else {
                    0.0
                };

                if let Some(elem) = document.get_element_by_id("dropRate") {
                    elem.set_inner_html(&format!("{}%", drop_rate));
                }

                // Update last value
                if let Some(elem) = document.get_element_by_id("avgValue") {
                    elem.set_inner_html(&format!("{:.3}", metrics.last_value));
                }
            }
        }
    }
}
