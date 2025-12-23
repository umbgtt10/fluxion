// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use wasm_bindgen::prelude::*;
use web_sys::{Document, Window};

use crate::sensors::SensorSimulator;
use crate::ui::UI;

/// Main dashboard application
pub struct Dashboard {
    window: Window,
    document: Document,
    ui: UI,
    simulator: SensorSimulator,
    running: bool,
}

impl Dashboard {
    pub fn new(window: Window, document: Document) -> Result<Self, JsValue> {
        let ui = UI::new(&window, &document)?;
        let simulator = SensorSimulator::new();

        web_sys::console::log_1(&"✅ Dashboard initialized".into());

        Ok(Self {
            window,
            document,
            ui,
            simulator,
            running: false,
        })
    }

    pub async fn run(&mut self) -> Result<(), JsValue> {
        web_sys::console::log_1(&"▶️ Dashboard starting...".into());

        // Set up event listeners
        self.ui.setup_controls(&mut self.simulator)?;

        // Main update loop will be driven by animation frames
        self.running = true;

        web_sys::console::log_1(&"✅ Dashboard running".into());

        Ok(())
    }

    pub fn stop(&mut self) {
        self.running = false;
        web_sys::console::log_1(&"⏹️ Dashboard stopped".into());
    }
}
