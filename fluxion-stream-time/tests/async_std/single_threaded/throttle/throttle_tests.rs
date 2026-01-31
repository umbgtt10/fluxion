// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::async_std::helpers::{person_alice, test_channel, unwrap_stream, Person};
use fluxion_runtime::impls::async_std::AsyncStdTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream_time::{AsyncStdTimestamped, ThrottleExt};
use std::time::Duration;

#[async_std::test]
async fn test_throttle_basic() {
    // Arrange
    let timer = AsyncStdTimer;
    let (tx, stream) = test_channel::<AsyncStdTimestamped<Person>>();
    let mut throttled = stream.throttle(Duration::from_millis(100));

    // Act
    tx.try_send(AsyncStdTimestamped::new(person_alice(), timer.now()))
        .unwrap();

    // Assert
    let result = unwrap_stream(&mut throttled, 200).await;
    assert_eq!(result.unwrap().value, person_alice());
}
