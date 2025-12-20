// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::smol::helpers::{person_alice, test_channel, timestamped_person};
use fluxion_stream_time::prelude::*;
use futures::StreamExt;
use std::time::Duration;

#[test]
fn test_timeout_smol_single_threaded() {
    smol::block_on(async {
        let (tx, rx) = test_channel();
        let mut timed = rx.timeout(Duration::from_millis(100));

        // Send item immediately
        tx.unbounded_send(timestamped_person(person_alice()))
            .unwrap();
        let result = timed.next().await;
        assert!(result.is_some());

        // Don't send anything, should timeout
        drop(tx);
    });
}
