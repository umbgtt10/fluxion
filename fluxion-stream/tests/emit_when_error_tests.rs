// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `emit_when` operator.

use fluxion_core::StreamItem;
use fluxion_stream::{CombinedState, EmitWhenExt};
use fluxion_test_utils::{sequenced::Sequenced, ErrorInjectingStream};
use futures::{stream, StreamExt};

#[tokio::test]
async fn test_emit_when_propagates_source_error() {
    let source = vec![
        Sequenced::with_sequence(10, 1),
        Sequenced::with_sequence(20, 2),
        Sequenced::with_sequence(30, 3),
    ];
    let filter = vec![Sequenced::with_sequence(15, 4)];

    // Error at position 1 in source
    let source_stream = ErrorInjectingStream::new(stream::iter(source), 1);
    let filter_stream = stream::iter(filter).map(StreamItem::Value);

    let mut result =
        source_stream.emit_when(filter_stream, |state| state.values()[0] > state.values()[1]);

    let mut has_error = false;
    let mut has_value = false;

    while let Some(item) = result.next().await {
        match item {
            StreamItem::Error(_) => has_error = true,
            StreamItem::Value(_) => has_value = true,
        }
    }

    assert!(has_error, "Should propagate source error");
    assert!(has_value, "Should emit values");
}

#[tokio::test]
async fn test_emit_when_propagates_filter_error() {
    let source = vec![
        Sequenced::with_sequence(10, 1),
        Sequenced::with_sequence(20, 3),
    ];
    let filter = vec![
        Sequenced::with_sequence(5, 2),
        Sequenced::with_sequence(15, 4),
    ];

    let source_stream = stream::iter(source).map(StreamItem::Value);
    // Error in filter
    let filter_stream = ErrorInjectingStream::new(stream::iter(filter), 1);

    let mut result =
        source_stream.emit_when(filter_stream, |state| state.values()[0] > state.values()[1]);

    let mut error_count = 0;
    let mut value_count = 0;

    while let Some(item) = result.next().await {
        match item {
            StreamItem::Error(_) => error_count += 1,
            StreamItem::Value(_) => value_count += 1,
        }
    }

    assert!(error_count >= 1, "Should propagate filter error");
    assert!(value_count >= 1, "Should emit values");
}

#[tokio::test]
async fn test_emit_when_predicate_continues_after_error() {
    let source = vec![
        Sequenced::with_sequence(10, 1),
        Sequenced::with_sequence(20, 3),
        Sequenced::with_sequence(30, 5),
    ];
    let filter = vec![
        Sequenced::with_sequence(15, 2),
        Sequenced::with_sequence(25, 4),
    ];

    // Error in source
    let source_stream = ErrorInjectingStream::new(stream::iter(source), 1);
    let filter_stream = stream::iter(filter).map(StreamItem::Value);

    // Only emit when source > filter
    let mut result =
        source_stream.emit_when(filter_stream, |state| state.values()[0] > state.values()[1]);

    let items: Vec<_> = result.collect().await;

    let has_error = items.iter().any(|i| matches!(i, StreamItem::Error(_)));
    let has_value = items.iter().any(|i| matches!(i, StreamItem::Value(_)));

    assert!(has_error, "Should have error");
    assert!(has_value, "Predicate should still evaluate after error");
}

#[tokio::test]
async fn test_emit_when_both_streams_have_errors() {
    let source = vec![
        Sequenced::with_sequence(10, 1),
        Sequenced::with_sequence(20, 3),
    ];
    let filter = vec![
        Sequenced::with_sequence(5, 2),
        Sequenced::with_sequence(15, 4),
    ];

    // Both have errors
    let source_stream = ErrorInjectingStream::new(stream::iter(source), 1);
    let filter_stream = ErrorInjectingStream::new(stream::iter(filter), 1);

    let mut result =
        source_stream.emit_when(filter_stream, |state| state.values()[0] > state.values()[1]);

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

#[tokio::test]
async fn test_emit_when_error_before_filter_ready() {
    let source = vec![Sequenced::with_sequence(10, 1)];
    let filter = vec![Sequenced::with_sequence(5, 2)];

    // Error immediately before filter has value
    let source_stream = ErrorInjectingStream::new(stream::iter(source), 0);
    let filter_stream = stream::iter(filter).map(StreamItem::Value);

    let mut result =
        source_stream.emit_when(filter_stream, |state| state.values()[0] > state.values()[1]);

    let first = result.next().await.unwrap();
    assert!(matches!(first, StreamItem::Error(_)));
}
