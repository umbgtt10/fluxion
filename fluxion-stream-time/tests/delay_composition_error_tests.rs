// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::EmitWhenExt;
use fluxion_stream_time::{ChronoStreamOps, ChronoTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel_with_errors,
    test_data::{person_alice, person_bob},
    TestData,
};
use tokio::time::{advance, pause};

#[tokio::test]
async fn test_emit_when_delay_error_propagation() -> anyhow::Result<()> {
    // Arrange
    pause();

    let (tx_source, source) = test_channel_with_errors::<ChronoTimestamped<TestData>>();
    let (tx_filter, filter) = test_channel_with_errors::<ChronoTimestamped<TestData>>();
    let delay_duration = std::time::Duration::from_millis(200);

    // Chain emit_when then delay
    // emit_when gates source emissions based on filter state
    let mut processed = source.emit_when(filter, |_| true).delay(delay_duration);

    // Act & Assert
    tx_filter.send(StreamItem::Value(ChronoTimestamped::now(person_bob())))?;
    tx_source.send(StreamItem::Value(ChronoTimestamped::now(person_alice())))?;
    assert_no_element_emitted(&mut processed, 0).await;

    let error = FluxionError::stream_error("filter error");
    tx_filter.send(StreamItem::Error(error.clone()))?;
    assert_eq!(
        unwrap_stream(&mut processed, 100)
            .await
            .err()
            .unwrap()
            .to_string(),
        error.to_string()
    );

    advance(std::time::Duration::from_millis(200)).await;
    assert_eq!(
        unwrap_stream(&mut processed, 100).await.unwrap().value,
        person_alice()
    );

    Ok(())
}
