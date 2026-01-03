// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::smol::helpers::{person_alice, test_channel};
use fluxion_stream_time::prelude::*;
use fluxion_stream_time::timer::Timer;
use fluxion_stream_time::{SmolTimer, SmolTimestamped};
use futures::StreamExt;
use std::time::Duration;

#[test]
fn test_timeout_smol_single_threaded() {
    smol::block_on(async {
        // Arrange
        let (tx, rx) = test_channel();
        let mut timed = rx.timeout(Duration::from_millis(100));
        let timer = SmolTimer;

        // Act & Assert
        tx.unbounded_send(SmolTimestamped::new(person_alice(), timer.now()))
            .unwrap();
        assert!(timed.next().await.is_some());

        // Don't send anything, should timeout
        drop(tx);
    });
}
