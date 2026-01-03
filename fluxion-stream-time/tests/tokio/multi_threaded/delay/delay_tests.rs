// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

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

    let handle = tokio::spawn(async move {
        let mut delayed = stream.delay(Duration::from_millis(50));
        delayed.next().await
    });

    // Act
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    drop(tx);

    // Assert
    assert_eq!(
        handle.await.unwrap().unwrap().unwrap().value,
        person_alice()
    );

    Ok(())
}
