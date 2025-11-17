// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `map_ordered` operator.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::FluxionStream;
use fluxion_test_utils::{sequenced::Sequenced, test_channel_with_errors};
use futures::StreamExt;

#[tokio::test]
async fn test_map_ordered_propagates_errors() {
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Use combine_with_previous then map to string
    let mut result = FluxionStream::new(stream)
        .combine_with_previous()
        .map_ordered(|x| format!("Current: {}", x.current.get()));

    // Send value
    tx.send(StreamItem::Value(Sequenced::new(1))).unwrap();

    let item1 = result.next().await.unwrap();
    assert!(matches!(item1, StreamItem::Value(ref v) if v == "Current: 1"));

    // Send error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))
        .unwrap();

    let item2 = result.next().await.unwrap();
    assert!(matches!(item2, StreamItem::Error(_)));

    // Continue
    tx.send(StreamItem::Value(Sequenced::new(2))).unwrap();

    let item3 = result.next().await.unwrap();
    assert!(matches!(item3, StreamItem::Value(_)));

    drop(tx);
}

#[tokio::test]
async fn test_map_ordered_transformation_after_error() {
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Use combine_with_previous then map
    let mut result = FluxionStream::new(stream)
        .combine_with_previous()
        .map_ordered(|x| x.current.get() * 2);

    // Send values
    tx.send(StreamItem::Value(Sequenced::with_sequence(10, 1)))
        .unwrap();

    let item1 = result.next().await.unwrap();
    assert!(matches!(item1, StreamItem::Value(20)));

    tx.send(StreamItem::Value(Sequenced::with_sequence(20, 2)))
        .unwrap();

    let item2 = result.next().await.unwrap();
    assert!(matches!(item2, StreamItem::Value(40)));

    // Send error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))
        .unwrap();

    let item3 = result.next().await.unwrap();
    assert!(matches!(item3, StreamItem::Error(_)));

    // Continue after error
    tx.send(StreamItem::Value(Sequenced::with_sequence(40, 4)))
        .unwrap();

    let item4 = result.next().await.unwrap();
    assert!(matches!(item4, StreamItem::Value(80)));

    drop(tx);
}

#[tokio::test]
async fn test_map_ordered_preserves_error_passthrough() {
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = FluxionStream::new(stream)
        .combine_with_previous()
        .map_ordered(|x| x.current.get() * 100);

    // Error immediately
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))
        .unwrap();

    let first = result.next().await.unwrap();
    assert!(
        matches!(first, StreamItem::Error(_)),
        "Error should pass through"
    );

    // Continue with value
    tx.send(StreamItem::Value(Sequenced::with_sequence(2, 2)))
        .unwrap();

    let second = result.next().await.unwrap();
    assert!(
        matches!(second, StreamItem::Value(200)),
        "Should have mapped value after error"
    );

    drop(tx);
}

#[tokio::test]
async fn test_map_ordered_chain_after_error() {
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Chain combine_with_previous and map
    let mut result = FluxionStream::new(stream)
        .combine_with_previous()
        .map_ordered(|x| x.current.get() * 2);

    // Send value
    tx.send(StreamItem::Value(Sequenced::with_sequence(5, 1)))
        .unwrap();

    let item1 = result.next().await.unwrap();
    assert!(matches!(item1, StreamItem::Value(10)));

    // Send error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))
        .unwrap();

    let item2 = result.next().await.unwrap();
    assert!(matches!(item2, StreamItem::Error(_)));

    // Continue
    tx.send(StreamItem::Value(Sequenced::with_sequence(15, 3)))
        .unwrap();

    let item3 = result.next().await.unwrap();
    assert!(matches!(item3, StreamItem::Value(30)));

    drop(tx);
}
