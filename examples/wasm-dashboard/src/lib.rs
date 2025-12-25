// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::CancellationToken;
use wasm_bindgen::prelude::*;

mod gui;
mod presentation;
mod source;

use gui::DashboardUI;
use presentation::DashboardUpdater;
use source::Sensors;

use crate::source::SensorStreams;

/// Entry point called from JavaScript
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Set panic hook for better error messages
    console_error_panic_hook::set_once();

    // Log initialization
    web_sys::console::log_1(&"🚀 Fluxion WASM Dashboard initializing...".into());

    Ok(())
}

/// Initialize and start the dashboard
#[wasm_bindgen]
pub async fn start_dashboard() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    web_sys::console::log_1(&"✅ Starting....".into());

    // Create persistent cancellation token for close button
    let close_cancel_token = CancellationToken::new();

    web_sys::console::log_1(&"✅ Starting....".into());

    // Create GUI with 12 hooking points (9 windows + 3 buttons)
    let ui = DashboardUI::new(&document, close_cancel_token.clone())?;

    web_sys::console::log_1(
        &"✅ Dashboard UI created with 12 hooking points (close button wired)".into(),
    );

    // Enable start button by default
    ui.borrow_mut().enable_start();

    // Spawn dashboard as background task - move sensors/streams into closure to keep them alive
    wasm_bindgen_futures::spawn_local(async move {
        let sensors = Sensors::new(close_cancel_token.clone());
        let streams = SensorStreams::new(sensors);
        let updater = DashboardUpdater::new(&streams, ui.clone(), close_cancel_token.clone());

        updater.run().await;

        web_sys::console::log_1(&"✅ Dashboard shutdown complete".into());
    });

    web_sys::console::log_1(&"✅ Dashboard running".into());

    Ok(())
}
