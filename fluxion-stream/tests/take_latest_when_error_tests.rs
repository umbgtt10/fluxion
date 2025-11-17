// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `take_latest_when` operator.

use fluxion_core::StreamItem;
use fluxion_stream::TakeLatestWhenExt;
use fluxion_test_utils::{sequenced::Sequenced, ErrorInjectingStream};
use futures::{stream, StreamExt};

#[tokio::test]
async fn test_take_latest_when_propagates_source_error() {
    let source = vec![
        Sequenced::with_sequence(1, 1),
        Sequenced::with_sequence(2, 2),
        Sequenced::with_sequence(3, 3),
    ];
    let trigger = vec![
        Sequenced::with_sequence(10, 4),
        Sequenced::with_sequence(20, 5),
    ];

    // Error at position 1 in source
    let source_stream = ErrorInjectingStream::new(stream::iter(source), 1);
    let trigger_stream = stream::iter(trigger).map(StreamItem::Value);

    let mut result = source_stream.take_latest_when(trigger_stream, |_| true);

    // Should get error when it appears in source
    let mut has_error = false;
    let mut has_value = false;

    while let Some(item) = result.next().await {
        match item {
            StreamItem::Error(_) => has_error = true,
            StreamItem::Value(_) => has_value = true,
        }
    }

    assert!(has_error, "Should propagate source error");
    assert!(has_value, "Should also emit values");
}

#[tokio::test]
async fn test_take_latest_when_propagates_trigger_error() {
    let source = vec![
        Sequenced::with_sequence(1, 1),
        Sequenced::with_sequence(2, 2),
    ];
    let trigger = vec![
        Sequenced::with_sequence(10, 3),
        Sequenced::with_sequence(20, 4),
        Sequenced::with_sequence(30, 5),
    ];

    let source_stream = stream::iter(source).map(StreamItem::Value);
    // Error at position 1 in trigger
    let trigger_stream = ErrorInjectingStream::new(stream::iter(trigger), 1);

    let mut result = source_stream.take_latest_when(trigger_stream, |_| true);

    let mut error_count = 0;
    let mut value_count = 0;

    while let Some(item) = result.next().await {
        match item {
            StreamItem::Error(_) => error_count += 1,
            StreamItem::Value(_) => value_count += 1,
        }
    }

    assert!(error_count >= 1, "Should propagate trigger error");
    assert!(value_count >= 1, "Should emit values");
}

#[tokio::test]
async fn test_take_latest_when_filter_predicate_after_error() {
    let source = vec![
        Sequenced::with_sequence(1, 1),
        Sequenced::with_sequence(2, 2),
        Sequenced::with_sequence(3, 3),
    ];
    let trigger = vec![
        Sequenced::with_sequence(10, 4),
        Sequenced::with_sequence(100, 5),
    ];

    // Error in source
    let source_stream = ErrorInjectingStream::new(stream::iter(source), 1);
    let trigger_stream = stream::iter(trigger).map(StreamItem::Value);

    // Only emit when trigger is > 50
    let mut result = source_stream.take_latest_when(trigger_stream, |t| *t > 50);

    let items: Vec<_> = result.collect().await;

    let has_error = items.iter().any(|i| matches!(i, StreamItem::Error(_)));
    let has_value = items.iter().any(|i| matches!(i, StreamItem::Value(_)));

    assert!(has_error, "Should have error");
    assert!(has_value, "Predicate should still work after error");
}

#[tokio::test]
async fn test_take_latest_when_both_streams_have_errors() {
    let source = vec![
        Sequenced::with_sequence(1, 1),
        Sequenced::with_sequence(2, 3),
    ];
    let trigger = vec![
        Sequenced::with_sequence(10, 2),
        Sequenced::with_sequence(20, 4),
    ];

    // Both streams have errors
    let source_stream = ErrorInjectingStream::new(stream::iter(source), 1);
    let trigger_stream = ErrorInjectingStream::new(stream::iter(trigger), 1);

    let mut result = source_stream.take_latest_when(trigger_stream, |_| true);

    let mut error_count = 0;

    while let Some(item) = result.next().await {
        if matches!(item, StreamItem::Error(_)) {
            error_count += 1;
        }
    }

    assert!(
        error_count >= 2,
        "Should propagate errors from both streams"
    );
}
