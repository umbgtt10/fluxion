// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for composed stream operations.

use fluxion_core::{Ordered, StreamItem};
use fluxion_stream::{CombineLatestExt, EmitWhenExt, FluxionStream, TakeLatestWhenExt};
use fluxion_test_utils::{sequenced::Sequenced, ErrorInjectingStream};
use futures::{stream, StreamExt};

#[tokio::test]
async fn test_error_propagation_through_multiple_operators() {
    // Create a stream with error injection
    let items = vec![
        Sequenced::with_sequence(1, 1),
        Sequenced::with_sequence(2, 2),
        Sequenced::with_sequence(3, 3),
        Sequenced::with_sequence(4, 4),
        Sequenced::with_sequence(5, 5),
    ];

    // Inject error at position 2
    let stream = ErrorInjectingStream::new(stream::iter(items), 2);

    // Chain multiple operators
    let result = FluxionStream::new(stream)
        .filter_ordered(|x| *x > 1) // Filter out first item
        .combine_with_previous()
        .map_ordered(|x| x.current.get() * 10);

    let items: Vec<_> = result.collect().await;

    // Should have error and transformed values
    let has_error = items.iter().any(|i| matches!(i, StreamItem::Error(_)));
    let has_values = items.iter().any(|i| matches!(i, StreamItem::Value(_)));

    assert!(has_error, "Error should propagate through operator chain");
    assert!(has_values, "Values should be transformed correctly");

    // Check that transformations work before and after error
    let values: Vec<_> = items
        .iter()
        .filter_map(|item| match item {
            StreamItem::Value(v) => Some(*v),
            _ => None,
        })
        .collect();

    assert!(!values.is_empty(), "Should have transformed values");
    // Values should be multiplied by 10
    assert!(
        values.iter().all(|&v| v % 10 == 0),
        "All values should be multiples of 10"
    );
}

#[tokio::test]
async fn test_error_in_long_operator_chain() {
    let items = vec![
        Sequenced::with_sequence(10, 1),
        Sequenced::with_sequence(20, 2),
        Sequenced::with_sequence(30, 3),
        Sequenced::with_sequence(40, 4),
    ];

    // Inject error at position 1
    let stream = ErrorInjectingStream::new(stream::iter(items), 1);

    // Long chain: filter -> combine_with_previous -> map
    let result = FluxionStream::new(stream)
        .filter_ordered(|x| *x >= 10)
        .combine_with_previous()
        .map_ordered(|x| x.current.get() + 5);

    let items: Vec<_> = result.collect().await;

    let error_count = items
        .iter()
        .filter(|i| matches!(i, StreamItem::Error(_)))
        .count();
    let value_count = items
        .iter()
        .filter(|i| matches!(i, StreamItem::Value(_)))
        .count();

    assert_eq!(error_count, 1, "Should have exactly one error");
    assert!(
        value_count >= 1,
        "Should have transformed and filtered values"
    );
}

#[tokio::test]
async fn test_multiple_errors_through_composition() {
    let items1 = vec![
        Sequenced::with_sequence(1, 1),
        Sequenced::with_sequence(2, 2),
        Sequenced::with_sequence(3, 3),
    ];
    let items2 = vec![
        Sequenced::with_sequence(10, 4),
        Sequenced::with_sequence(20, 5),
    ];

    // Inject errors in primary stream
    let stream1 = ErrorInjectingStream::new(stream::iter(items1), 1);
    let stream2 = stream::iter(items2).map(StreamItem::Value);

    // Combine then transform
    let result = stream1
        .combine_latest(vec![stream2], |_| true)
        .combine_with_previous()
        .map_ordered(|x| format!("Combined: {:?}", x.current.get().values()));

    let items: Vec<_> = result.collect().await;

    let has_error = items.iter().any(|i| matches!(i, StreamItem::Error(_)));
    let has_string_values = items
        .iter()
        .any(|i| matches!(i, StreamItem::Value(s) if s.contains("Combined")));

    assert!(
        has_error,
        "Should propagate errors through combined operators"
    );
    assert!(has_string_values, "Should format combined values correctly");
}

#[tokio::test]
async fn test_error_recovery_in_composed_streams() {
    let source = vec![
        Sequenced::with_sequence(5, 1),
        Sequenced::with_sequence(10, 2),
        Sequenced::with_sequence(15, 3),
        Sequenced::with_sequence(20, 4),
    ];
    let trigger = vec![
        Sequenced::with_sequence(100, 5),
        Sequenced::with_sequence(200, 6),
    ];

    // Error in source
    let source_stream = ErrorInjectingStream::new(stream::iter(source), 1);
    let trigger_stream = stream::iter(trigger).map(StreamItem::Value);

    // Complex composition
    let result = source_stream
        .take_latest_when(trigger_stream, |_| true)
        .combine_with_previous()
        .map_ordered(|x| *x.current.get());

    let items: Vec<_> = result.collect().await;

    // Stream should complete and process values despite error
    let has_error = items.iter().any(|i| matches!(i, StreamItem::Error(_)));

    assert!(
        has_error || items.is_empty(),
        "Should handle error gracefully"
    );
}

#[tokio::test]
async fn test_error_with_emit_when_composition() {
    let source = vec![
        Sequenced::with_sequence(5, 1),
        Sequenced::with_sequence(15, 2),
        Sequenced::with_sequence(25, 3),
    ];
    let filter = vec![
        Sequenced::with_sequence(10, 4),
        Sequenced::with_sequence(20, 5),
    ];

    let source_stream = ErrorInjectingStream::new(stream::iter(source), 1);
    let filter_stream = stream::iter(filter).map(StreamItem::Value);

    // emit_when + map composition
    let result = source_stream
        .emit_when(filter_stream, |state| state.values()[0] > state.values()[1])
        .combine_with_previous()
        .map_ordered(|x| x.current.get() * 2);

    let items: Vec<_> = result.collect().await;

    let has_error = items.iter().any(|i| matches!(i, StreamItem::Error(_)));

    assert!(
        has_error,
        "Should propagate error through emit_when composition"
    );
}
