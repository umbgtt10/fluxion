// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::smol::helpers::{person_alice, test_channel, timestamped_person};
use fluxion_stream_time::{DelayExt, SmolTimer};
use futures::StreamExt;
use std::time::Duration;

#[test]
fn test_delay_smol_single_threaded() {
    smol::block_on(async {
        let (tx, rx) = test_channel();
        let timer = SmolTimer;

        let mut delayed = rx.delay(Duration::from_millis(50), timer);

        tx.unbounded_send(timestamped_person(person_alice()))
            .unwrap();
        drop(tx);

        let start = std::time::Instant::now();
        let result = delayed.next().await;
        let elapsed = start.elapsed();

        assert!(result.is_some());
        assert!(elapsed >= Duration::from_millis(40)); // Allow 10ms tolerance
    });
}
