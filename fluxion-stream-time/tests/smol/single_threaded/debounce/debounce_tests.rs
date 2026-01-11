// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::smol::helpers::{person_alice, test_channel};
use fluxion_runtime::impls::smol::SmolTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream_time::{DebounceExt, SmolTimestamped};
use futures::StreamExt;
use std::time::Duration;

#[test]
fn test_debounce_smol_single_threaded() {
    smol::block_on(async {
        // Arrange
        let (tx, rx) = test_channel();
        let mut debounced = rx.debounce(Duration::from_millis(100));
        let timer = SmolTimer;

        // Act
        tx.unbounded_send(SmolTimestamped::new(person_alice(), timer.now()))
            .unwrap();
        smol::Timer::after(Duration::from_millis(10)).await;
        tx.unbounded_send(SmolTimestamped::new(person_alice(), timer.now()))
            .unwrap();
        drop(tx);

        // Assert
        assert!(debounced.next().await.is_some());
    });
}
