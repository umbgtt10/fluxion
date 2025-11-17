// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `emit_when` operator.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::EmitWhenExt;
use fluxion_test_utils::{sequenced::Sequenced, test_channel_with_errors};
use futures::StreamExt;

#[tokio::test]
async fn test_emit_when_propagates_source_error() {
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result =
        source_stream.emit_when(filter_stream, |state| state.values()[0] > state.values()[1]);

    // Send filter value first
    filter_tx
        .send(StreamItem::Value(Sequenced::with_sequence(15, 1)))
        .unwrap();

    // Send source value
    source_tx
        .send(StreamItem::Value(Sequenced::with_sequence(10, 2)))
        .unwrap();

    // Send error in source
    source_tx
        .send(StreamItem::Error(FluxionError::stream_error(
            "Source error",
        )))
        .unwrap();

    let result1 = result.next().await.unwrap();
    assert!(matches!(result1, StreamItem::Error(_)));

    // Continue with value
    source_tx
        .send(StreamItem::Value(Sequenced::with_sequence(30, 3)))
        .unwrap();

    let result2 = result.next().await.unwrap();
    assert!(matches!(result2, StreamItem::Value(_)));

    drop(source_tx);
    drop(filter_tx);
}

#[tokio::test]
async fn test_emit_when_propagates_filter_error() {
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result =
        source_stream.emit_when(filter_stream, |state| state.values()[0] > state.values()[1]);

    // Send filter value
    filter_tx
        .send(StreamItem::Value(Sequenced::with_sequence(5, 1)))
        .unwrap();

    // Send source value
    source_tx
        .send(StreamItem::Value(Sequenced::with_sequence(10, 2)))
        .unwrap();

    let result1 = result.next().await.unwrap();
    assert!(matches!(result1, StreamItem::Value(_)));

    // Send error in filter
    filter_tx
        .send(StreamItem::Error(FluxionError::stream_error(
            "Filter error",
        )))
        .unwrap();

    let result2 = result.next().await.unwrap();
    assert!(matches!(result2, StreamItem::Error(_)));

    // Continue
    filter_tx
        .send(StreamItem::Value(Sequenced::with_sequence(15, 4)))
        .unwrap();
    source_tx
        .send(StreamItem::Value(Sequenced::with_sequence(20, 3)))
        .unwrap();

    let result3 = result.next().await.unwrap();
    assert!(matches!(result3, StreamItem::Value(_)));

    drop(source_tx);
    drop(filter_tx);
}

#[tokio::test]
async fn test_emit_when_predicate_continues_after_error() {
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Only emit when source > filter
    let mut result =
        source_stream.emit_when(filter_stream, |state| state.values()[0] > state.values()[1]);

    // Send filter value
    filter_tx
        .send(StreamItem::Value(Sequenced::with_sequence(15, 1)))
        .unwrap();

    // Send source value (doesn't pass predicate)
    source_tx
        .send(StreamItem::Value(Sequenced::with_sequence(10, 2)))
        .unwrap();

    // Send error in source
    source_tx
        .send(StreamItem::Error(FluxionError::stream_error("Error")))
        .unwrap();

    let result1 = result.next().await.unwrap();
    assert!(matches!(result1, StreamItem::Error(_)));

    // Send value that passes predicate
    filter_tx
        .send(StreamItem::Value(Sequenced::with_sequence(25, 4)))
        .unwrap();
    source_tx
        .send(StreamItem::Value(Sequenced::with_sequence(30, 5)))
        .unwrap();

    let result2 = result.next().await.unwrap();
    assert!(matches!(result2, StreamItem::Value(_)));

    drop(source_tx);
    drop(filter_tx);
}

#[tokio::test]
async fn test_emit_when_both_streams_have_errors() {
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result =
        source_stream.emit_when(filter_stream, |state| state.values()[0] > state.values()[1]);

    // Send initial values
    filter_tx
        .send(StreamItem::Value(Sequenced::with_sequence(5, 1)))
        .unwrap();
    source_tx
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

    // Error from filter
    filter_tx
        .send(StreamItem::Error(FluxionError::stream_error(
            "Filter error",
        )))
        .unwrap();

    let result3 = result.next().await.unwrap();
    assert!(matches!(result3, StreamItem::Error(_)));

    drop(source_tx);
    drop(filter_tx);
}

#[tokio::test]
async fn test_emit_when_error_before_filter_ready() {
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result =
        source_stream.emit_when(filter_stream, |state| state.values()[0] > state.values()[1]);

    // Error immediately before filter has value
    source_tx
        .send(StreamItem::Error(FluxionError::stream_error("Early error")))
        .unwrap();

    let first = result.next().await.unwrap();
    assert!(matches!(first, StreamItem::Error(_)));

    // Continue
    filter_tx
        .send(StreamItem::Value(Sequenced::with_sequence(5, 2)))
        .unwrap();
    source_tx
        .send(StreamItem::Value(Sequenced::with_sequence(10, 1)))
        .unwrap();

    let result2 = result.next().await.unwrap();
    assert!(matches!(result2, StreamItem::Value(_)));

    drop(source_tx);
    drop(filter_tx);
}
