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
async fn test_throttle_basic() {
    // Arrange
    let timer = WasmTimer::new();
    let (tx, stream) = test_channel::<WasmTimestamped<Person>>();
    let mut throttled = stream.throttle(Duration::from_millis(100));

    // Act
    tx.unbounded_send(WasmTimestamped::new(person_alice(), timer.now()))
        .unwrap();

    gloo_timers::future::sleep(Duration::from_millis(50)).await;

    // Assert
    assert_eq!(
        unwrap_stream(&mut throttled, 200).await.unwrap().value,
        person_alice()
    );
}
