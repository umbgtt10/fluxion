// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `take_while_with` operator.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::TakeWhileExt;
use fluxion_test_utils::{
    assert_no_element_emitted, sequenced::Sequenced, test_channel_with_errors, unwrap_stream,
};
use futures::StreamExt;

#[tokio::test]
async fn test_take_while_with_propagates_source_error() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut filtered_stream = source_stream.take_while_with(filter_stream, |f| *f > 0);

    // Act & Assert: Send filter value first then source value
    filter_tx.send(StreamItem::Value(Sequenced::with_sequence(100, 1)))?;
    assert_no_element_emitted(&mut filtered_stream, 100).await;
    source_tx.send(StreamItem::Value(Sequenced::with_sequence(1, 2)))?;
    assert!(matches!(
        unwrap_stream(&mut filtered_stream, 100).await,
        StreamItem::Value(_)
    ));

    // Send error in source
    source_tx.send(StreamItem::Error(FluxionError::stream_error(
        "Source error",
    )))?;

    // Drop channels to close streams
    drop(source_tx);
    drop(filter_tx);

    // stream terminates on error
    let element = filtered_stream.next().await;
    assert!(element.is_none(), "Stream should terminate on error");

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_propagates_filter_error() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut filtered_stream = source_stream.take_while_with(filter_stream, |f| *f > 0);

    // Act & Assert: Send filter value
    filter_tx.send(StreamItem::Value(Sequenced::with_sequence(100, 1)))?;
    assert_no_element_emitted(&mut filtered_stream, 100).await;
    source_tx.send(StreamItem::Value(Sequenced::with_sequence(1, 2)))?;
    assert!(matches!(
        unwrap_stream(&mut filtered_stream, 100).await,
        StreamItem::Value(_)
    ));

    // Send error in filter
    filter_tx.send(StreamItem::Error(FluxionError::stream_error(
        "Filter error",
    )))?;

    drop(source_tx);
    drop(filter_tx);

    // stream terminates on error
    let element = filtered_stream.next().await;
    assert!(element.is_none(), "Stream should terminate on error");

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_predicate_after_error() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = source_stream.take_while_with(filter_stream, |f| *f > 0);

    // Act & Assert: Send filter value
    filter_tx.send(StreamItem::Value(Sequenced::with_sequence(100, 1)))?;
    assert_no_element_emitted(&mut result, 100).await;
    source_tx.send(StreamItem::Value(Sequenced::with_sequence(1, 2)))?;
    assert!(matches!(result.next().await.unwrap(), StreamItem::Value(_)));

    // Send error in source
    source_tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;

    drop(source_tx);
    drop(filter_tx);

    // Stream terminates on error
    let element = result.next().await;
    assert!(element.is_none(), "Stream should terminate on error");

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_error_at_start() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = source_stream.take_while_with(filter_stream, |f| *f > 0);

    // Act & Assert: Send filter value
    filter_tx.send(StreamItem::Value(Sequenced::with_sequence(100, 1)))?;
    assert_no_element_emitted(&mut result, 100).await;
    source_tx.send(StreamItem::Error(FluxionError::stream_error("Early error")))?;

    drop(source_tx);
    drop(filter_tx);

    // Stream terminates on error
    let element = result.next().await;
    assert!(element.is_none(), "Stream should terminate on error");

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_stops_on_false_despite_errors() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = source_stream.take_while_with(filter_stream, |f| *f > 0);

    // Act & Assert: Send filter value that passes
    filter_tx.send(StreamItem::Value(Sequenced::with_sequence(100, 1)))?;
    assert_no_element_emitted(&mut result, 100).await;
    source_tx.send(StreamItem::Value(Sequenced::with_sequence(1, 2)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Send filter value that fails predicate (stops stream)
    filter_tx.send(StreamItem::Value(Sequenced::with_sequence(0, 3)))?;

    drop(source_tx);
    drop(filter_tx);

    // Stream should terminate when predicate returns false
    let element = result.next().await;
    assert!(element.is_none(), "Stream should stop on false predicate");

    Ok(())
}
