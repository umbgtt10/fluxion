// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::smol::helpers::{person_alice, test_channel, timestamped_person};
use fluxion_stream_time::prelude::*;
use fluxion_stream_time::SmolTimer;
use futures::StreamExt;
use std::time::Duration;

#[test]
fn test_sample_smol_single_threaded() {
    smol::block_on(async {
        let (tx, rx) = test_channel();
        let timer = SmolTimer;

        let mut sampled = rx.sample(Duration::from_millis(100));

        // Send one item
        tx.unbounded_send(timestamped_person(person_alice()))
            .unwrap();

        // Wait for sample period to complete
        smol::Timer::after(Duration::from_millis(150)).await;

        // Should receive the sampled item
        let result = sampled.next().await;
        assert!(result.is_some());
    });
}
