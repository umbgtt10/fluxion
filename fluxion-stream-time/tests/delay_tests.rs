// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use chrono::Duration;
use fluxion_stream::FluxionStream;
use fluxion_stream_time::{ChronoStreamOps, ChronoTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel,
    test_data::{person_alice, person_bob},
    TestData,
};
use futures::StreamExt;
use tokio::time::{advance, pause};
use tokio::{spawn, sync::mpsc::unbounded_channel};

#[tokio::test]
async fn test_delay_with_chrono_timestamped() -> anyhow::Result<()> {
    // Arrange
    pause();
    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let delay_duration = Duration::seconds(1);
    let mut delayed = FluxionStream::new(stream).delay(delay_duration);

    // Act & Assert
    tx.send(ChronoTimestamped::now(person_alice()))?;
    advance(std::time::Duration::from_millis(100)).await;
    assert_no_element_emitted(&mut delayed, 100).await;

    advance(std::time::Duration::from_millis(900)).await;
    assert_eq!(
        unwrap_stream(&mut delayed, 100).await.unwrap().value,
        person_alice()
    );

    tx.send(ChronoTimestamped::now(person_bob()))?;
    advance(std::time::Duration::from_millis(100)).await;
    assert_no_element_emitted(&mut delayed, 100).await;

    advance(std::time::Duration::from_millis(900)).await;
    assert_eq!(
        unwrap_stream(&mut delayed, 100).await.unwrap().value,
        person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_delay_preserves_order() -> anyhow::Result<()> {
    // Arrange
    pause();
    
    let (tx, stream) = test_channel::<ChronoTimestamped<i32>>();
    let delay_duration = Duration::milliseconds(10);
    let delayed = FluxionStream::new(stream).delay(delay_duration);

    let count = 1000;
    let (result_tx, mut result_rx) = unbounded_channel();

    // Spawn a task to drive the stream
    spawn(async move {
        let mut stream = delayed.map_ordered(|item| item.value);
        while let Some(item) = stream.next().await {
            result_tx.send(item.unwrap()).unwrap();
        }
    });

    // Act
    for i in 0..count {
        tx.send(ChronoTimestamped::now(i))?;
    }
    // Drop tx to signal end of stream
    drop(tx);

    // Advance time to ensure all delays expire
    advance(std::time::Duration::from_millis(100)).await;

    // Assert
    let mut results = Vec::with_capacity(count as usize);
    while let Some(item) = result_rx.recv().await {
        results.push(item);
    }

    assert_eq!(results.len(), count as usize);

    for i in 0..count {
        assert_eq!(results[i as usize], i, "Order mismatch at index {}", i);
    }

    Ok(())
}
