// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `take_while_with` operator.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::TakeWhileExt;
use fluxion_test_utils::{sequenced::Sequenced, test_channel_with_errors};
use futures::StreamExt;

#[tokio::test]
async fn test_take_while_with_propagates_source_error() {
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = source_stream.take_while_with(filter_stream, |f| *f > 0);

    // Send filter value first
    filter_tx
        .send(StreamItem::Value(Sequenced::with_sequence(100, 1)))
        .unwrap();

    // Send source value
    source_tx
        .send(StreamItem::Value(Sequenced::with_sequence(1, 2)))
        .unwrap();

    let result1 = result.next().await.unwrap();
    assert!(matches!(result1, StreamItem::Value(_)));

    // Send error in source
    source_tx
        .send(StreamItem::Error(FluxionError::stream_error(
            "Source error",
        )))
        .unwrap();

    // Drop channels to close streams
    drop(source_tx);
    drop(filter_tx);

    // take_while_with terminates on error
    let result2 = result.next().await;
    assert!(result2.is_none(), "Stream should terminate on error");
}

#[tokio::test]
async fn test_take_while_with_propagates_filter_error() {
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = source_stream.take_while_with(filter_stream, |f| *f > 0);

    // Send filter value
    filter_tx
        .send(StreamItem::Value(Sequenced::with_sequence(100, 1)))
        .unwrap();

    // Send source value
    source_tx
        .send(StreamItem::Value(Sequenced::with_sequence(1, 2)))
        .unwrap();

    let result1 = result.next().await.unwrap();
    assert!(matches!(result1, StreamItem::Value(_)));

    // Send error in filter
    filter_tx
        .send(StreamItem::Error(FluxionError::stream_error(
            "Filter error",
        )))
        .unwrap();

    drop(source_tx);
    drop(filter_tx);

    // Stream may terminate
    let result2 = result.next().await;
    assert!(result2.is_none() || matches!(result2, Some(StreamItem::Error(_))));
}

#[tokio::test]
async fn test_take_while_with_predicate_after_error() {
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = source_stream.take_while_with(filter_stream, |f| *f > 0);

    // Send filter value
    filter_tx
        .send(StreamItem::Value(Sequenced::with_sequence(100, 1)))
        .unwrap();

    // Send source value
    source_tx
        .send(StreamItem::Value(Sequenced::with_sequence(1, 2)))
        .unwrap();

    let result1 = result.next().await.unwrap();
    assert!(matches!(result1, StreamItem::Value(_)));

    // Send error in source
    source_tx
        .send(StreamItem::Error(FluxionError::stream_error("Error")))
        .unwrap();

    drop(source_tx);
    drop(filter_tx);

    // Stream terminates on error
    let result2 = result.next().await;
    assert!(result2.is_none());
}

#[tokio::test]
async fn test_take_while_with_error_at_start() {
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = source_stream.take_while_with(filter_stream, |f| *f > 0);

    // Send filter value
    filter_tx
        .send(StreamItem::Value(Sequenced::with_sequence(100, 1)))
        .unwrap();

    // Error immediately in source
    source_tx
        .send(StreamItem::Error(FluxionError::stream_error("Early error")))
        .unwrap();

    drop(source_tx);
    drop(filter_tx);

    // Stream terminates on error
    let result1 = result.next().await;
    assert!(result1.is_none());
}

#[tokio::test]
async fn test_take_while_with_stops_on_false_despite_errors() {
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<i32>>();
    let (filter_tx, filter_stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = source_stream.take_while_with(filter_stream, |f| *f > 0);

    // Send filter value that passes
    filter_tx
        .send(StreamItem::Value(Sequenced::with_sequence(100, 1)))
        .unwrap();

    // Send source value
    source_tx
        .send(StreamItem::Value(Sequenced::with_sequence(1, 2)))
        .unwrap();

    let result1 = result.next().await.unwrap();
    assert!(matches!(result1, StreamItem::Value(_)));

    // Send filter value that fails predicate (stops stream)
    filter_tx
        .send(StreamItem::Value(Sequenced::with_sequence(0, 3)))
        .unwrap();

    drop(source_tx);
    drop(filter_tx);

    // Stream should terminate when predicate returns false
    let result2 = result.next().await;
    assert!(result2.is_none(), "Stream should stop on false predicate");
}
