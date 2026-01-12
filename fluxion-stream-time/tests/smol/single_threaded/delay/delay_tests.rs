// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::smol::helpers::{person_alice, test_channel};
use fluxion_runtime::impls::smol::SmolTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream_time::{DelayExt, SmolTimestamped};
use futures::StreamExt;
use std::time::Duration;

#[test]
fn test_delay_smol_single_threaded() {
    smol::block_on(async {
        // Arrange
        let (tx, rx) = test_channel();
        let mut delayed = rx.delay(Duration::from_millis(50));
        let timer = SmolTimer;

        // Act
        tx.try_send(SmolTimestamped::new(person_alice(), timer.now()))
            .unwrap();
        drop(tx);

        let start = std::time::Instant::now();
        let result = delayed.next().await;
        let elapsed = start.elapsed();

        // Assert
        assert!(result.is_some());
        assert!(elapsed >= Duration::from_millis(40)); // Allow 10ms tolerance
    });
}
