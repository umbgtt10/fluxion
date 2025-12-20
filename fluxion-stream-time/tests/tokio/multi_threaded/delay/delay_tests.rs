// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream_time::prelude::*;
use fluxion_stream_time::{timer::Timer, TokioTimer, TokioTimestamped};
use fluxion_test_utils::{test_channel, test_data::person_alice, TestData};
use futures::StreamExt;
use std::time::Duration;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_delay_across_threads() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();

    // Spawn on different thread
    let handle = tokio::spawn(async move {
        let mut delayed = stream.delay(Duration::from_millis(50));
        delayed.next().await
    });

    // Act
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    drop(tx);

    // Assert
    let result = handle.await.unwrap();
    assert_eq!(result.unwrap().unwrap().value, person_alice());

    Ok(())
}
