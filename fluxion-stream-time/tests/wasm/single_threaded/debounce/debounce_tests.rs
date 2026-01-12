// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::wasm::helpers::{person_alice, test_channel, unwrap_stream, Person};
use fluxion_runtime::impls::wasm::WasmTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream_time::{DebounceExt, WasmTimestamped};
use std::time::Duration;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
async fn test_debounce_basic() {
    // Arrange
    let timer = WasmTimer;
    let (tx, stream) = test_channel::<WasmTimestamped<Person>>();
    let mut debounced = stream.debounce(Duration::from_millis(100));

    // Act
    tx.try_send(WasmTimestamped::new(person_alice(), timer.now()))
        .unwrap();

    gloo_timers::future::sleep(Duration::from_millis(150)).await;

    // Assert
    assert_eq!(
        unwrap_stream(&mut debounced, 200).await.unwrap().value,
        person_alice()
    );
}
