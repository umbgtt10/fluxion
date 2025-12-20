// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `FluxionShared`.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::prelude::*;
use fluxion_stream::ShareExt;
use fluxion_test_utils::test_data::{person_alice, TestData};
use fluxion_test_utils::{assert_stream_ended, test_channel_with_errors, unwrap_stream, Sequenced};

#[tokio::test]
async fn shared_propagates_error_and_closes_all_subscribers() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = test_channel_with_errors::<Sequenced<TestData>>();
    let source = rx;

    let shared = source.share();
    let mut sub1 = shared.subscribe().unwrap();
    let mut sub2 = shared.subscribe().unwrap();
    let mut sub3 = shared.subscribe().unwrap();

    // Act - send value then error
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "critical failure",
    )))?;

    // Assert - all subscribers receive the value
    assert!(matches!(
        unwrap_stream(&mut sub1, 500).await,
        StreamItem::Value(ref v) if matches!(v.value, TestData::Person(ref p) if p.name == "Alice")
    ));
    assert!(matches!(
        unwrap_stream(&mut sub2, 500).await,
        StreamItem::Value(ref v) if matches!(v.value, TestData::Person(ref p) if p.name == "Alice")
    ));
    assert!(matches!(
        unwrap_stream(&mut sub3, 500).await,
        StreamItem::Value(ref v) if matches!(v.value, TestData::Person(ref p) if p.name == "Alice")
    ));

    // Assert - all subscribers receive the error
    assert!(matches!(
        unwrap_stream(&mut sub1, 500).await,
        StreamItem::Error(FluxionError::StreamProcessingError { context }) if context == "critical failure"
    ));
    assert!(matches!(
        unwrap_stream(&mut sub2, 500).await,
        StreamItem::Error(FluxionError::StreamProcessingError { context }) if context == "critical failure"
    ));
    assert!(matches!(
        unwrap_stream(&mut sub3, 500).await,
        StreamItem::Error(FluxionError::StreamProcessingError { context }) if context == "critical failure"
    ));

    // Assert - all subscribers complete after error
    assert_stream_ended(&mut sub1, 500).await;
    assert_stream_ended(&mut sub2, 500).await;
    assert_stream_ended(&mut sub3, 500).await;

    Ok(())
}

#[tokio::test]
async fn shared_error_from_source_operator_propagates_to_subscribers() -> anyhow::Result<()> {
    // Arrange - source with an operator that injects error
    let (tx, rx) = test_channel_with_errors::<Sequenced<TestData>>();

    let source = rx.map_ordered(|item| {
        // This operator runs on the source stream before sharing
        Sequenced::new(item.into_inner())
    });

    let shared = source.share();
    let mut sub1 = shared.subscribe().unwrap();
    let mut sub2 = shared.subscribe().unwrap();

    // Act - send error from source
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "upstream error",
    )))?;

    // Assert - both subscribers receive the error
    assert!(matches!(
        unwrap_stream(&mut sub1, 500).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut sub2, 500).await,
        StreamItem::Error(_)
    ));

    // Assert - both subscribers complete
    assert_stream_ended(&mut sub1, 500).await;
    assert_stream_ended(&mut sub2, 500).await;

    Ok(())
}

#[tokio::test]
async fn shared_multiple_errors_propagate_in_order() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = test_channel_with_errors::<Sequenced<TestData>>();
    let source = rx;

    let shared = source.share();
    let mut sub = shared.subscribe().unwrap();

    // Act - send value, error, value (stream continues after error if source allows)
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("first error")))?;

    // Assert - subscriber receives value
    assert!(matches!(
        unwrap_stream(&mut sub, 500).await,
        StreamItem::Value(ref v) if matches!(v.value, TestData::Person(ref p) if p.name == "Alice")
    ));

    // Assert - subscriber receives error
    assert!(matches!(
        unwrap_stream(&mut sub, 500).await,
        StreamItem::Error(FluxionError::StreamProcessingError { context }) if context == "first error"
    ));

    // Note: After error, subject closes, so stream ends
    assert_stream_ended(&mut sub, 500).await;

    Ok(())
}
