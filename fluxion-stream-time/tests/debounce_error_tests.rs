// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::FluxionStream;
use fluxion_stream_time::{ChronoStreamOps, ChronoTimestamped};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel_with_errors,
    test_data::{person_alice, person_bob},
    TestData,
};
use tokio::time::{advance, pause};

#[tokio::test]
async fn test_debounce_errors_pass_through() -> anyhow::Result<()> {
    // Arrange
    pause();

    let (tx, stream) = test_channel_with_errors::<ChronoTimestamped<TestData>>();
    let debounce_duration = std::time::Duration::from_millis(500);
    let mut debounced = FluxionStream::new(stream).debounce(debounce_duration);

    // Act & Assert
    tx.send(StreamItem::Value(ChronoTimestamped::now(person_alice())))?;
    assert_no_element_emitted(&mut debounced, 0).await; // Poll the stream to let debounce see the value

    advance(std::time::Duration::from_millis(300)).await;
    assert_no_element_emitted(&mut debounced, 0).await;

    tx.send(StreamItem::Error(FluxionError::stream_error("test error")))?;
    assert!(matches!(
        unwrap_stream(&mut debounced, 100).await,
        StreamItem::Error(_)
    ));

    advance(std::time::Duration::from_millis(300)).await;
    assert_no_element_emitted(&mut debounced, 0).await;

    tx.send(StreamItem::Value(ChronoTimestamped::now(person_bob())))?;
    assert_no_element_emitted(&mut debounced, 0).await; // Poll the stream to let debounce see the value

    advance(std::time::Duration::from_millis(300)).await;
    assert_no_element_emitted(&mut debounced, 0).await;

    advance(std::time::Duration::from_millis(200)).await;
    assert_eq!(
        unwrap_stream(&mut debounced, 100).await.unwrap().value,
        person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_debounce_error_discards_pending() -> anyhow::Result<()> {
    // Arrange
    pause();

    let (tx, stream) = test_channel_with_errors::<ChronoTimestamped<TestData>>();
    let debounce_duration = std::time::Duration::from_millis(500);
    let mut debounced = FluxionStream::new(stream).debounce(debounce_duration);

    // Act & Assert
    tx.send(StreamItem::Value(ChronoTimestamped::now(person_alice())))?;

    advance(std::time::Duration::from_millis(200)).await;
    tx.send(StreamItem::Value(ChronoTimestamped::now(person_bob())))?;

    advance(std::time::Duration::from_millis(200)).await;
    tx.send(StreamItem::Error(FluxionError::stream_error("test error")))?;
    assert!(matches!(
        unwrap_stream(&mut debounced, 100).await,
        StreamItem::Error(_)
    ));

    advance(std::time::Duration::from_millis(500)).await;
    assert_no_element_emitted(&mut debounced, 100).await;

    Ok(())
}
