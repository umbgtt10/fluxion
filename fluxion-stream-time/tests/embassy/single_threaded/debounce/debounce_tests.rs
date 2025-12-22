// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::embassy::helpers::{person_alice, test_channel, unwrap_stream, Person};
use embassy_time::Timer;
use fluxion_stream_time::runtimes::EmbassyTimerImpl;
use fluxion_stream_time::timer::Timer as TimerTrait;
use fluxion_stream_time::{prelude::*, EmbassyTimestamped};
use std::time::Duration;

/// Test basic debounce functionality with Embassy timer
///
/// NOTE: This test currently runs within a Tokio runtime due to Embassy executor
/// requiring nightly features (`#[embassy_executor::task]` needs unstable features).
/// However, it still validates:
/// 1. EmbassyTimerImpl integration with time operators
/// 2. embassy_time::Timer usage for delays
/// 3. EmbassyTimestamped type correctness
///
/// Full Embassy executor testing would require:
/// - Nightly Rust
/// - Embassy executor with std features
/// - More complex test harness
#[test]
fn test_debounce_basic() {
    // Use Tokio runtime as harness for running the async test
    // (Embassy executor requires nightly for task macro)
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        // Arrange - use Embassy timer implementation
        let timer = EmbassyTimerImpl;
        let (tx, stream) = test_channel::<EmbassyTimestamped<Person>>();
        let mut debounced = stream.debounce(Duration::from_millis(100));

        // Act - send a value with Embassy timestamp
        tx.unbounded_send(EmbassyTimestamped::new(person_alice(), timer.now()))
            .unwrap();

        // Wait using Embassy's timer (std driver)
        Timer::after(embassy_time::Duration::from_millis(150)).await;

        // Assert - value should be debounced and emitted
        let result = unwrap_stream(&mut debounced, 200).await;
        assert_eq!(result.unwrap().value, person_alice());
    });
}
