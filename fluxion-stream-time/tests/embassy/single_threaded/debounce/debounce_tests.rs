// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::embassy::helpers::{person_alice, test_channel, unwrap_stream, Person};
use fluxion_runtime::impls::embassy::EmbassyTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream_time::{DebounceExt, EmbassyTimestamped};
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
    let timer = EmbassyTimer;
    let (tx, stream) = test_channel::<EmbassyTimestamped<Person>>();
    let mut debounced = stream.debounce(Duration::from_millis(100));

    // Act
    tx.try_send(EmbassyTimestamped::new(person_alice(), timer.now()))
        .unwrap();

    embassy_time::Timer::after(embassy_time::Duration::from_millis(150)).await;

    // Assert
    let result = unwrap_stream(&mut debounced, 200).await;
    assert_eq!(result.unwrap().value, person_alice());

    panic!("Test passed - using panic to exit executor");
}
