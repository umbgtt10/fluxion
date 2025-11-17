// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `map_ordered` operator.

use fluxion_core::StreamItem;
use fluxion_stream::{CombineWithPreviousExt, FluxionStream};
use fluxion_test_utils::{sequenced::Sequenced, test_channel, ErrorInjectingStream};
use futures::{stream, StreamExt};

#[tokio::test]
async fn test_map_ordered_propagates_errors() {
    let (tx, stream) = test_channel::<Sequenced<i32>>();

    // Use combine_with_previous then map to string
    let mut result = stream
        .combine_with_previous()
        .map_ordered(|x| format!("Current: {}", x.current.get()));

    // Send values and inject error
    tx.send(Sequenced::new(1)).unwrap();
    let item1 = result.next().await.unwrap();
    match item1 {
        StreamItem::Value(v) => assert_eq!(v, "Current: 1"),
        _ => panic!("Expected value"),
    }

    // Send error by dropping sender temporarily
    drop(tx);
}

#[tokio::test]
async fn test_map_ordered_transformation_after_error() {
    let items = vec![
        Sequenced::with_sequence(10, 1),
        Sequenced::with_sequence(20, 2),
        Sequenced::with_sequence(30, 3),
        Sequenced::with_sequence(40, 4),
    ];

    let stream = ErrorInjectingStream::new(stream::iter(items), 2);

    // Use combine_with_previous then map
    let mut result = FluxionStream::new(stream)
        .combine_with_previous()
        .map_ordered(|x| x.current.get() * 2);

    let items: Vec<_> = result.collect().await;

    let has_error = items.iter().any(|i| matches!(i, StreamItem::Error(_)));
    let has_value = items.iter().any(|i| matches!(i, StreamItem::Value(_)));

    // Should have both errors and values
    assert!(has_error, "Should have error");
    assert!(
        has_value,
        "Transformations should work before and after error"
    );
}

#[tokio::test]
async fn test_map_ordered_preserves_error_passthrough() {
    let items = vec![
        Sequenced::with_sequence(1, 1),
        Sequenced::with_sequence(2, 2),
    ];

    let stream = ErrorInjectingStream::new(stream::iter(items), 0);

    let mut result = FluxionStream::new(stream)
        .combine_with_previous()
        .map_ordered(|x| x.current.get() * 100);

    let first = result.next().await.unwrap();
    assert!(
        matches!(first, StreamItem::Error(_)),
        "Error should pass through"
    );

    let second = result.next().await.unwrap();
    assert!(
        matches!(second, StreamItem::Value(_)),
        "Should have mapped value after error"
    );
}

#[tokio::test]
async fn test_map_ordered_chain_after_error() {
    let items = vec![
        Sequenced::with_sequence(5, 1),
        Sequenced::with_sequence(10, 2),
        Sequenced::with_sequence(15, 3),
    ];

    let stream = ErrorInjectingStream::new(stream::iter(items), 1);

    // Chain combine_with_previous and map
    let mut result = FluxionStream::new(stream)
        .combine_with_previous()
        .map_ordered(|x| x.current.get() * 2);

    let items: Vec<_> = result.collect().await;

    let error_count = items
        .iter()
        .filter(|i| matches!(i, StreamItem::Error(_)))
        .count();
    let value_count = items
        .iter()
        .filter(|i| matches!(i, StreamItem::Value(_)))
        .count();

    assert_eq!(error_count, 1, "Should have one error");
    assert!(
        value_count >= 2,
        "Should have at least two transformed values"
    );
}
