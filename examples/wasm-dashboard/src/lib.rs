// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use wasm_bindgen::prelude::*;

mod gui;

use gui::DashboardUI;

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

    // Create GUI with 11 hooking points (9 windows + 2 buttons)
    let ui = DashboardUI::new(&document)?;

    web_sys::console::log_1(&"✅ Dashboard UI created with 11 hooking points".into());

    // Enable start button by default
    ui.borrow_mut().enable_start();

    Ok(())
}
