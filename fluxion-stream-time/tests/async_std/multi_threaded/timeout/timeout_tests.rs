// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::async_std::helpers::{person_alice, test_channel, Person};
use fluxion_runtime::impls::async_std::AsyncStdTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream_time::{AsyncStdTimestamped, TimeoutExt};
use futures::StreamExt;
use std::time::Duration;

#[async_std::test]
async fn test_timeout_across_threads() {
    // Arrange
    let timer = AsyncStdTimer;
    let (tx, stream) = test_channel::<AsyncStdTimestamped<Person>>();

    let handle = async_std::task::spawn(async move {
        let mut timed = stream.timeout(Duration::from_millis(200));
        timed.next().await
    });

    // Act
    tx.try_send(AsyncStdTimestamped::new(person_alice(), timer.now()))
        .unwrap();
    drop(tx);

    // Assert
    let result = handle.await;
    assert_eq!(result.unwrap().unwrap().value, person_alice());
}
