// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::embassy::helpers::{person_alice, test_channel, unwrap_stream, Person};
use embassy_time::Timer;
use fluxion_stream_time::runtimes::EmbassyTimerImpl;
use fluxion_stream_time::timer::Timer as TimerTrait;
use fluxion_stream_time::{prelude::*, EmbassyTimestamped};
use std::panic;
use std::time::Duration;

#[test]
fn test_debounce_basic() {
    let result = panic::catch_unwind(|| {
        let executor = Box::leak(Box::new(embassy_executor::Executor::new()));
        executor.run(|spawner| {
            spawner.must_spawn(run_test());
        });
    });

    assert!(result.is_err(), "Expected test to panic to exit executor");
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

#[embassy_executor::task]
async fn run_test() {
    // Arrange
    let timer = EmbassyTimerImpl;
    let (tx, stream) = test_channel::<EmbassyTimestamped<Person>>();
    let mut debounced = stream.debounce(Duration::from_millis(100));

    // Act
    tx.unbounded_send(EmbassyTimestamped::new(person_alice(), timer.now()))
        .unwrap();

    Timer::after(embassy_time::Duration::from_millis(150)).await;

    // Assert
    let result = unwrap_stream(&mut debounced, 200).await;
    assert_eq!(result.unwrap().value, person_alice());

    panic!("Test passed - using panic to exit executor");
}
