// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Composition error tests for `tap` operator.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::{FilterOrderedExt, MapOrderedExt, TapExt};
use fluxion_test_utils::person::Person;
use fluxion_test_utils::test_data::{person_alice, person_bob, TestData};
use fluxion_test_utils::Sequenced;
use fluxion_test_utils::{helpers::unwrap_stream, test_channel_with_errors, unwrap_value};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[tokio::test]
async fn test_tap_after_filter_ordered_with_errors() -> anyhow::Result<()> {
    // Arrange
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream
        .filter_ordered(|item| matches!(item, TestData::Person(_)))
        .tap(move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &person_alice()
    );

    tx.send(StreamItem::Error(FluxionError::stream_error("test error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &person_bob()
    );

    assert_eq!(counter.load(Ordering::SeqCst), 2);

    Ok(())
}

#[tokio::test]
async fn test_tap_before_filter_ordered_with_errors() -> anyhow::Result<()> {
    // Arrange
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream
        .tap(move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        })
        .filter_ordered(|item| matches!(item, TestData::Person(_)));

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &person_alice()
    );

    tx.send(StreamItem::Error(FluxionError::stream_error("test error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &person_bob()
    );

    assert_eq!(counter.load(Ordering::SeqCst), 2);

    Ok(())
}

#[tokio::test]
async fn test_tap_with_map_ordered_and_errors() -> anyhow::Result<()> {
    // Arrange
    let observed = Arc::new(Mutex::new(Vec::new()));
    let observed_clone = observed.clone();

    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream
        .tap(move |value| {
            observed_clone.lock().push(value.clone());
        })
        .map_ordered(|item: Sequenced<TestData>| {
            Sequenced::new(match item.value {
                TestData::Person(p) => TestData::Person(Person {
                    name: format!("Mapped: {}", p.name),
                    ..p
                }),
                other => other,
            })
        });

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert!(matches!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        TestData::Person(p) if p.name.starts_with("Mapped: ")
    ));

    tx.send(StreamItem::Error(FluxionError::stream_error("test error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Error(_)
    ));

    assert_eq!(*observed.lock(), [person_alice()]);

    Ok(())
}

#[tokio::test]
async fn test_chained_taps_with_errors() -> anyhow::Result<()> {
    // Arrange
    let counter1 = Arc::new(AtomicUsize::new(0));
    let counter1_clone = counter1.clone();
    let counter2 = Arc::new(AtomicUsize::new(0));
    let counter2_clone = counter2.clone();

    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream
        .tap(move |_| {
            counter1_clone.fetch_add(1, Ordering::SeqCst);
        })
        .tap(move |_| {
            counter2_clone.fetch_add(1, Ordering::SeqCst);
        });

    // Act
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    unwrap_stream(&mut result, 500).await;

    tx.send(StreamItem::Error(FluxionError::stream_error("error")))?;
    unwrap_stream(&mut result, 500).await;

    tx.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    unwrap_stream(&mut result, 500).await;

    // Assert
    assert_eq!(counter1.load(Ordering::SeqCst), 2);
    assert_eq!(counter2.load(Ordering::SeqCst), 2);

    Ok(())
}

#[tokio::test]
async fn test_complex_pipeline_with_multiple_errors() -> anyhow::Result<()> {
    // Arrange
    let counter_before_filter = Arc::new(AtomicUsize::new(0));
    let counter_before_filter_clone = counter_before_filter.clone();
    let counter_after_map = Arc::new(AtomicUsize::new(0));
    let counter_after_map_clone = counter_after_map.clone();

    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream
        .tap(move |_| {
            counter_before_filter_clone.fetch_add(1, Ordering::SeqCst);
        })
        .filter_ordered(|item| matches!(item, TestData::Person(_)))
        .map_ordered(|item: Sequenced<TestData>| {
            Sequenced::new(match item.value {
                TestData::Person(p) => TestData::Person(Person {
                    name: format!("Pipeline: {}", p.name),
                    ..p
                }),
                other => other,
            })
        })
        .tap(move |_| {
            counter_after_map_clone.fetch_add(1, Ordering::SeqCst);
        });

    // Act & Assert
    tx.send(StreamItem::Error(FluxionError::stream_error("error 1")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert!(matches!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        TestData::Person(p) if p.name.starts_with("Pipeline: ")
    ));

    tx.send(StreamItem::Error(FluxionError::stream_error("error 2")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    assert!(matches!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        TestData::Person(p) if p.name.starts_with("Pipeline: ")
    ));

    assert_eq!(counter_before_filter.load(Ordering::SeqCst), 2);
    assert_eq!(counter_after_map.load(Ordering::SeqCst), 2);

    Ok(())
}

#[tokio::test]
async fn test_tap_error_preserves_through_composition() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.tap(|_| {}).filter_ordered(|_: &TestData| true);

    // Act
    let error_message = "specific error message preserved";
    tx.send(StreamItem::Error(FluxionError::stream_error(error_message)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Error(e) if e.to_string().contains(error_message)
    ));

    Ok(())
}
