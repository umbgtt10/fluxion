// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `with_latest_from` operator.

use fluxion_core::StreamItem;
use fluxion_stream::WithLatestFromExt;
use fluxion_test_utils::{sequenced::Sequenced, ErrorInjectingStream};
use futures::{stream, StreamExt};

#[tokio::test]
async fn test_with_latest_from_propagates_primary_error() {
    let primary = vec![
        Sequenced::with_sequence(1, 1),
        Sequenced::with_sequence(2, 3),
        Sequenced::with_sequence(3, 5),
    ];
    let secondary = vec![
        Sequenced::with_sequence(10, 2),
        Sequenced::with_sequence(20, 4),
    ];

    // Inject error at position 1 in primary
    let primary_stream = ErrorInjectingStream::new(stream::iter(primary), 1);
    let secondary_stream = stream::iter(secondary).map(StreamItem::Value);

    let result = primary_stream.with_latest_from(secondary_stream, |_| true);

    let items: Vec<_> = result.collect().await;

    let has_error = items.iter().any(|i| matches!(i, StreamItem::Error(_)));
    let has_value = items.iter().any(|i| matches!(i, StreamItem::Value(_)));

    assert!(has_error, "Should propagate error from primary stream");
    assert!(has_value, "Should also emit values");
}

#[tokio::test]
async fn test_with_latest_from_propagates_secondary_error() {
    let primary = vec![
        Sequenced::with_sequence(1, 2),
        Sequenced::with_sequence(2, 4),
    ];
    let secondary = vec![
        Sequenced::with_sequence(10, 1),
        Sequenced::with_sequence(20, 3),
        Sequenced::with_sequence(30, 5),
    ];

    let primary_stream = stream::iter(primary).map(StreamItem::Value);
    // Inject error in secondary
    let secondary_stream = ErrorInjectingStream::new(stream::iter(secondary), 1);

    let mut result = primary_stream.with_latest_from(secondary_stream, |_| true);

    // Should get error when secondary emits it
    let mut has_error = false;
    let mut has_value = false;

    while let Some(item) = result.next().await {
        match item {
            StreamItem::Error(_) => has_error = true,
            StreamItem::Value(_) => has_value = true,
        }
    }

    assert!(has_error, "Should propagate error from secondary");
    assert!(has_value, "Should have values");
}

#[tokio::test]
async fn test_with_latest_from_error_before_secondary_ready() {
    let primary = vec![Sequenced::with_sequence(1, 1)];
    let secondary = vec![Sequenced::with_sequence(10, 2)];

    // Error immediately in primary, before secondary has value
    let primary_stream = ErrorInjectingStream::new(stream::iter(primary), 0);
    let secondary_stream = stream::iter(secondary).map(StreamItem::Value);

    let mut result = primary_stream.with_latest_from(secondary_stream, |_| true);

    let item = result.next().await.unwrap();
    assert!(matches!(item, StreamItem::Error(_)));
}

#[tokio::test]
async fn test_with_latest_from_selector_continues_after_error() {
    let primary = vec![
        Sequenced::with_sequence(1, 1),
        Sequenced::with_sequence(2, 3),
        Sequenced::with_sequence(3, 5),
    ];
    let secondary = vec![
        Sequenced::with_sequence(100, 2),
        Sequenced::with_sequence(200, 4),
    ];

    let primary_stream = ErrorInjectingStream::new(stream::iter(primary), 1);
    let secondary_stream = stream::iter(secondary).map(StreamItem::Value);

    // Custom selector - pass through combined state
    let mut result = primary_stream.with_latest_from(secondary_stream, |combined| combined.clone());

    let mut items = vec![];
    while let Some(item) = result.next().await {
        items.push(item);
    }

    // Should have error and some values
    let error_count = items
        .iter()
        .filter(|i| matches!(i, StreamItem::Error(_)))
        .count();
    let value_count = items
        .iter()
        .filter(|i| matches!(i, StreamItem::Value(_)))
        .count();

    assert_eq!(error_count, 1, "Should have exactly one error");
    assert!(value_count >= 1, "Should have at least one value");
}
