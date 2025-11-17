// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `filter_ordered` operator.

use fluxion_core::StreamItem;
use fluxion_stream::FluxionStream;
use fluxion_test_utils::{sequenced::Sequenced, ErrorInjectingStream};
use futures::{stream, StreamExt};

#[tokio::test]
async fn test_filter_ordered_propagates_errors() {
    let items = vec![
        Sequenced::with_sequence(1, 1),
        Sequenced::with_sequence(2, 2),
        Sequenced::with_sequence(3, 3),
        Sequenced::with_sequence(4, 4),
    ];

    // Inject error at position 1
    let stream = ErrorInjectingStream::new(stream::iter(items), 1);

    let mut result = FluxionStream::new(stream).filter_ordered(|x| x % 2 == 0);

    // First item (1) filtered out
    // Second item is error (injected)
    let item1 = result.next().await.unwrap();
    assert!(matches!(item1, StreamItem::Error(_)));

    // Third item (actual value 2) passes
    let item2 = result.next().await.unwrap();
    match item2 {
        StreamItem::Value(v) => assert_eq!(*v.get(), 2),
        _ => panic!("Expected value"),
    }

    // Fourth item (3) filtered out
    // Fifth item (4) passes
    let item3 = result.next().await.unwrap();
    match item3 {
        StreamItem::Value(v) => assert_eq!(*v.get(), 4),
        _ => panic!("Expected value"),
    }
}

#[tokio::test]
async fn test_filter_ordered_predicate_after_error() {
    let items = vec![
        Sequenced::with_sequence(10, 1),
        Sequenced::with_sequence(15, 2),
        Sequenced::with_sequence(20, 3),
        Sequenced::with_sequence(25, 4),
        Sequenced::with_sequence(30, 5),
    ];

    let stream = ErrorInjectingStream::new(stream::iter(items), 2);

    // Only pass values > 18
    let mut result = FluxionStream::new(stream).filter_ordered(|x| *x > 18);

    let items: Vec<_> = result.collect().await;

    let error_count = items
        .iter()
        .filter(|i| matches!(i, StreamItem::Error(_)))
        .count();
    let values: Vec<_> = items
        .iter()
        .filter_map(|item| match item {
            StreamItem::Value(v) => Some(*v.get()),
            _ => None,
        })
        .collect();

    assert_eq!(error_count, 1, "Should have one error");
    assert_eq!(
        values,
        vec![20, 25, 30],
        "Predicate should work after error"
    );
}

#[tokio::test]
async fn test_filter_ordered_error_at_start() {
    let items = vec![
        Sequenced::with_sequence(1, 1),
        Sequenced::with_sequence(2, 2),
        Sequenced::with_sequence(3, 3),
    ];

    let stream = ErrorInjectingStream::new(stream::iter(items), 0);

    let mut result = FluxionStream::new(stream).filter_ordered(|x| *x > 1);

    // First is error
    let first = result.next().await.unwrap();
    assert!(matches!(first, StreamItem::Error(_)));

    // Then filtered values
    let second = result.next().await.unwrap();
    match second {
        StreamItem::Value(v) => assert_eq!(*v.get(), 2),
        _ => panic!("Expected value"),
    }

    let third = result.next().await.unwrap();
    match third {
        StreamItem::Value(v) => assert_eq!(*v.get(), 3),
        _ => panic!("Expected value"),
    }
}

#[tokio::test]
async fn test_filter_ordered_all_filtered_except_error() {
    let items = vec![
        Sequenced::with_sequence(1, 1),
        Sequenced::with_sequence(3, 2),
        Sequenced::with_sequence(5, 3),
    ];

    let stream = ErrorInjectingStream::new(stream::iter(items), 1);

    // Filter out all odd numbers (but error passes through)
    let mut result = FluxionStream::new(stream).filter_ordered(|x| x % 2 == 0);

    // Only the error should come through (all values are odd)
    let first = result.next().await.unwrap();
    assert!(matches!(first, StreamItem::Error(_)));

    // No more items
    assert!(result.next().await.is_none());
}

#[tokio::test]
async fn test_filter_ordered_chain_with_map_after_error() {
    let items = vec![
        Sequenced::with_sequence(10, 1),
        Sequenced::with_sequence(20, 2),
        Sequenced::with_sequence(30, 3),
        Sequenced::with_sequence(40, 4),
    ];

    let stream = ErrorInjectingStream::new(stream::iter(items), 1);

    // Chain filter and map
    let result = FluxionStream::new(stream)
        .filter_ordered(|x| *x >= 20)
        .combine_with_previous()
        .map_ordered(|x| x.current.get() / 10);

    let items: Vec<_> = result.collect().await;

    let has_error = items.iter().any(|i| matches!(i, StreamItem::Error(_)));
    let values: Vec<_> = items
        .iter()
        .filter_map(|item| match item {
            StreamItem::Value(v) => Some(*v),
            _ => None,
        })
        .collect();

    assert!(has_error, "Should have error");
    assert_eq!(
        values,
        vec![2, 3, 4],
        "Filter and map should work correctly"
    );
}
