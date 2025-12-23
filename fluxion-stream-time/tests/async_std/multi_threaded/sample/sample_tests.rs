// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::async_std::helpers::{person_alice, test_channel, unwrap_stream, Person};
use fluxion_stream_time::runtimes::AsyncStdTimer;
use fluxion_stream_time::timer::Timer;
use fluxion_stream_time::{prelude::*, InstantTimestamped};
use std::time::Duration;

type AsyncStdTimestamped<T> = InstantTimestamped<T, AsyncStdTimer>;

#[async_std::test]
async fn test_sample_across_threads() {
    // Arrange
    let timer = AsyncStdTimer;
    let (tx, stream) = test_channel::<AsyncStdTimestamped<Person>>();

    let handle = async_std::task::spawn(async move {
        let mut sampled = stream.sample(Duration::from_millis(100));
        unwrap_stream(&mut sampled, 200).await
    });

    // Act
    tx.unbounded_send(AsyncStdTimestamped::new(person_alice(), timer.now()))
        .unwrap();

    async_io::Timer::after(Duration::from_millis(150)).await;
    drop(tx);

    // Assert
    let result = handle.await;
    assert_eq!(result.unwrap().value, person_alice());
}
