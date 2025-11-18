// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `take_latest_when` operator.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::TakeLatestWhenExt;
use fluxion_test_utils::{sequenced::Sequenced, test_channel_with_errors};
use futures::StreamExt;

#[tokio::test]
async fn test_take_latest_when_propagates_source_error() -> anyhow::Result<()> {
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = source_stream.take_latest_when(trigger_stream, |_| true);

    // Send source values
    source_tx
        .send(StreamItem::Value(Sequenced::with_sequence(1, 1)))
        .unwrap();
    source_tx
        .send(StreamItem::Value(Sequenced::with_sequence(2, 2)))
        .unwrap();

    // Send trigger
    trigger_tx
        .send(StreamItem::Value(Sequenced::with_sequence(10, 4)))
        .unwrap();

    let result1 = result.next().await.unwrap();
    assert!(matches!(result1, StreamItem::Value(_)));

    // Send error in source
    source_tx
        .send(StreamItem::Error(FluxionError::stream_error(
            "Source error",
        )))
        .unwrap();

    let result2 = result.next().await.unwrap();
    assert!(matches!(result2, StreamItem::Error(_)));

    // Continue with values
    source_tx
        .send(StreamItem::Value(Sequenced::with_sequence(3, 3)))
        .unwrap();
    trigger_tx
        .send(StreamItem::Value(Sequenced::with_sequence(20, 5)))
        .unwrap();

    let result3 = result.next().await.unwrap();
    assert!(matches!(result3, StreamItem::Value(_)));

    drop(source_tx);
    drop(trigger_tx);
    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_propagates_trigger_error() -> anyhow::Result<()> {
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = source_stream.take_latest_when(trigger_stream, |_| true);

    // Send source values
    source_tx
        .send(StreamItem::Value(Sequenced::with_sequence(1, 1)))
        .unwrap();
    source_tx
        .send(StreamItem::Value(Sequenced::with_sequence(2, 2)))
        .unwrap();

    // Send trigger
    trigger_tx
        .send(StreamItem::Value(Sequenced::with_sequence(10, 3)))
        .unwrap();

    let result1 = result.next().await.unwrap();
    assert!(matches!(result1, StreamItem::Value(_)));

    // Send error in trigger
    trigger_tx
        .send(StreamItem::Error(FluxionError::stream_error(
            "Trigger error",
        )))
        .unwrap();

    let result2 = result.next().await.unwrap();
    assert!(matches!(result2, StreamItem::Error(_)));

    // Continue
    trigger_tx
        .send(StreamItem::Value(Sequenced::with_sequence(30, 5)))
        .unwrap();

    let result3 = result.next().await.unwrap();
    assert!(matches!(result3, StreamItem::Value(_)));

    drop(source_tx);
    drop(trigger_tx);
    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_filter_predicate_after_error() -> anyhow::Result<()> {
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Only emit when trigger is > 50
    let mut result = source_stream.take_latest_when(trigger_stream, |t| *t > 50);

    // Send source values
    source_tx
        .send(StreamItem::Value(Sequenced::with_sequence(1, 1)))
        .unwrap();
    source_tx
        .send(StreamItem::Value(Sequenced::with_sequence(2, 2)))
        .unwrap();

    // Send error in source
    source_tx
        .send(StreamItem::Error(FluxionError::stream_error("Error")))
        .unwrap();

    let result1 = result.next().await.unwrap();
    assert!(matches!(result1, StreamItem::Error(_)));

    // Send value and trigger that passes filter
    source_tx
        .send(StreamItem::Value(Sequenced::with_sequence(3, 3)))
        .unwrap();
    trigger_tx
        .send(StreamItem::Value(Sequenced::with_sequence(100, 5)))
        .unwrap();

    let result2 = result.next().await.unwrap();
    assert!(matches!(result2, StreamItem::Value(_)));

    drop(source_tx);
    drop(trigger_tx);
    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_both_streams_have_errors() -> anyhow::Result<()> {
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = source_stream.take_latest_when(trigger_stream, |_| true);

    // Send initial values
    source_tx
        .send(StreamItem::Value(Sequenced::with_sequence(1, 1)))
        .unwrap();
    trigger_tx
        .send(StreamItem::Value(Sequenced::with_sequence(10, 2)))
        .unwrap();

    let result1 = result.next().await.unwrap();
    assert!(matches!(result1, StreamItem::Value(_)));

    // Error from source
    source_tx
        .send(StreamItem::Error(FluxionError::stream_error(
            "Source error",
        )))
        .unwrap();

    let result2 = result.next().await.unwrap();
    assert!(matches!(result2, StreamItem::Error(_)));

    // Error from trigger
    trigger_tx
        .send(StreamItem::Error(FluxionError::stream_error(
            "Trigger error",
        )))
        .unwrap();

    let result3 = result.next().await.unwrap();
    assert!(matches!(result3, StreamItem::Error(_)));

    drop(source_tx);
    drop(trigger_tx);
    Ok(())
}
