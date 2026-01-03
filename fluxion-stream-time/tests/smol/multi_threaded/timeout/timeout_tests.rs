// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::smol::helpers::{person_alice, test_channel};
use fluxion_stream_time::prelude::*;
use fluxion_stream_time::timer::Timer;
use fluxion_stream_time::{SmolTimer, SmolTimestamped};
use futures::StreamExt;
use std::time::Duration;

#[test]
fn test_timeout_smol_multi_threaded() {
    // Arrange
    let executor = smol::Executor::new();
    futures::executor::block_on(executor.run(async {
        let (tx, rx) = test_channel();
        let mut timed = rx.timeout(Duration::from_millis(100));
        let timer = SmolTimer;

        executor
            .spawn(async move {
                // Act
                tx.unbounded_send(SmolTimestamped::new(person_alice(), timer.now()))
                    .unwrap();
            })
            .detach();

        // Assert
        assert!(timed.next().await.is_some());
    }));
}
