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
async fn test_delay_errors_pass_through() -> anyhow::Result<()> {
    // Arrange
    pause();
    let (tx, stream) = test_channel_with_errors::<ChronoTimestamped<TestData>>();
    let delay_duration = std::time::Duration::from_secs(1);
    let mut delayed = FluxionStream::new(stream).delay(delay_duration);

    // Act & Assert
    tx.send(StreamItem::Value(ChronoTimestamped::now(person_alice())))?;
    advance(std::time::Duration::from_millis(100)).await;
    assert_no_element_emitted(&mut delayed, 100).await;

    advance(std::time::Duration::from_millis(900)).await;
    assert_eq!(
        unwrap_stream(&mut delayed, 100).await.unwrap().value,
        person_alice()
    );

    tx.send(StreamItem::Error(FluxionError::stream_error("test error")))?;
    assert!(matches!(
        unwrap_stream(&mut delayed, 100).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Value(ChronoTimestamped::now(person_bob())))?;
    advance(std::time::Duration::from_millis(100)).await;
    assert_no_element_emitted(&mut delayed, 100).await;

    advance(std::time::Duration::from_millis(900)).await;
    assert_eq!(
        unwrap_stream(&mut delayed, 100).await.unwrap().value,
        person_bob()
    );

    Ok(())
}
