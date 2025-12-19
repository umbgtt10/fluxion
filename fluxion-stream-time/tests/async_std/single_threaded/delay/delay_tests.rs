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
async fn test_delay_basic() {
    // Arrange
    let timer = AsyncStdTimer;
    let (tx, stream) = test_channel::<AsyncStdTimestamped<Person>>();
    let mut delayed = stream.delay(Duration::from_millis(100), timer.clone());

    // Act
    let start = timer.now();
    tx.unbounded_send(AsyncStdTimestamped::new(person_alice(), timer.now()))
        .unwrap();

    // Assert
    let result = unwrap_stream(&mut delayed, 200).await;
    let elapsed = timer.now().duration_since(start);

    assert_eq!(result.unwrap().value, person_alice());
    assert!(elapsed >= Duration::from_millis(100));
}
