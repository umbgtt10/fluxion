// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::CancellationToken;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlButtonElement, HtmlElement};

/// Shared dashboard UI with 12 hooking points (9 windows + 3 buttons)
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
    delay_window: HtmlElement,
    sample_window: HtmlElement,
    timeout_window: HtmlElement,

    // 3 control buttons
    start_button: HtmlButtonElement,
    stop_button: HtmlButtonElement,
    close_button: HtmlButtonElement,
}

impl DashboardUI {
    /// Creates a new DashboardUI wrapped in Rc<RefCell<>> for sharing across subscribe closures
    pub fn new(
        document: &Document,
        close_token: CancellationToken,
    ) -> Result<Rc<RefCell<Self>>, JsValue> {
        let ui = Self {
            // Get sensor windows
            sensor1_window: Self::get_element(document, "sensor1Window")?,
            sensor2_window: Self::get_element(document, "sensor2Window")?,
            sensor3_window: Self::get_element(document, "sensor3Window")?,

            // Get combined window
            combined_window: Self::get_element(document, "combinedWindow")?,

            // Get operator windows
            debounce_window: Self::get_element(document, "debounceWindow")?,
            delay_window: Self::get_element(document, "delayWindow")?,
            sample_window: Self::get_element(document, "sampleWindow")?,
            throttle_window: Self::get_element(document, "throttleWindow")?,
            timeout_window: Self::get_element(document, "timeoutWindow")?,

            // Get buttons
            start_button: Self::get_button(document, "startBtn")?,
            stop_button: Self::get_button(document, "stopBtn")?,
            close_button: Self::get_button(document, "closeBtn")?,
        };

        let ui_rc = Rc::new(RefCell::new(ui));
        ui_rc
            .borrow_mut()
            .wire_close_button_to_application_closure(close_token);

        Ok(ui_rc)
    }

    // ==================== Update methods for sensor windows (3) ====================

    pub fn wire_closure_to_start_button(&mut self, start_closure: Closure<dyn FnMut()>) {
        self.start_button
            .add_event_listener_with_callback("click", start_closure.as_ref().unchecked_ref())
            .unwrap();

        start_closure.forget();
    }

    pub fn wire_closure_to_stop_button(&mut self, stop_closure: Closure<dyn FnMut()>) {
        self.stop_button
            .add_event_listener_with_callback("click", stop_closure.as_ref().unchecked_ref())
            .unwrap();

        stop_closure.forget();
    }

    fn wire_close_button_to_application_closure(&mut self, close_token: CancellationToken) {
        let close_closure = Closure::wrap(Box::new(move || {
            web_sys::console::log_1(&"❌ Closing dashboard...".into());

            close_token.cancel();

            // Close the window
            if let Some(window) = web_sys::window() {
                let _ = window.close();
            }
        }) as Box<dyn FnMut()>);

        self.close_button
            .add_event_listener_with_callback("click", close_closure.as_ref().unchecked_ref())
            .unwrap();
        close_closure.forget();
    }

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

    pub fn update_delay(&mut self, value: u32) {
        self.delay_window
            .set_inner_html(&format!("<div class='value'>{}</div>", value));
    }

    pub fn update_throttle(&mut self, value: u32) {
        self.throttle_window
            .set_inner_html(&format!("<div class='value'>{}</div>", value));
    }

    pub fn update_sample(&mut self, value: u32) {
        self.sample_window
            .set_inner_html(&format!("<div class='value'>{}</div>", value));
    }

    pub fn update_timeout(&mut self, count: u32) {
        self.timeout_window
            .set_inner_html(&format!("<div class='value'>{}</div>", count));
    }

    pub fn update_timeout_error(&mut self, error: &str) {
        self.timeout_window
            .set_inner_html(&format!("<div class='value'>{}</div>", error));
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
}
