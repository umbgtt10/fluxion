// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::async_std::helpers::{person_alice, test_channel, unwrap_stream, Person};
use fluxion_runtime::impls::async_std::AsyncStdTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream_time::{AsyncStdTimestamped, DebounceExt};
use std::time::Duration;

#[async_std::test]
async fn test_debounce_basic() {
    // Arrange
    let timer = AsyncStdTimer;
    let (tx, stream) = test_channel::<AsyncStdTimestamped<Person>>();
    let mut debounced = stream.debounce(Duration::from_millis(100));

    // Act
    tx.unbounded_send(AsyncStdTimestamped::new(person_alice(), timer.now()))
        .unwrap();

    async_io::Timer::after(Duration::from_millis(150)).await;

    // Assert
    let result = unwrap_stream(&mut debounced, 200).await;
    assert_eq!(result.unwrap().value, person_alice());
}
