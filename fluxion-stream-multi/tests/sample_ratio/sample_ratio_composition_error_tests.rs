// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Composition error tests for `sample_ratio` operator.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream_multi::{FilterOrderedExt, MapOrderedExt, SampleRatioExt};
use fluxion_test_utils::person::Person;
use fluxion_test_utils::test_data::{animal_dog, person_alice, person_bob, TestData};
use fluxion_test_utils::Sequenced;
use fluxion_test_utils::{helpers::unwrap_stream, test_channel_with_errors, unwrap_value};

#[tokio::test]
async fn test_sample_ratio_after_filter_ordered_with_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream
        .filter_ordered(|item| matches!(item, TestData::Person(_)))
        .sample_ratio(1.0, 42);

    // Act & Assert
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &person_alice()
    );

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("test error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Error(e) if e.to_string().contains("test error")
    ),);

    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_sample_ratio_before_filter_ordered_with_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream
        .sample_ratio(1.0, 42)
        .filter_ordered(|item| matches!(item, TestData::Person(_)));

    // Act & Assert
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &person_alice()
    );

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("test error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Error(e) if e.to_string().contains("test error")
    ),);

    Ok(())
}

#[tokio::test]
async fn test_sample_ratio_zero_still_passes_errors_in_composition() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream
        .sample_ratio(0.0, 42)
        .filter_ordered(|_: &TestData| true); // Pass-through filter

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("test error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Error(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_sample_ratio_with_map_ordered_and_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream
        .sample_ratio(1.0, 42)
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
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert!(matches!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        TestData::Person(p) if p.name.starts_with("Mapped: ")
    ));

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("test error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Error(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_chained_sample_ratios_with_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.sample_ratio(1.0, 42).sample_ratio(1.0, 99);

    // Act & Assert
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &person_alice()
    );

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("test error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Error(_)
    ));

    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_complex_pipeline_with_multiple_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream
        .filter_ordered(|item| matches!(item, TestData::Person(_)))
        .sample_ratio(1.0, 42)
        .map_ordered(|item: Sequenced<TestData>| {
            Sequenced::new(match item.value {
                TestData::Person(p) => TestData::Person(Person {
                    name: format!("Pipeline: {}", p.name),
                    ..p
                }),
                other => other,
            })
        });

    // Act & Assert - interleaved pattern
    tx.unbounded_send(StreamItem::Value(Sequenced::new(animal_dog())))?; // Filtered out

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("test error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Error(_)
    ));

    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert!(matches!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        TestData::Person(p) if p.name.starts_with("Pipeline: ")
    ));

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("test error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Error(_)
    ));

    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    assert!(matches!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        TestData::Person(p) if p.name.starts_with("Pipeline: ")
    ));

    Ok(())
}

#[tokio::test]
async fn test_sample_ratio_error_preserves_through_composition() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream
        .sample_ratio(1.0, 42)
        .filter_ordered(|_: &TestData| true);

    // Act
    let error_message = "specific error message preserved";
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(error_message)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Error(e) if e.to_string().contains(error_message)
    ));

    Ok(())
}
