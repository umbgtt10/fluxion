// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::embassy::helpers::{person_alice, test_channel, unwrap_stream, Person};
use embassy_time::Timer;
use fluxion_stream_time::runtimes::EmbassyTimerImpl;
use fluxion_stream_time::timer::Timer as TimerTrait;
use fluxion_stream_time::{prelude::*, EmbassyTimestamped};
use std::time::Duration;

#[test]
fn test_debounce_basic() {
    use std::panic;

    // Catch the panic we use to exit
    let result = panic::catch_unwind(|| {
        let executor = Box::leak(Box::new(embassy_executor::Executor::new()));
        executor.run(|spawner| {
            spawner.must_spawn(run_test());
        });
    });

    // Verify it was our expected panic
    if let Err(e) = result {
        if let Some(msg) = e.downcast_ref::<&str>() {
            assert!(msg.contains("Test passed"), "Unexpected panic: {}", msg);
        } else if let Some(msg) = e.downcast_ref::<String>() {
            assert!(msg.contains("Test passed"), "Unexpected panic: {}", msg);
        } else {
            panic!("Test panicked unexpectedly");
        }
    }
}

/// Test basic debounce functionality with Embassy timer.
/// Uses Embassy executor (requires nightly).
#[embassy_executor::task]
async fn run_test() {
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

    // Exit the executor after test completes
    panic!("Test passed - using panic to exit executor");
}
