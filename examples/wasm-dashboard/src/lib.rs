// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

mod gui;
mod processing;
mod source;

use crate::source::{Sensors, SourceLayer};
use crate::{
    processing::{DashboardOrchestrator, ProcessingLayer},
    source::SensorStreams,
};
use fluxion_core::CancellationToken;
use gui::DashboardUI;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{console, window};

/// Entry point called from JavaScript
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Set panic hook for better error messages
    console_error_panic_hook::set_once();

    // Log initialization
    console::log_1(&"🚀 Fluxion WASM Dashboard initializing...".into());

    Ok(())
}

/// Initialize and start the dashboard
#[wasm_bindgen]
pub async fn start_dashboard() -> Result<(), JsValue> {
    let window = window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    console::log_1(&"✅ Starting....".into());

    let close_token = CancellationToken::new();
    let ui = DashboardUI::new(&document, close_token.clone())?;

    let stop_token = Rc::new(RefCell::new(Option::<CancellationToken>::None));

    let ui_for_start = ui.clone();
    let stop_token_for_stop = stop_token.clone();
    ui.borrow_mut()
        .wire_closure_to_start_button(Closure::wrap(Box::new(move || {
            let new_stop_token = CancellationToken::new();
            *stop_token_for_stop.borrow_mut() = Some(new_stop_token.clone());
            wasm_bindgen_futures::spawn_local(start(ui_for_start.clone(), new_stop_token.clone()));
        })));

    let ui_for_stop = ui.clone();
    let stop_token_for_stop = stop_token.clone();
    ui.borrow_mut()
        .wire_closure_to_stop_button(Closure::wrap(Box::new(move || {
            if let Some(token) = stop_token_for_stop.borrow().as_ref() {
                token.cancel();
                ui_for_stop.borrow_mut().enable_start();
            }
        })));

    console::log_1(&"✅ Dashboard UI created with 12 hooking points (close button wired)".into());

    console::log_1(&"✅ Dashboard running".into());

    close_token.cancelled().await;

    console::log_1(&"✅ Cancelled! No longer waiting for close cancellation".into());

    Ok(())
}

async fn start(ui: Rc<RefCell<DashboardUI>>, stop_token: CancellationToken) {
    let ui_clone = ui.clone();
    ui_clone.borrow_mut().enable_stop();

    console::log_1(&"✅ Started".into());

    // Create source layer
    let sensors = Sensors::new(stop_token.clone());
    let sensor_streams = SensorStreams::new(sensors);
    let source_layer = SourceLayer::new(sensor_streams);

    // Create processing layer
    let processing_layer = ProcessingLayer::new(&source_layer);

    // Create orchestrator with both traits
    let orchestrator = DashboardOrchestrator::new(processing_layer, ui_clone, stop_token);

    console::log_1(&"✅ Running".into());

    orchestrator.run().await;

    console::log_1(&"✅ Dashboard shutdown complete".into());
}
