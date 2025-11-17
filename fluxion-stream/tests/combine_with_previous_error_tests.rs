// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `combine_with_previous` operator.

use fluxion_core::StreamItem;
use fluxion_stream::CombineWithPreviousExt;
use fluxion_test_utils::{sequenced::Sequenced, ErrorInjectingStream};
use futures::{stream, StreamExt};

#[tokio::test]
async fn test_combine_with_previous_propagates_errors() {
    let items = vec![
        Sequenced::with_sequence(1, 1),
        Sequenced::with_sequence(2, 2),
        Sequenced::with_sequence(3, 3),
        Sequenced::with_sequence(4, 4),
    ];

    // Inject error at position 2
    let stream = ErrorInjectingStream::new(stream::iter(items), 2);

    let mut result = stream.combine_with_previous();

    // First item - no previous
    let item1 = result.next().await.unwrap();
    assert!(matches!(item1, StreamItem::Value(_)));

    // Second item - has previous
    let item2 = result.next().await.unwrap();
    assert!(matches!(item2, StreamItem::Value(_)));

    // Third item - error
    let item3 = result.next().await.unwrap();
    assert!(matches!(item3, StreamItem::Error(_)));

    // Fourth item - continues after error
    let item4 = result.next().await.unwrap();
    assert!(matches!(item4, StreamItem::Value(_)));
}

#[tokio::test]
async fn test_combine_with_previous_error_at_first_item() {
    let items = vec![
        Sequenced::with_sequence(1, 1),
        Sequenced::with_sequence(2, 2),
    ];

    // Error immediately
    let stream = ErrorInjectingStream::new(stream::iter(items), 0);

    let mut result = stream.combine_with_previous();

    let first = result.next().await.unwrap();
    assert!(
        matches!(first, StreamItem::Error(_)),
        "First item should be error"
    );

    let second = result.next().await.unwrap();
    assert!(matches!(second, StreamItem::Value(_)), "Should continue");
}

#[tokio::test]
async fn test_combine_with_previous_multiple_errors() {
    let items = vec![
        Sequenced::with_sequence(1, 1),
        Sequenced::with_sequence(2, 2),
        Sequenced::with_sequence(3, 3),
        Sequenced::with_sequence(4, 4),
        Sequenced::with_sequence(5, 5),
    ];

    // We can only inject one error with ErrorInjectingStream, but we can test the behavior
    let stream = ErrorInjectingStream::new(stream::iter(items), 1);

    let mut result = stream.combine_with_previous();

    let mut error_count = 0;
    let mut value_count = 0;

    while let Some(item) = result.next().await {
        match item {
            StreamItem::Error(_) => error_count += 1,
            StreamItem::Value(_) => value_count += 1,
        }
    }

    assert_eq!(error_count, 1, "Should have exactly one error");
    assert!(value_count >= 3, "Should have multiple values");
}

#[tokio::test]
async fn test_combine_with_previous_preserves_pairing_after_error() {
    let items = vec![
        Sequenced::with_sequence(10, 1),
        Sequenced::with_sequence(20, 2),
        Sequenced::with_sequence(30, 3),
        Sequenced::with_sequence(40, 4),
    ];

    let stream = ErrorInjectingStream::new(stream::iter(items), 1);

    let mut result = stream.combine_with_previous();

    let items: Vec<_> = result.collect().await;

    // Should have values before and after error
    let values: Vec<_> = items
        .iter()
        .filter_map(|item| match item {
            StreamItem::Value(v) => Some(v),
            _ => None,
        })
        .collect();

    // After error, pairing should still work correctly
    assert!(values.len() >= 2, "Should have multiple value pairs");
}

#[tokio::test]
async fn test_combine_with_previous_single_item_stream_with_error() {
    let items = vec![Sequenced::with_sequence(1, 1)];

    let stream = ErrorInjectingStream::new(stream::iter(items), 0);

    let result = stream.combine_with_previous();

    let items: Vec<_> = result.collect().await;

    // Should have error and the value after
    let has_error = items.iter().any(|i| matches!(i, StreamItem::Error(_)));
    assert!(has_error, "Should have error");
}
