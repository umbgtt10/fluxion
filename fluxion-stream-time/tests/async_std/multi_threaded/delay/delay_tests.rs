// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::async_std::helpers::{person_alice, test_channel, Person};

use fluxion_stream_time::runtimes::AsyncStdTimer;
use fluxion_stream_time::timer::Timer;
use fluxion_stream_time::{prelude::*, InstantTimestamped};
use std::time::Duration;

type AsyncStdTimestamped<T> = InstantTimestamped<T, AsyncStdTimer>;

#[async_std::test]
async fn test_delay_across_threads() {
    // Arrange
    let timer = AsyncStdTimer;
    let (tx, stream) = test_channel::<AsyncStdTimestamped<Person>>();

    // Spawn on different thread
    let timer_clone = timer.clone();
    let handle = async_std::task::spawn(async move {
        use futures::StreamExt;
        let mut delayed = stream.delay(Duration::from_millis(50));
        delayed.next().await
    });

    // Act
    tx.unbounded_send(AsyncStdTimestamped::new(person_alice(), timer.now()))
        .unwrap();
    drop(tx);

    // Assert
    let result = handle.await;
    assert_eq!(result.unwrap().unwrap().value, person_alice());
}
