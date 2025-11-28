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

#[tokio::test]
async fn test_delay_with_chrono_timestamped() -> anyhow::Result<()> {
    tokio::time::pause(); // Mock time for instant test execution

    // Arrange
    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let delay_duration = Duration::seconds(1);
    let mut delayed = FluxionStream::new(stream).delay(delay_duration);

    // Act - Send first value
    tx.send(ChronoTimestamped::now(person_alice()))?;

    // Assert - Should NOT arrive immediately (advance 100ms)
    tokio::time::advance(std::time::Duration::from_millis(100)).await;
    tokio::task::yield_now().await; // Allow tasks to process
    assert_no_element_emitted(&mut delayed, 100).await;

    // Assert - Advance remaining time, should arrive
    tokio::time::advance(std::time::Duration::from_millis(900)).await;
    tokio::task::yield_now().await; // Allow tasks to process
    let result = unwrap_stream(&mut delayed, 100).await.unwrap();
    assert_eq!(result.value, person_alice());

    // Act - Send second value
    tx.send(ChronoTimestamped::now(person_bob()))?;

    // Assert - Should NOT arrive immediately (advance 100ms)
    tokio::time::advance(std::time::Duration::from_millis(100)).await;
    tokio::task::yield_now().await; // Allow tasks to process
    assert_no_element_emitted(&mut delayed, 100).await;

    // Assert - Advance remaining time, should arrive
    tokio::time::advance(std::time::Duration::from_millis(900)).await;
    tokio::task::yield_now().await; // Allow tasks to process
    let result = unwrap_stream(&mut delayed, 100).await.unwrap();
    assert_eq!(result.value, person_bob());

    Ok(())
}
