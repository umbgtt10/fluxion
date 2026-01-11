// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_runtime::impls::tokio::TokioTimer;
use fluxion_runtime::timer::Timer;
use fluxion_stream_time::{DebounceExt, TokioTimestamped};
use fluxion_test_utils::{test_channel, test_data::person_alice, unwrap_stream, TestData};
use std::time::Duration;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_debounce_across_threads() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();

    let handle = tokio::spawn(async move {
        let mut debounced = stream.debounce(Duration::from_millis(100));
        unwrap_stream(&mut debounced, 200).await
    });

    // Act
    tx.unbounded_send(TokioTimestamped::new(person_alice(), timer.now()))?;
    drop(tx);

    // Assert
    assert_eq!(handle.await.unwrap().unwrap().value, person_alice());

    Ok(())
}
