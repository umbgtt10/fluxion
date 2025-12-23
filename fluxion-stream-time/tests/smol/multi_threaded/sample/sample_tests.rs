// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::smol::helpers::{person_alice, test_channel};
use fluxion_stream_time::prelude::*;
use fluxion_stream_time::timer::Timer;
use fluxion_stream_time::{SmolTimer, SmolTimestamped};
use futures::StreamExt;
use std::time::Duration;

#[test]
fn test_sample_smol_multi_threaded() {
    // Arrange
    let executor = smol::Executor::new();
    futures::executor::block_on(executor.run(async {
        let (tx, rx) = test_channel();
        let mut sampled = rx.sample(Duration::from_millis(50));
        let timer = SmolTimer;

        executor
            .spawn(async move {
                // Act
                for _ in 0..5 {
                    tx.unbounded_send(SmolTimestamped::new(person_alice(), timer.now()))
                        .unwrap();
                    smol::Timer::after(Duration::from_millis(15)).await;
                }
            })
            .detach();

        // Assert
        assert!(sampled.next().await.is_some());
    }));
}
