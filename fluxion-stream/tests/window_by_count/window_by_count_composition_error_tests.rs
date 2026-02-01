// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, HasTimestamp, StreamItem};
use fluxion_stream::prelude::*;
use fluxion_test_utils::{
    helpers::{test_channel_with_errors, unwrap_stream, unwrap_value},
    sequenced::Sequenced,
    test_data::{
        animal_cat, animal_dog, person_alice, person_bob, person_charlie, plant_rose, TestData,
    },
};

#[tokio::test]
async fn test_map_ordered_then_window_by_count_propagates_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream
        .map_ordered(|x: Sequenced<TestData>| {
            Sequenced::with_timestamp(person_alice(), x.timestamp())
        })
        .window_by_count::<Sequenced<Vec<TestData>>>(2);

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("map error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.unbounded_send(StreamItem::Value(Sequenced::new(animal_dog())))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(animal_cat())))?;

    // Assert
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

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "filter error",
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_charlie())))?;

    // Assert
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

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        person_alice()
    );

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "window error",
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(animal_dog())))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(animal_cat())))?;

    // Assert
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
            window.iter().any(|d| matches!(d, TestData::Person(_)))
        });

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(animal_dog())))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(animal_cat())))?; // window [dog, cat], filtered
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(animal_dog())))?; // window [alice, dog], passes

    // Assert
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

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await))
            .current
            .value,
        vec![person_alice(), person_bob()]
    );

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(animal_dog())))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(animal_cat())))?;

    // Assert
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

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?; // inner [alice, bob]
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("chain error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(animal_dog())))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(animal_cat())))?; // inner [dog, cat]
    tx.unbounded_send(StreamItem::Value(Sequenced::new(plant_rose())))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_charlie())))?; // inner [rose, charlie] -> outer

    // Assert
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

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "test data error",
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_charlie())))?;

    // Assert
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
        .map_ordered(|x: Sequenced<TestData>| x)
        .window_by_count::<Sequenced<Vec<TestData>>>(2);

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("map error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(animal_dog())))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        vec![person_bob(), animal_dog()]
    );

    Ok(())
}
