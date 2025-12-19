// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::smol::helpers::{person_alice, test_channel, timestamped_person};
use fluxion_stream_time::{SmolTimer, ThrottleExt};
use futures::StreamExt;
use std::time::Duration;

#[test]
fn test_throttle_smol_single_threaded() {
    smol::block_on(async {
        let (tx, rx) = test_channel();
        let timer = SmolTimer;

        let mut throttled = rx.throttle(Duration::from_millis(50), timer);

        // Send first item
        tx.unbounded_send(timestamped_person(person_alice()))
            .unwrap();
        let result1 = throttled.next().await;
        assert!(result1.is_some());

        // Send second item immediately (should be throttled)
        tx.unbounded_send(timestamped_person(person_alice()))
            .unwrap();
        drop(tx);
    });
}
