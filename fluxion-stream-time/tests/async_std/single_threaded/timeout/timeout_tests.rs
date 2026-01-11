// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::async_std::helpers::{person_alice, test_channel, unwrap_stream, Person};
use fluxion_runtime::impls::async_std::AsyncStdTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream_time::{AsyncStdTimestamped, TimeoutExt};
use std::time::Duration;

#[async_std::test]
async fn test_timeout_basic() {
    // Arrange
    let timer = AsyncStdTimer;
    let (tx, stream) = test_channel::<AsyncStdTimestamped<Person>>();
    let mut timed = stream.timeout(Duration::from_millis(200));

    // Act
    tx.unbounded_send(AsyncStdTimestamped::new(person_alice(), timer.now()))
        .unwrap();

    // Assert
    let result = unwrap_stream(&mut timed, 300).await;
    assert_eq!(result.unwrap().value, person_alice());
}
