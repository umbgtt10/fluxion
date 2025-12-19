// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream_time::{timer::Timer, DebounceExt, TokioTimer, TokioTimestamped};
use fluxion_test_utils::{test_channel, test_data::person_alice, unwrap_stream, TestData};
use std::time::Duration;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_debounce_across_threads() -> anyhow::Result<()> {
    // Arrange
    let timer = TokioTimer;
    let (tx, stream) = test_channel::<TokioTimestamped<TestData>>();

    // Spawn on different thread
    let timer_clone = timer.clone();
    let handle = tokio::spawn(async move {
        let mut debounced = stream.debounce(Duration::from_millis(100), timer_clone);
        unwrap_stream(&mut debounced, 200).await
    });

    // Act
    tx.send(TokioTimestamped::new(person_alice(), timer.now()))?;
    drop(tx);

    // Assert
    let result = handle.await.unwrap();
    assert_eq!(result.unwrap().value, person_alice());

    Ok(())
}
