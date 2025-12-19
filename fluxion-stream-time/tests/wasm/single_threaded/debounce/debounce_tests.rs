// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::wasm::helpers::{person_alice, test_channel, unwrap_stream, Person};
use fluxion_stream_time::runtimes::wasm_implementation::WasmTimer;
use fluxion_stream_time::timer::Timer;
use fluxion_stream_time::{prelude::*, InstantTimestamped};
use std::time::Duration;
use wasm_bindgen_test::*;

type WasmTimestamped<T> = InstantTimestamped<T, WasmTimer>;

#[wasm_bindgen_test]
async fn test_debounce_basic() {
    // Arrange
    let timer = WasmTimer::new();
    let (tx, stream) = test_channel::<WasmTimestamped<Person>>();
    let mut debounced = stream.debounce(Duration::from_millis(100), timer.clone());

    // Act
    tx.unbounded_send(WasmTimestamped::new(person_alice(), timer.now()))
        .unwrap();

    // Note: WASM tests cannot use pause/advance like Tokio
    // Tests must use real delays with gloo_timers::sleep
    gloo_timers::future::sleep(Duration::from_millis(150)).await;

    // Assert
    let result = unwrap_stream(&mut debounced, 200).await;
    assert_eq!(result.unwrap().value, person_alice());
}
