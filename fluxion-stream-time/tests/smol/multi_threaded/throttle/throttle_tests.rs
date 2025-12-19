// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::smol::helpers::{person_alice, test_channel, timestamped_person};
use fluxion_stream_time::{SmolTimer, ThrottleExt};
use futures::StreamExt;
use std::time::Duration;

#[test]
fn test_throttle_smol_multi_threaded() {
    let executor = smol::Executor::new();
    futures::executor::block_on(executor.run(async {
        let (tx, rx) = test_channel();
        let timer = SmolTimer;

        let mut throttled = rx.throttle(Duration::from_millis(50), timer);

        executor
            .spawn(async move {
                tx.unbounded_send(timestamped_person(person_alice()))
                    .unwrap();
                // Wait for throttled result
                smol::Timer::after(Duration::from_millis(60)).await;
                tx.unbounded_send(timestamped_person(person_alice()))
                    .unwrap();
            })
            .detach();

        let result1 = throttled.next().await;
        assert!(result1.is_some());
    }));
}
