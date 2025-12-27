// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod config;
mod gui;
mod presentation;
mod processing;
mod source;

use crate::config::DashboardConfig;
use crate::source::Sensors;
use crate::{
    processing::{CombinedStream, ResultStreams},
    source::SensorStreams,
};
use fluxion_core::CancellationToken;
use gui::DashboardUI;
use presentation::DashboardUpdater;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

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

    web_sys::console::log_1(
        &"✅ Dashboard UI created with 12 hooking points (close button wired)".into(),
    );

    web_sys::console::log_1(&"✅ Dashboard running".into());

    close_token.cancelled().await;

    web_sys::console::log_1(&"✅ Cancelled! No longer waiting for close cancellation".into());

    Ok(())
}

async fn start(ui: Rc<RefCell<DashboardUI>>, stop_token: CancellationToken) {
    let ui_clone = ui.clone();
    ui_clone.borrow_mut().enable_stop();

    web_sys::console::log_1(&"✅ Started".into());
    // Load configuration
    let config = match DashboardConfig::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            web_sys::console::error_1(&format!("Failed to load config: {}", e).into());
            return;
        }
    };

    let sensors = Sensors::new(config, stop_token.clone());
    let streams = SensorStreams::new(sensors);
    let combined_stream = CombinedStream::new(&streams);
    let result_streams = ResultStreams::new(&combined_stream);

    let updater = DashboardUpdater::new(
        &streams,
        &combined_stream,
        result_streams.subscribe_debounce(),
        result_streams.subscribe_delay(),
        result_streams.subscribe_sample(),
        result_streams.subscribe_throttle(),
        result_streams.subscribe_timeout(),
        ui_clone,
        stop_token,
    );

    web_sys::console::log_1(&"✅ Running".into());

    updater.run().await;

    web_sys::console::log_1(&"✅ Dashboard shutdown complete".into());
}
