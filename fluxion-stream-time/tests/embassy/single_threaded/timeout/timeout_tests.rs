// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::embassy::helpers::{person_alice, test_channel, Person};
use fluxion_core::StreamItem;
use fluxion_runtime::impls::embassy::EmbassyTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream_time::{EmbassyTimestamped, TimeoutExt};
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
    let timer = EmbassyTimer;
    let (tx, stream) = test_channel::<EmbassyTimestamped<Person>>();
    let mut timed = stream.timeout(Duration::from_millis(100));

    // Act & Assert
    tx.try_send(EmbassyTimestamped::new(person_alice(), timer.now()))
        .unwrap();

    let result: StreamItem<EmbassyTimestamped<Person>> = timed.next().await.unwrap();
    assert!(matches!(result, StreamItem::Value(_)));
    if let StreamItem::Value(v) = result {
        assert_eq!(v.value, person_alice());
    }

    embassy_time::Timer::after(embassy_time::Duration::from_millis(150)).await;
    let timeout_result: StreamItem<EmbassyTimestamped<Person>> = timed.next().await.unwrap();
    assert!(matches!(timeout_result, StreamItem::Error(_)));

    panic!("Test passed - using panic to exit executor");
}
