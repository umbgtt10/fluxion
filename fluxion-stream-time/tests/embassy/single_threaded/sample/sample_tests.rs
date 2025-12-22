// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::embassy::helpers::{person_alice, person_bob, test_channel, unwrap_stream, Person};
use embassy_time::Timer;
use fluxion_stream_time::runtimes::EmbassyTimerImpl;
use fluxion_stream_time::timer::Timer as TimerTrait;
use fluxion_stream_time::{prelude::*, EmbassyTimestamped};
use std::time::Duration;

/// Test basic sample functionality with Embassy timer.
/// Runs on Embassy executor with nightly Rust.
#[test]
fn test_sample_basic() {
    use std::panic;
    let result = panic::catch_unwind(|| {
        let executor = Box::leak(Box::new(embassy_executor::Executor::new()));
        executor.run(|spawner| {
            spawner.must_spawn(test_impl());
        });
    });

    // Verify the panic occurred with expected message
    assert!(result.is_err(), "Expected test to panic to exit executor");
    if let Err(err) = result {
        if let Some(msg) = err.downcast_ref::<&str>() {
            assert!(
                msg.contains("Test passed"),
                "Expected 'Test passed' panic message, got: {}",
                msg
            );
        } else if let Some(msg) = err.downcast_ref::<String>() {
            assert!(
                msg.contains("Test passed"),
                "Expected 'Test passed' panic message, got: {}",
                msg
            );
        } else {
            panic!("Unexpected panic payload");
        }
    }
}

#[embassy_executor::task]
async fn test_impl() {
    // Arrange
    let timer = EmbassyTimerImpl;
    let (tx, stream) = test_channel::<EmbassyTimestamped<Person>>();
    let mut sampled = stream.sample(Duration::from_millis(100));

    // Act - send first value
    tx.unbounded_send(EmbassyTimestamped::new(person_alice(), timer.now()))
        .unwrap();

    // Wait a bit, then send second value
    Timer::after(embassy_time::Duration::from_millis(50)).await;
    tx.unbounded_send(EmbassyTimestamped::new(person_bob(), timer.now()))
        .unwrap();

    // Wait for sample period
    Timer::after(embassy_time::Duration::from_millis(100)).await;

    // Assert - should get the latest value (Bob) after sample period
    let result = unwrap_stream(&mut sampled, 200).await;
    assert_eq!(result.unwrap().value, person_bob());

    // Exit executor by panicking (Embassy executor.run() never returns)
    panic!("Test passed - using panic to exit executor");
}
