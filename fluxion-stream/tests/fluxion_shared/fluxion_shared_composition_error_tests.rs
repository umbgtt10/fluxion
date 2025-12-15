// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Composition error tests for `FluxionShared` with other operators.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::prelude::*;
use fluxion_stream::ShareExt;
use fluxion_test_utils::person::Person;
use fluxion_test_utils::test_data::{person_alice, TestData};
use fluxion_test_utils::{
    assert_stream_ended, test_channel_with_errors, unwrap_stream, unwrap_value, Sequenced,
};

#[tokio::test]
async fn shared_error_propagates_through_chained_operators() -> anyhow::Result<()> {
    // Arrange - source with operators, then share, then more operators per subscriber
    let (tx, rx) = test_channel_with_errors::<Sequenced<TestData>>();

    // Apply map before sharing
    let source = rx.map_ordered(|data| {
        let updated = match data.into_inner() {
            TestData::Person(p) => TestData::Person(Person::new(p.name, p.age + 1)),
            other => other,
        };
        Sequenced::new(updated)
    });

    let shared = source.share();

    // Each subscriber chains further
    let mut sub1 = shared
        .subscribe()
        .unwrap()
        .filter_ordered(|data| matches!(data, TestData::Person(_)))
        .map_ordered(|data| {
            let age = match data.into_inner() {
                TestData::Person(p) => p.age,
                _ => 0,
            };
            Sequenced::new(age)
        });

    let mut sub2 = shared.subscribe().unwrap().combine_with_previous();

    // Act - send value
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;

    // Assert - sub1 receives transformed value (age + 1 = 26)
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut sub1, 500).await)).into_inner(),
        26
    );

    // Assert - sub2 receives value with no previous
    let with_prev = unwrap_value(Some(unwrap_stream(&mut sub2, 500).await));
    assert!(!with_prev.has_previous());

    // Act - send error
    tx.send(StreamItem::Error(FluxionError::stream_error(
        "pipeline failure",
    )))?;

    // Assert - both subscribers receive error
    assert!(matches!(
        unwrap_stream(&mut sub1, 500).await,
        StreamItem::Error(FluxionError::StreamProcessingError { context }) if context == "pipeline failure"
    ));
    assert!(matches!(
        unwrap_stream(&mut sub2, 500).await,
        StreamItem::Error(FluxionError::StreamProcessingError { context }) if context == "pipeline failure"
    ));

    // Assert - both subscribers complete
    assert_stream_ended(&mut sub1, 500).await;
    assert_stream_ended(&mut sub2, 500).await;

    Ok(())
}

#[tokio::test]
async fn shared_with_on_error_allows_selective_handling() -> anyhow::Result<()> {
    // Arrange - use on_error to consume certain errors
    let (tx, rx) = test_channel_with_errors::<Sequenced<TestData>>();
    let source = rx;

    let shared = source.share();

    // Subscriber 1: consumes "transient" errors, propagates others
    let mut tolerant = shared
        .subscribe()
        .unwrap()
        .on_error(|err| err.to_string().contains("transient"));

    // Subscriber 2: no error handling, receives all errors
    let mut strict = shared.subscribe().unwrap();

    // Act - send value, transient error, value, fatal error
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx.send(StreamItem::Error(FluxionError::stream_error(
        "transient network issue",
    )))?;

    // Assert - both receive Alice
    assert!(matches!(
        unwrap_stream(&mut tolerant, 500).await,
        StreamItem::Value(ref v) if matches!(v.value, TestData::Person(ref p) if p.name == "Alice")
    ));
    assert!(matches!(
        unwrap_stream(&mut strict, 500).await,
        StreamItem::Value(ref v) if matches!(v.value, TestData::Person(ref p) if p.name == "Alice")
    ));

    // Assert - tolerant consumes the error (doesn't see it), strict sees error
    // Note: After error, the shared stream closes, so we check strict first
    assert!(matches!(
        unwrap_stream(&mut strict, 500).await,
        StreamItem::Error(_)
    ));

    // Both streams end after the error (shared closes on error)
    assert_stream_ended(&mut tolerant, 500).await;
    assert_stream_ended(&mut strict, 500).await;

    Ok(())
}
