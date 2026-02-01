// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `take_while_with` operator.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::TakeWhileExt;
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, test_channel_with_errors, unwrap_stream},
    sequenced::Sequenced,
    test_data::{person_alice, TestData},
};

#[tokio::test]
async fn test_take_while_with_propagates_source_error() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act
    filter_tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Act
    source_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Source error",
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Assert
    assert_no_element_emitted(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_propagates_filter_error() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act
    filter_tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Act
    filter_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Filter error",
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_predicate_after_error() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act
    filter_tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Act
    source_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_error_at_start() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act
    filter_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Early error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        2,
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_error_recovery() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act
    filter_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        2,
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Act
    source_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("err1")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // aCT
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        3,
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Act
    filter_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("err2")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        4,
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_filter_error_at_start() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act
    filter_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Early filter error",
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    filter_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        2,
    )))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        3,
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    Ok(())
}
