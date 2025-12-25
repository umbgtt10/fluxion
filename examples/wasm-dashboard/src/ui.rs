// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlButtonElement, HtmlElement, Window};

use crate::sensors::SensorSimulator;

/// Shared dashboard UI with 11 hooking points (9 windows + 2 buttons)
pub struct DashboardUI {
    document: Document,
    // 3 sensor windows
    sensor1_window: HtmlElement,
    sensor2_window: HtmlElement,
    sensor3_window: HtmlElement,
    // 1 combined stream window
    combined_window: HtmlElement,
    // 5 time-bound operator windows
    debounce_window: HtmlElement,
    throttle_window: HtmlElement,
    buffer_window: HtmlElement,
    sample_window: HtmlElement,
    window_window: HtmlElement,
    // 2 control buttons
    start_button: HtmlButtonElement,
    stop_button: HtmlButtonElement,
}

impl DashboardUI {
    pub fn new(document: &Document) -> Result<Rc<RefCell<Self>>, JsValue> {
        let ui = Self {
            document: document.clone(),
            // Get sensor windows
            sensor1_window: Self::get_element(document, "sensor1Window")?,
            sensor2_window: Self::get_element(document, "sensor2Window")?,
            sensor3_window: Self::get_element(document, "sensor3Window")?,
            // Get combined window
            combined_window: Self::get_element(document, "combinedWindow")?,
            // Get operator windows
            debounce_window: Self::get_element(document, "debounceWindow")?,
            throttle_window: Self::get_element(document, "throttleWindow")?,
            buffer_window: Self::get_element(document, "bufferWindow")?,
            sample_window: Self::get_element(document, "sampleWindow")?,
            window_window: Self::get_element(document, "windowWindow")?,
            // Get buttons
            start_button: Self::get_button(document, "startBtn")?,
            stop_button: Self::get_button(document, "stopBtn")?,
        };

        Ok(Rc::new(RefCell::new(ui)))
    }

    fn get_element(document: &Document, id: &str) -> Result<HtmlElement, JsValue> {
        document
            .get_element_by_id(id)
            .ok_or_else(|| JsValue::from_str(&format!("Element {} not found", id)))?
            .dyn_into::<HtmlElement>()
    }

    fn get_button(document: &Document, id: &str) -> Result<HtmlButtonElement, JsValue> {
        document
            .get_element_by_id(id)
            .ok_or_else(|| JsValue::from_str(&format!("Button {} not found", id)))?
            .dyn_into::<HtmlButtonElement>()
    }

    // Update methods for sensor windows (3)
    pub fn update_sensor1(&mut self, value: u32) {
        self.sensor1_window
            .set_text_content(Some(&format!("Sensor 1: {}", value)));
    }

    pub fn update_sensor2(&mut self, value: u32) {
        self.sensor2_window
            .set_text_content(Some(&format!("Sensor 2: {}", value)));
    }

    pub fn update_sensor3(&mut self, value: u32) {
        self.sensor3_window
            .set_text_content(Some(&format!("Sensor 3: {}", value)));
    }

    // Update method for combined window (1)
    pub fn update_combined(&mut self, value: u32) {
        self.combined_window
            .set_text_content(Some(&format!("Combined: {}", value)));
    }

    // Update methods for time-bound operator windows (5)
    pub fn update_debounce(&mut self, value: u32) {
        self.debounce_window
            .set_text_content(Some(&format!("Debounce: {}", value)));
    }

    pub fn update_throttle(&mut self, value: u32) {
        self.throttle_window
            .set_text_content(Some(&format!("Throttle: {}", value)));
    }

    pub fn update_buffer(&mut self, values: &[u32]) {
        self.buffer_window.set_text_content(Some(&format!(
            "Buffer: [{} items] {:?}",
            values.len(),
            values
        )));
    }

    pub fn update_sample(&mut self, value: u32) {
        self.sample_window
            .set_text_content(Some(&format!("Sample: {}", value)));
    }

    pub fn update_window(&mut self, count: usize) {
        self.window_window
            .set_text_content(Some(&format!("Window: {} items", count)));
    }

    // Button control methods
    pub fn enable_start(&mut self) {
        self.start_button.set_disabled(false);
        self.stop_button.set_disabled(true);
    }

    pub fn enable_stop(&mut self) {
        self.start_button.set_disabled(true);
        self.stop_button.set_disabled(false);
    }
}

/// Legacy UI manager for the dashboard
pub struct UI {
    _window: Window,
    document: Document,
}

impl UI {
    pub fn new(window: &Window, document: &Document) -> Result<Self, JsValue> {
        Ok(Self {
            _window: window.clone(),
            document: document.clone(),
        })
    }

    #[allow(dead_code)]
    pub fn setup_controls(&self, _simulator: &mut SensorSimulator) -> Result<(), JsValue> {
        // Start button
        self.setup_button("startBtn", || {
            web_sys::console::log_1(&"🚀 Starting sensors...".into());
        })?;

        // Stop button
        self.setup_button("stopBtn", || {
            web_sys::console::log_1(&"⏹️ Stopping sensors...".into());
        })?;

        // Reset button
        self.setup_button("resetBtn", || {
            web_sys::console::log_1(&"🔄 Resetting dashboard...".into());
        })?;

        // Operator toggles
        self.setup_toggle("debounceToggle", || {
            web_sys::console::log_1(&"Toggle debounce".into());
        })?;

        self.setup_toggle("throttleToggle", || {
            web_sys::console::log_1(&"Toggle throttle".into());
        })?;

        self.setup_toggle("delayToggle", || {
            web_sys::console::log_1(&"Toggle delay".into());
        })?;

        self.setup_toggle("sampleToggle", || {
            web_sys::console::log_1(&"Toggle sample".into());
        })?;

        self.setup_toggle("timeoutToggle", || {
            web_sys::console::log_1(&"Toggle timeout".into());
        })?;

        Ok(())
    }

    fn setup_button<F>(&self, id: &str, callback: F) -> Result<(), JsValue>
    where
        F: Fn() + 'static,
    {
        let button = self
            .document
            .get_element_by_id(id)
            .ok_or_else(|| JsValue::from_str(&format!("Button {} not found", id)))?
            .dyn_into::<HtmlButtonElement>()?;

        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            callback();
        }) as Box<dyn FnMut(_)>);

        button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();

        Ok(())
    }

    fn setup_toggle<F>(&self, id: &str, callback: F) -> Result<(), JsValue>
    where
        F: Fn() + 'static,
    {
        self.setup_button(id, callback)
    }

    #[allow(dead_code)]
    pub fn update_metrics(&self, received: u32, emitted: u32) -> Result<(), JsValue> {
        if let Some(element) = self.document.get_element_by_id("eventsReceived") {
            element.set_text_content(Some(&received.to_string()));
        }

        if let Some(element) = self.document.get_element_by_id("eventsEmitted") {
            element.set_text_content(Some(&emitted.to_string()));
        }

        if let Some(element) = self.document.get_element_by_id("dropRate") {
            let rate = if received > 0 {
                ((received - emitted) as f64 / received as f64 * 100.0) as u32
            } else {
                0
            };
            element.set_text_content(Some(&format!("{}%", rate)));
        }

        Ok(())
    }
}
