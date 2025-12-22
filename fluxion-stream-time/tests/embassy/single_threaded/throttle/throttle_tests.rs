// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::embassy::helpers::{person_alice, person_bob, test_channel, unwrap_stream, Person};
use embassy_time::Timer;
use fluxion_stream_time::runtimes::EmbassyTimerImpl;
use fluxion_stream_time::timer::Timer as TimerTrait;
use fluxion_stream_time::{prelude::*, EmbassyTimestamped};
use std::time::Duration;

/// Test basic throttle functionality with Embassy timer
///
/// NOTE: This test runs within a Tokio runtime as a test harness
/// (Embassy executor requires nightly features for #[embassy_executor::task]).
/// However, it validates Embassy timer integration with the throttle operator.
#[test]
fn test_throttle_basic() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        // Arrange
        let timer = EmbassyTimerImpl;
        let (tx, stream) = test_channel::<EmbassyTimestamped<Person>>();
        let mut throttled = stream.throttle(Duration::from_millis(100));

        // Act - send first value (should emit immediately)
        tx.unbounded_send(EmbassyTimestamped::new(person_alice(), timer.now()))
            .unwrap();

        // First value should emit immediately
        let result = unwrap_stream(&mut throttled, 50).await;
        assert_eq!(result.unwrap().value, person_alice());

        // Send second value immediately (should be dropped by throttle)
        tx.unbounded_send(EmbassyTimestamped::new(person_bob(), timer.now()))
            .unwrap();

        // Wait for throttle period to expire
        Timer::after(embassy_time::Duration::from_millis(150)).await;

        // Send third value (throttle period expired, should emit)
        tx.unbounded_send(EmbassyTimestamped::new(person_bob(), timer.now()))
            .unwrap();

        // Third value should emit
        let result = unwrap_stream(&mut throttled, 200).await;
        assert_eq!(result.unwrap().value, person_bob());
    });
}
