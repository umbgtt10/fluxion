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
use tokio::time::{advance, pause};

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
