// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::CancellationToken;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

mod gui;
mod source;

use gui::DashboardUI;
use source::Sensors;

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

    // Create persistent cancellation token for close button
    let cancel_token = CancellationToken::new();

    let _sensors = Sensors::new(cancel_token.clone());

    // Create GUI with 12 hooking points (9 windows + 3 buttons)
    let ui = DashboardUI::new(&document, cancel_token)?;

    web_sys::console::log_1(
        &"✅ Dashboard UI created with 12 hooking points (close button wired)".into(),
    );

    // Enable start button by default
    ui.borrow_mut().enable_start();

    Ok(())
}
