// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `take_while_with` operator.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::TakeWhileExt;
use fluxion_test_utils::{
    assert_no_element_emitted, assert_stream_ended, test_channel_with_errors,
    test_data::{person_alice, TestData},
    unwrap_stream, Sequenced,
};

#[tokio::test]
async fn test_take_while_with_propagates_source_error() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act & Assert: Send filter value first then source value
    filter_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;
    assert_no_element_emitted(&mut result, 100).await;
    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        2,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Send error in source
    source_tx.send(StreamItem::Error(FluxionError::stream_error(
        "Source error",
    )))?;

    // Error should be propagated
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Stream continues after error
    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        3,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Drop channels to close streams
    drop(source_tx);
    drop(filter_tx);

    // stream terminates when channels close
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_propagates_filter_error() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act & Assert: Send filter value
    filter_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;
    assert_no_element_emitted(&mut result, 100).await;
    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        2,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Send error in filter
    filter_tx.send(StreamItem::Error(FluxionError::stream_error(
        "Filter error",
    )))?;

    // Error should be propagated
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Stream continues after error
    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        3,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(source_tx);
    drop(filter_tx);

    // stream terminates when channels close
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_predicate_after_error() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act & Assert: Send filter value
    filter_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;
    assert_no_element_emitted(&mut result, 100).await;
    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        2,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Send error in source
    source_tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;

    // Error should be propagated
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Stream continues - send more values
    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        3,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(source_tx);
    drop(filter_tx);

    // Stream terminates when channels close
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_error_at_start() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act & Assert: Send filter value
    filter_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;
    assert_no_element_emitted(&mut result, 100).await;
    source_tx.send(StreamItem::Error(FluxionError::stream_error("Early error")))?;

    // Error should be propagated
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Stream continues
    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        2,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(source_tx);
    drop(filter_tx);

    // Stream terminates when channels close
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_error_recovery() -> anyhow::Result<()> {
    // Test that stream continues after errors
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    filter_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;
    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        2,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Error from source
    source_tx.send(StreamItem::Error(FluxionError::stream_error("err1")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Continue
    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        3,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));
    // Error from filter
    filter_tx.send(StreamItem::Error(FluxionError::stream_error("err2")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Continue
    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        4,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));
    drop(source_tx);
    drop(filter_tx);

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_filter_error_at_start() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act & Assert: Send filter error immediately
    filter_tx.send(StreamItem::Error(FluxionError::stream_error(
        "Early filter error",
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    filter_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        2,
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        3,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(source_tx);
    drop(filter_tx);

    assert_stream_ended(&mut result, 500).await;

    Ok(())
}
