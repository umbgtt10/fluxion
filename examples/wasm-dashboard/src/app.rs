// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use wasm_bindgen::prelude::*;
use web_sys::{Document, Window};

use crate::sensors::SensorSimulator;
use crate::ui::UI;

/// Main dashboard application
pub struct Dashboard {
    _window: Window,
    _document: Document,
    ui: UI,
    simulator: SensorSimulator,
}

impl Dashboard {
    pub fn new(window: Window, document: Document) -> Result<Self, JsValue> {
        let ui = UI::new(&window, &document)?;
        let simulator = SensorSimulator::new();

        web_sys::console::log_1(&"✅ Dashboard initialized".into());

        Ok(Self {
            _window: window,
            _document: document,
            ui,
            simulator,
        })
    }

    pub async fn run(&mut self) -> Result<(), JsValue> {
        web_sys::console::log_1(&"▶️ Dashboard starting...".into());

        // Set up event listeners
        self.ui.setup_controls(&mut self.simulator)?;

        web_sys::console::log_1(&"✅ Dashboard running".into());

        Ok(())
    }
}
