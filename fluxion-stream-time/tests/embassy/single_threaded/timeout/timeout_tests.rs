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

/// Test basic timeout functionality with Embassy timer
///
/// NOTE: This test runs within a Tokio runtime as a test harness
/// (Embassy executor requires nightly features for #[embassy_executor::task]).
/// However, it validates Embassy timer integration with the timeout operator.
#[test]
fn test_timeout_basic() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
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
    });
}
