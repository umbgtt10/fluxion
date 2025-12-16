// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Composition error propagation tests for `window_by_count` operator.

use fluxion_core::{FluxionError, HasTimestamp, StreamItem};
use fluxion_stream::prelude::*;
use fluxion_test_utils::{
    test_channel_with_errors,
    test_data::{
        animal_cat, animal_dog, person_alice, person_bob, person_charlie, plant_rose, TestData,
    },
    unwrap_stream, unwrap_value, Sequenced,
};

#[tokio::test]
async fn test_map_ordered_then_window_by_count_propagates_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream
        .map_ordered(|x: Sequenced<TestData>| {
            // Map all to alice
            Sequenced::with_timestamp(person_alice(), x.timestamp())
        })
        .window_by_count::<Sequenced<Vec<TestData>>>(2);

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    tx.send(StreamItem::Error(FluxionError::stream_error("map error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Value(Sequenced::new(animal_dog())))?;
    tx.send(StreamItem::Value(Sequenced::new(animal_cat())))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        vec![person_alice(), person_alice()]
    );

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_then_window_by_count_propagates_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream
        .filter_ordered(|x: &TestData| matches!(x, TestData::Person(_)))
        .window_by_count::<Sequenced<Vec<TestData>>>(2);

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx.send(StreamItem::Error(FluxionError::stream_error(
        "filter error",
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    tx.send(StreamItem::Value(Sequenced::new(person_charlie())))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        vec![person_bob(), person_charlie()]
    );

    Ok(())
}

#[tokio::test]
async fn test_window_by_count_then_map_ordered_propagates_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream
        .window_by_count::<Sequenced<Vec<TestData>>>(2)
        .map_ordered(|window: Sequenced<Vec<TestData>>| {
            let first = window.value.first().cloned().unwrap_or(person_alice());
            Sequenced::with_timestamp(first, window.timestamp())
        });

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        person_alice()
    );

    tx.send(StreamItem::Error(FluxionError::stream_error(
        "window error",
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Value(Sequenced::new(animal_dog())))?;
    tx.send(StreamItem::Value(Sequenced::new(animal_cat())))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        animal_dog()
    );

    Ok(())
}

#[tokio::test]
async fn test_window_by_count_then_filter_ordered_propagates_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream
        .window_by_count::<Sequenced<Vec<TestData>>>(2)
        .filter_ordered(|window: &Vec<TestData>| {
            // Keep windows with at least one person
            window.iter().any(|d| matches!(d, TestData::Person(_)))
        });

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::new(animal_dog())))?;
    tx.send(StreamItem::Value(Sequenced::new(animal_cat())))?; // window [dog, cat], filtered
    tx.send(StreamItem::Error(FluxionError::stream_error("error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx.send(StreamItem::Value(Sequenced::new(animal_dog())))?; // window [alice, dog], passes
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        vec![person_alice(), animal_dog()]
    );

    Ok(())
}

#[tokio::test]
async fn test_window_by_count_with_combine_with_previous_propagates_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream
        .window_by_count::<Sequenced<Vec<TestData>>>(2)
        .combine_with_previous();

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await))
            .current
            .value,
        vec![person_alice(), person_bob()]
    );

    tx.send(StreamItem::Error(FluxionError::stream_error("error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Value(Sequenced::new(animal_dog())))?;
    tx.send(StreamItem::Value(Sequenced::new(animal_cat())))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await))
            .current
            .value,
        vec![animal_dog(), animal_cat()]
    );

    Ok(())
}

#[tokio::test]
async fn test_chained_window_by_count_propagates_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream
        .window_by_count::<Sequenced<Vec<TestData>>>(2)
        .window_by_count::<Sequenced<Vec<Vec<TestData>>>>(2);

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx.send(StreamItem::Value(Sequenced::new(person_bob())))?; // inner [alice, bob]
    tx.send(StreamItem::Error(FluxionError::stream_error("chain error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Value(Sequenced::new(animal_dog())))?;
    tx.send(StreamItem::Value(Sequenced::new(animal_cat())))?; // inner [dog, cat]
    tx.send(StreamItem::Value(Sequenced::new(plant_rose())))?;
    tx.send(StreamItem::Value(Sequenced::new(person_charlie())))?; // inner [rose, charlie] -> outer
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        vec![
            vec![animal_dog(), animal_cat()],
            vec![plant_rose(), person_charlie()]
        ]
    );

    Ok(())
}

#[tokio::test]
async fn test_window_by_count_with_test_data_propagates_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.window_by_count::<Sequenced<Vec<TestData>>>(2);

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx.send(StreamItem::Error(FluxionError::stream_error(
        "test data error",
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    tx.send(StreamItem::Value(Sequenced::new(person_charlie())))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        vec![person_bob(), person_charlie()]
    );

    Ok(())
}

#[tokio::test]
async fn test_map_ordered_identity_then_window_by_count_propagates_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream
        .map_ordered(|x: Sequenced<TestData>| x) // identity map
        .window_by_count::<Sequenced<Vec<TestData>>>(2);

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx.send(StreamItem::Error(FluxionError::stream_error("map error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    tx.send(StreamItem::Value(Sequenced::new(animal_dog())))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        vec![person_bob(), animal_dog()]
    );

    Ok(())
}
