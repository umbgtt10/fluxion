// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::smol::helpers::{person_alice, test_channel, timestamped_person};
use fluxion_stream_time::{SmolTimer, TimeoutExt};
use futures::StreamExt;
use std::time::Duration;

#[test]
fn test_timeout_smol_multi_threaded() {
    let executor = smol::Executor::new();
    futures::executor::block_on(executor.run(async {
        let (tx, rx) = test_channel();
        let timer = SmolTimer;

        let mut timed = rx.timeout(Duration::from_millis(100), timer);

        executor
            .spawn(async move {
                tx.unbounded_send(timestamped_person(person_alice()))
                    .unwrap();
            })
            .detach();

        let result = timed.next().await;
        assert!(result.is_some());
    }));
}
