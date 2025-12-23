// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlButtonElement, Window};

use crate::sensors::SensorSimulator;

/// UI manager for the dashboard
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
