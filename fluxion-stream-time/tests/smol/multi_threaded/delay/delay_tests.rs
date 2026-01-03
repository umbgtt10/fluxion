// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::smol::helpers::{person_alice, test_channel};
use fluxion_stream_time::prelude::*;
use fluxion_stream_time::timer::Timer;
use fluxion_stream_time::{SmolTimer, SmolTimestamped};
use futures::StreamExt;
use std::time::Duration;

#[test]
fn test_delay_smol_multi_threaded() {
    // Arrange
    let executor = smol::Executor::new();
    futures::executor::block_on(executor.run(async {
        let (tx, rx) = test_channel();
        let mut delayed = rx.delay(Duration::from_millis(50));
        let timer = SmolTimer;

        executor
            .spawn(async move {
                // Act
                tx.unbounded_send(SmolTimestamped::new(person_alice(), timer.now()))
                    .unwrap();
            })
            .detach();

        let start = std::time::Instant::now();
        let result = delayed.next().await;
        let elapsed = start.elapsed();

        // Assert
        assert!(result.is_some());
        assert!(elapsed >= Duration::from_millis(40)); // Allow 10ms tolerance
    }));
}
