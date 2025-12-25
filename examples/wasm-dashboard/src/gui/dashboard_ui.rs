// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlButtonElement, HtmlElement};

/// Shared dashboard UI with 11 hooking points (9 windows + 2 buttons)
pub struct DashboardUI {
    // 3 sensor windows (top section)
    sensor1_window: HtmlElement,
    sensor2_window: HtmlElement,
    sensor3_window: HtmlElement,

    // 1 combined stream window (middle section)
    combined_window: HtmlElement,

    // 5 time-bound operator windows (bottom section)
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
    /// Creates a new DashboardUI wrapped in Rc<RefCell<>> for sharing across subscribe closures
    pub fn new(document: &Document) -> Result<Rc<RefCell<Self>>, JsValue> {
        let ui = Self {
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
        Ok(document
            .get_element_by_id(id)
            .ok_or_else(|| JsValue::from_str(&format!("Element {} not found", id)))?
            .dyn_into::<HtmlElement>()?)
    }

    fn get_button(document: &Document, id: &str) -> Result<HtmlButtonElement, JsValue> {
        Ok(document
            .get_element_by_id(id)
            .ok_or_else(|| JsValue::from_str(&format!("Button {} not found", id)))?
            .dyn_into::<HtmlButtonElement>()?)
    }

    // ==================== Update methods for sensor windows (3) ====================

    pub fn update_sensor1(&mut self, value: u32) {
        self.sensor1_window
            .set_inner_html(&format!("<div class='value'>{}</div>", value));
    }

    pub fn update_sensor2(&mut self, value: u32) {
        self.sensor2_window
            .set_inner_html(&format!("<div class='value'>{}</div>", value));
    }

    pub fn update_sensor3(&mut self, value: u32) {
        self.sensor3_window
            .set_inner_html(&format!("<div class='value'>{}</div>", value));
    }

    // ==================== Update method for combined window (1) ====================

    pub fn update_combined(&mut self, value: u32) {
        self.combined_window
            .set_inner_html(&format!("<div class='value combined'>{}</div>", value));
    }

    // ==================== Update methods for time-bound operator windows (5) ====================

    pub fn update_debounce(&mut self, value: u32) {
        self.debounce_window
            .set_inner_html(&format!("<div class='value'>{}</div>", value));
    }

    pub fn update_throttle(&mut self, value: u32) {
        self.throttle_window
            .set_inner_html(&format!("<div class='value'>{}</div>", value));
    }

    pub fn update_buffer(&mut self, values: &[u32]) {
        let items_html = values
            .iter()
            .map(|v| format!("<span class='buffer-item'>{}</span>", v))
            .collect::<Vec<_>>()
            .join(" ");

        self.buffer_window.set_inner_html(&format!(
            "<div class='buffer-count'>{} items</div><div class='buffer-items'>{}</div>",
            values.len(),
            items_html
        ));
    }

    pub fn update_sample(&mut self, value: u32) {
        self.sample_window
            .set_inner_html(&format!("<div class='value'>{}</div>", value));
    }

    pub fn update_window(&mut self, count: usize) {
        self.window_window.set_inner_html(&format!(
            "<div class='window-count'>{} items in window</div>",
            count
        ));
    }

    // ==================== Button control methods ====================

    pub fn enable_start(&mut self) {
        self.start_button.set_disabled(false);
        self.stop_button.set_disabled(true);
    }

    pub fn enable_stop(&mut self) {
        self.start_button.set_disabled(true);
        self.stop_button.set_disabled(false);
    }

    // ==================== Button accessors for event wiring ====================

    pub fn start_button(&self) -> &HtmlButtonElement {
        &self.start_button
    }

    pub fn stop_button(&self) -> &HtmlButtonElement {
        &self.stop_button
    }
}
