// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `filter_ordered` operator.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::FluxionStream;
use fluxion_test_utils::{sequenced::Sequenced, test_channel_with_errors};
use futures::StreamExt;

#[tokio::test]
async fn test_filter_ordered_propagates_errors() {
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = FluxionStream::new(stream).filter_ordered(|x| x % 2 == 0);

    // First item (1) filtered out
    tx.send(StreamItem::Value(Sequenced::with_sequence(1, 1)))
        .unwrap();

    // Error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))
        .unwrap();

    let item1 = result.next().await.unwrap();
    assert!(matches!(item1, StreamItem::Error(_)));

    // Value that passes filter
    tx.send(StreamItem::Value(Sequenced::with_sequence(2, 2)))
        .unwrap();

    let item2 = result.next().await.unwrap();
    assert!(matches!(item2, StreamItem::Value(ref v) if *v.get() == 2));

    // Value filtered out
    tx.send(StreamItem::Value(Sequenced::with_sequence(3, 3)))
        .unwrap();

    // Value that passes
    tx.send(StreamItem::Value(Sequenced::with_sequence(4, 4)))
        .unwrap();

    let item3 = result.next().await.unwrap();
    assert!(matches!(item3, StreamItem::Value(ref v) if *v.get() == 4));

    drop(tx);
}

#[tokio::test]
async fn test_filter_ordered_predicate_after_error() {
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Only pass values > 18
    let mut result = FluxionStream::new(stream).filter_ordered(|x| *x > 18);

    // Values that don't pass
    tx.send(StreamItem::Value(Sequenced::with_sequence(10, 1)))
        .unwrap();
    tx.send(StreamItem::Value(Sequenced::with_sequence(15, 2)))
        .unwrap();

    // Error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))
        .unwrap();

    let item1 = result.next().await.unwrap();
    assert!(matches!(item1, StreamItem::Error(_)));

    // Values that pass filter
    tx.send(StreamItem::Value(Sequenced::with_sequence(20, 3)))
        .unwrap();

    let item2 = result.next().await.unwrap();
    assert!(matches!(item2, StreamItem::Value(ref v) if *v.get() == 20));

    tx.send(StreamItem::Value(Sequenced::with_sequence(25, 4)))
        .unwrap();

    let item3 = result.next().await.unwrap();
    assert!(matches!(item3, StreamItem::Value(ref v) if *v.get() == 25));

    tx.send(StreamItem::Value(Sequenced::with_sequence(30, 5)))
        .unwrap();

    let item4 = result.next().await.unwrap();
    assert!(matches!(item4, StreamItem::Value(ref v) if *v.get() == 30));

    drop(tx);
}

#[tokio::test]
async fn test_filter_ordered_error_at_start() {
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = FluxionStream::new(stream).filter_ordered(|x| *x > 1);

    // Error first
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))
        .unwrap();

    let first = result.next().await.unwrap();
    assert!(matches!(first, StreamItem::Error(_)));

    // Filtered values
    tx.send(StreamItem::Value(Sequenced::with_sequence(2, 2)))
        .unwrap();

    let second = result.next().await.unwrap();
    assert!(matches!(second, StreamItem::Value(ref v) if *v.get() == 2));

    tx.send(StreamItem::Value(Sequenced::with_sequence(3, 3)))
        .unwrap();

    let third = result.next().await.unwrap();
    assert!(matches!(third, StreamItem::Value(ref v) if *v.get() == 3));

    drop(tx);
}

#[tokio::test]
async fn test_filter_ordered_all_filtered_except_error() {
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Filter out all odd numbers (but error passes through)
    let mut result = FluxionStream::new(stream).filter_ordered(|x| x % 2 == 0);

    // Send odd number (filtered)
    tx.send(StreamItem::Value(Sequenced::with_sequence(1, 1)))
        .unwrap();

    // Error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))
        .unwrap();

    let first = result.next().await.unwrap();
    assert!(matches!(first, StreamItem::Error(_)));

    // More odd numbers (filtered)
    tx.send(StreamItem::Value(Sequenced::with_sequence(5, 3)))
        .unwrap();

    // Close stream
    drop(tx);

    // No more items (all filtered)
    assert!(result.next().await.is_none());
}

#[tokio::test]
async fn test_filter_ordered_chain_with_map_after_error() {
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    // Chain filter and map
    let mut result = FluxionStream::new(stream)
        .filter_ordered(|x| *x >= 20)
        .combine_with_previous()
        .map_ordered(|x| x.current.get() / 10);

    // Send values
    tx.send(StreamItem::Value(Sequenced::with_sequence(10, 1)))
        .unwrap(); // Filtered

    // Error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))
        .unwrap();

    let item1 = result.next().await.unwrap();
    assert!(matches!(item1, StreamItem::Error(_)));

    // Values that pass filter
    tx.send(StreamItem::Value(Sequenced::with_sequence(20, 2)))
        .unwrap();

    let item2 = result.next().await.unwrap();
    assert!(matches!(item2, StreamItem::Value(2)));

    tx.send(StreamItem::Value(Sequenced::with_sequence(30, 3)))
        .unwrap();

    let item3 = result.next().await.unwrap();
    assert!(matches!(item3, StreamItem::Value(3)));

    tx.send(StreamItem::Value(Sequenced::with_sequence(40, 4)))
        .unwrap();

    let item4 = result.next().await.unwrap();
    assert!(matches!(item4, StreamItem::Value(4)));

    drop(tx);
}
