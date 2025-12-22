// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::embassy::helpers::{person_alice, test_channel, unwrap_stream, Person};
use embassy_time::Timer;
use fluxion_stream_time::runtimes::EmbassyTimerImpl;
use fluxion_stream_time::timer::Timer as TimerTrait;
use fluxion_stream_time::{prelude::*, EmbassyTimestamped};
use std::time::Duration;

/// Test basic delay functionality with Embassy timer
///
/// NOTE: This test runs within a Tokio runtime as a test harness
/// (Embassy executor requires nightly features for #[embassy_executor::task]).
/// However, it validates Embassy timer integration with the delay operator.
#[test]
fn test_delay_basic() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        // Arrange
        let timer = EmbassyTimerImpl;
        let (tx, stream) = test_channel::<EmbassyTimestamped<Person>>();
        let mut delayed = stream.delay(Duration::from_millis(100));

        // Act - send a value
        tx.unbounded_send(EmbassyTimestamped::new(person_alice(), timer.now()))
            .unwrap();

        // Wait for delay period using Embassy's Timer
        Timer::after(embassy_time::Duration::from_millis(150)).await;

        // Assert - value should be delayed and emitted
        let result = unwrap_stream(&mut delayed, 200).await;
        assert_eq!(result.unwrap().value, person_alice());
    });
}
