// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::embassy::helpers::{person_alice, test_channel, Person};
use embassy_time::Timer;
use fluxion_core::StreamItem;
use fluxion_stream_time::runtimes::EmbassyTimerImpl;
use fluxion_stream_time::timer::Timer as TimerTrait;
use fluxion_stream_time::{prelude::*, EmbassyTimestamped};
use futures::StreamExt;
use std::time::Duration;

// Mark the async test function as an Embassy task
#[embassy_executor::task]
async fn test_impl() {
    // Arrange
    let timer = EmbassyTimerImpl;
    let (tx, stream) = test_channel::<EmbassyTimestamped<Person>>();
    let mut timed = stream.timeout(Duration::from_millis(100));

    // Act - send a value quickly (should not timeout)
    tx.unbounded_send(EmbassyTimestamped::new(person_alice(), timer.now()))
        .unwrap();

    // Should receive the value
    let result = timed.next().await.unwrap();
    assert!(matches!(result, StreamItem::Value(_)));
    if let StreamItem::Value(v) = result {
        assert_eq!(v.value, person_alice());
    }

    // Wait longer than timeout period without sending
    Timer::after(embassy_time::Duration::from_millis(150)).await;

    // Should get a timeout error
    let result = timed.next().await.unwrap();
    assert!(matches!(result, StreamItem::Error(_)));

    // Exit the executor after test completes
    panic!("Test passed - using panic to exit executor");
}

#[test]
fn test_timeout_basic() {
    use std::panic;

    // Catch the panic we use to exit
    let result = panic::catch_unwind(|| {
        let executor = Box::leak(Box::new(embassy_executor::Executor::new()));
        executor.run(|spawner| {
            spawner.must_spawn(test_impl());
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
