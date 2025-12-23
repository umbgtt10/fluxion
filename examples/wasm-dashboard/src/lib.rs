// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlElement, Window};

mod app;
mod chart;
mod sensors;
mod ui;

use app::Dashboard;

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

    // Initialize the dashboard
    let mut dashboard = Dashboard::new(window, document)?;

    // Start the main loop
    dashboard.run().await?;

    Ok(())
}

/// Get window and document helpers
fn get_window() -> Result<Window, JsValue> {
    web_sys::window().ok_or_else(|| JsValue::from_str("No window object"))
}

fn get_document() -> Result<Document, JsValue> {
    get_window()?
        .document()
        .ok_or_else(|| JsValue::from_str("No document object"))
}

fn get_element_by_id(id: &str) -> Result<HtmlElement, JsValue> {
    Ok(get_document()?
        .get_element_by_id(id)
        .ok_or_else(|| JsValue::from_str(&format!("Element {} not found", id)))?
        .dyn_into::<HtmlElement>()?)
}
