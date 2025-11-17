// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `combine_latest` operator.

use fluxion_core::StreamItem;
use fluxion_stream::CombineLatestExt;
use fluxion_test_utils::{sequenced::Sequenced, ErrorInjectingStream};
use futures::{stream, StreamExt};

#[tokio::test]
async fn test_combine_latest_propagates_error_from_primary_stream() {
    let items1 = vec![
        Sequenced::with_sequence(1, 1),
        Sequenced::with_sequence(2, 3),
        Sequenced::with_sequence(3, 5),
    ];
    let items2 = vec![
        Sequenced::with_sequence(10, 2),
        Sequenced::with_sequence(20, 4),
    ];

    // Inject error at position 1 in primary stream
    let stream1 = ErrorInjectingStream::new(stream::iter(items1), 1);
    let stream2 = stream::iter(items2).map(StreamItem::Value);

    let mut combined = stream1.combine_latest(vec![stream2], |_| true);

    // Collect all items
    let items: Vec<_> = combined.collect().await;

    let has_error = items.iter().any(|i| matches!(i, StreamItem::Error(_)));
    let has_value = items.iter().any(|i| matches!(i, StreamItem::Value(_)));

    assert!(has_error, "Should propagate error from primary stream");
    assert!(has_value, "Should also emit values");
}

#[tokio::test]
async fn test_combine_latest_propagates_error_from_secondary_stream() {
    let items1 = vec![
        Sequenced::with_sequence(1, 1),
        Sequenced::with_sequence(2, 2),
    ];
    let items2 = vec![
        Sequenced::with_sequence(10, 3),
        Sequenced::with_sequence(20, 4),
        Sequenced::with_sequence(30, 5),
    ];

    let stream1 = stream::iter(items1).map(StreamItem::Value);
    // Inject error at position 1 in secondary stream
    let stream2 = ErrorInjectingStream::new(stream::iter(items2), 1);

    let mut combined = stream1.combine_latest(vec![stream2], |_| true);

    // First emission should succeed
    let result1 = combined.next().await.unwrap();
    assert!(matches!(result1, StreamItem::Value(_)));

    // Second should be the error from secondary
    let result2 = combined.next().await.unwrap();
    assert!(matches!(result2, StreamItem::Error(_)));

    // Stream should continue after error
    let result3 = combined.next().await.unwrap();
    assert!(matches!(result3, StreamItem::Value(_)));
}

#[tokio::test]
async fn test_combine_latest_multiple_errors_from_different_streams() {
    let items1 = vec![
        Sequenced::with_sequence(1, 1),
        Sequenced::with_sequence(2, 3),
    ];
    let items2 = vec![
        Sequenced::with_sequence(10, 2),
        Sequenced::with_sequence(20, 4),
    ];

    // Both streams inject errors
    let stream1 = ErrorInjectingStream::new(stream::iter(items1), 1);
    let stream2 = ErrorInjectingStream::new(stream::iter(items2), 1);

    let mut combined = stream1.combine_latest(vec![stream2], |_| true);

    let mut error_count = 0;
    let mut value_count = 0;

    while let Some(item) = combined.next().await {
        match item {
            StreamItem::Value(_) => value_count += 1,
            StreamItem::Error(_) => error_count += 1,
        }
    }

    // Should see errors from both streams
    assert!(
        error_count >= 2,
        "Expected at least 2 errors, got {}",
        error_count
    );
    assert!(
        value_count >= 1,
        "Expected at least 1 value, got {}",
        value_count
    );
}

#[tokio::test]
async fn test_combine_latest_error_at_start() {
    let items1 = vec![
        Sequenced::with_sequence(1, 1),
        Sequenced::with_sequence(2, 2),
    ];
    let items2 = vec![Sequenced::with_sequence(10, 3)];

    // Inject error immediately
    let stream1 = ErrorInjectingStream::new(stream::iter(items1), 0);
    let stream2 = stream::iter(items2).map(StreamItem::Value);

    let mut combined = stream1.combine_latest(vec![stream2], |_| true);

    // First item should be the error
    let result1 = combined.next().await.unwrap();
    assert!(matches!(result1, StreamItem::Error(_)));

    // Should continue processing
    let result2 = combined.next().await.unwrap();
    assert!(matches!(result2, StreamItem::Value(_)));
}

#[tokio::test]
async fn test_combine_latest_filter_predicate_continues_after_error() {
    let items1 = vec![
        Sequenced::with_sequence(1, 1),
        Sequenced::with_sequence(2, 2),
        Sequenced::with_sequence(3, 3),
    ];
    let items2 = vec![Sequenced::with_sequence(10, 4)];

    let stream1 = ErrorInjectingStream::new(stream::iter(items1), 1);
    let stream2 = stream::iter(items2).map(StreamItem::Value);

    // Filter predicate should still be evaluated after error
    let mut combined = stream1.combine_latest(vec![stream2], |state| {
        state.values()[0] > 1 // Only pass values > 1
    });

    let mut results = vec![];
    while let Some(item) = combined.next().await {
        results.push(item);
    }

    // Should have error and filtered values
    let has_error = results
        .iter()
        .any(|item| matches!(item, StreamItem::Error(_)));
    let has_value = results
        .iter()
        .any(|item| matches!(item, StreamItem::Value(_)));

    assert!(has_error, "Should have at least one error");
    assert!(has_value, "Should have at least one value after filtering");
}
