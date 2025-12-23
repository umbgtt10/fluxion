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
use std::panic;
use std::time::Duration;

#[test]
fn test_timeout_basic() {
    let result = panic::catch_unwind(|| {
        let executor = Box::leak(Box::new(embassy_executor::Executor::new()));
        executor.run(|spawner| {
            spawner.must_spawn(test_impl());
        });
    });

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
async fn test_impl() {
    // Arrange
    let timer = EmbassyTimerImpl;
    let (tx, stream) = test_channel::<EmbassyTimestamped<Person>>();
    let mut timed = stream.timeout(Duration::from_millis(100));

    // Act & Assert
    tx.unbounded_send(EmbassyTimestamped::new(person_alice(), timer.now()))
        .unwrap();

    let result = timed.next().await.unwrap();
    assert!(matches!(result, StreamItem::Value(_)));
    if let StreamItem::Value(v) = result {
        assert_eq!(v.value, person_alice());
    }

    Timer::after(embassy_time::Duration::from_millis(150)).await;
    assert!(matches!(timed.next().await.unwrap(), StreamItem::Error(_)));

    panic!("Test passed - using panic to exit executor");
}
