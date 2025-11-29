// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::TakeLatestWhenExt;
use fluxion_stream_time::{ChronoStreamOps, ChronoTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel_with_errors,
    test_data::{person_alice, person_bob},
    TestData,
};
use std::time::Duration;
use tokio::time::{advance, pause};

#[tokio::test]
async fn test_take_latest_when_debounce_error_propagation() -> anyhow::Result<()> {
    // Arrange
    pause();

    let (tx_source, source) = test_channel_with_errors::<ChronoTimestamped<TestData>>();
    let (tx_trigger, trigger) = test_channel_with_errors::<ChronoTimestamped<TestData>>();
    let debounce_duration = Duration::from_millis(500);

    // Chain take_latest_when then debounce
    // take_latest_when emits the latest source value when trigger emits
    let mut processed = source
        .take_latest_when(trigger, |_| true)
        .debounce(debounce_duration);

    // Act & Assert
    tx_source.send(StreamItem::Value(ChronoTimestamped::now(person_alice())))?;
    let error = FluxionError::stream_error("trigger error");
    tx_trigger.send(StreamItem::Error(error.clone()))?;

    assert_eq!(
        unwrap_stream(&mut processed, 100)
            .await
            .err()
            .unwrap()
            .to_string(),
        error.to_string()
    );

    tx_trigger.send(StreamItem::Value(ChronoTimestamped::now(person_bob())))?;
    assert_no_element_emitted(&mut processed, 0).await;

    advance(Duration::from_millis(500)).await;
    assert_eq!(
        unwrap_stream(&mut processed, 100).await.unwrap().value,
        person_alice()
    );

    Ok(())
}
