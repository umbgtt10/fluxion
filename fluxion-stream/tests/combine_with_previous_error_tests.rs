// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `combine_with_previous` operator.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::CombineWithPreviousExt;
use fluxion_test_utils::{sequenced::Sequenced, test_channel_with_errors};
use futures::StreamExt;

#[tokio::test]
async fn test_combine_with_previous_propagates_errors() {
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = stream.combine_with_previous();

    // First item - no previous
    tx.send(StreamItem::Value(Sequenced::with_sequence(1, 1)))
        .unwrap();

    let item1 = result.next().await.unwrap();
    assert!(matches!(item1, StreamItem::Value(_)));

    // Second item - has previous
    tx.send(StreamItem::Value(Sequenced::with_sequence(2, 2)))
        .unwrap();

    let item2 = result.next().await.unwrap();
    assert!(matches!(item2, StreamItem::Value(_)));

    // Third item - error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))
        .unwrap();

    let item3 = result.next().await.unwrap();
    assert!(matches!(item3, StreamItem::Error(_)));

    // Fourth item - continues after error
    tx.send(StreamItem::Value(Sequenced::with_sequence(4, 4)))
        .unwrap();

    let item4 = result.next().await.unwrap();
    assert!(matches!(item4, StreamItem::Value(_)));

    drop(tx);
}

#[tokio::test]
async fn test_combine_with_previous_error_at_first_item() {
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = stream.combine_with_previous();

    // Error immediately
    tx.send(StreamItem::Error(FluxionError::stream_error("First error")))
        .unwrap();

    let first = result.next().await.unwrap();
    assert!(
        matches!(first, StreamItem::Error(_)),
        "First item should be error"
    );

    // Continue with value
    tx.send(StreamItem::Value(Sequenced::with_sequence(2, 2)))
        .unwrap();

    let second = result.next().await.unwrap();
    assert!(matches!(second, StreamItem::Value(_)), "Should continue");

    drop(tx);
}

#[tokio::test]
async fn test_combine_with_previous_multiple_errors() {
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = stream.combine_with_previous();

    // First value
    tx.send(StreamItem::Value(Sequenced::with_sequence(1, 1)))
        .unwrap();

    let result1 = result.next().await.unwrap();
    assert!(matches!(result1, StreamItem::Value(_)));

    // First error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error 1")))
        .unwrap();

    let result2 = result.next().await.unwrap();
    assert!(matches!(result2, StreamItem::Error(_)));

    // Value
    tx.send(StreamItem::Value(Sequenced::with_sequence(3, 3)))
        .unwrap();

    let result3 = result.next().await.unwrap();
    assert!(matches!(result3, StreamItem::Value(_)));

    // Second error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error 2")))
        .unwrap();

    let result4 = result.next().await.unwrap();
    assert!(matches!(result4, StreamItem::Error(_)));

    // Value
    tx.send(StreamItem::Value(Sequenced::with_sequence(5, 5)))
        .unwrap();

    let result5 = result.next().await.unwrap();
    assert!(matches!(result5, StreamItem::Value(_)));

    drop(tx);
}

#[tokio::test]
async fn test_combine_with_previous_preserves_pairing_after_error() {
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = stream.combine_with_previous();

    // First value
    tx.send(StreamItem::Value(Sequenced::with_sequence(10, 1)))
        .unwrap();

    let item1 = result.next().await.unwrap();
    assert!(matches!(item1, StreamItem::Value(_)));

    // Error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))
        .unwrap();

    let item2 = result.next().await.unwrap();
    assert!(matches!(item2, StreamItem::Error(_)));

    // More values - pairing should still work
    tx.send(StreamItem::Value(Sequenced::with_sequence(30, 3)))
        .unwrap();

    let item3 = result.next().await.unwrap();
    assert!(matches!(item3, StreamItem::Value(_)));

    tx.send(StreamItem::Value(Sequenced::with_sequence(40, 4)))
        .unwrap();

    let item4 = result.next().await.unwrap();
    assert!(matches!(item4, StreamItem::Value(_)));

    drop(tx);
}

#[tokio::test]
async fn test_combine_with_previous_single_item_stream_with_error() {
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = stream.combine_with_previous();

    // Error first
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))
        .unwrap();

    let item1 = result.next().await.unwrap();
    assert!(matches!(item1, StreamItem::Error(_)));

    drop(tx);
}
