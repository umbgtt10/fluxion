// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::HasTimestamp;
use fluxion_stream::prelude::*;
use fluxion_test_utils::{
    assert_no_element_emitted, test_channel,
    test_data::{
        animal_cat, animal_dog, person_alice, person_bob, person_charlie, plant_rose, TestData,
    },
    unwrap_stream, unwrap_value, Sequenced,
};
use parking_lot::Mutex;
use std::sync::Arc;

#[tokio::test]
async fn test_window_by_count_then_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream
        .window_by_count::<Sequenced<Vec<TestData>>>(2)
        .map_ordered(|window: Sequenced<Vec<TestData>>| {
            // Extract first item from window
            let first = window.value.first().cloned().unwrap_or(person_alice());
            Sequenced::with_timestamp(first, window.timestamp())
        });

    // Act & Assert
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        person_alice()
    );

    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    tx.unbounded_send(Sequenced::new(animal_cat()))?;
    let first2 = unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value;
    assert_eq!(first2, animal_dog());

    Ok(())
}

#[tokio::test]
async fn test_window_by_count_then_filter_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream
        .window_by_count::<Sequenced<Vec<TestData>>>(2)
        .filter_ordered(|window: &Vec<TestData>| {
            // Keep windows that contain at least one person
            window.iter().any(|d| matches!(d, TestData::Person(_)))
        });

    // Act
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    tx.unbounded_send(Sequenced::new(animal_cat()))?; // window [dog, cat] - no person, filtered
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?; // window [alice, dog] - has person, passes
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        vec![person_alice(), animal_dog()]
    );

    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(person_charlie()))?; // window [bob, charlie] - has person, passes
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        vec![person_bob(), person_charlie()]
    );

    Ok(())
}

#[tokio::test]
async fn test_map_ordered_then_window_by_count() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream
        .map_ordered(|x: Sequenced<TestData>| {
            // Map all items to person_alice for simplicity
            Sequenced::with_timestamp(person_alice(), x.timestamp())
        })
        .window_by_count::<Sequenced<Vec<TestData>>>(2);

    // Act & Assert
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        vec![person_alice(), person_alice()]
    );

    tx.unbounded_send(Sequenced::new(plant_rose()))?;
    tx.unbounded_send(Sequenced::new(person_charlie()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        vec![person_alice(), person_alice()]
    );

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_then_window_by_count() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream
        .filter_ordered(|x: &TestData| matches!(x, TestData::Person(_))) // Only persons
        .window_by_count::<Sequenced<Vec<TestData>>>(2);

    // Act & Assert
    tx.unbounded_send(Sequenced::new(animal_dog()))?; // filtered
    tx.unbounded_send(Sequenced::new(person_alice()))?; // kept
    tx.unbounded_send(Sequenced::new(plant_rose()))?; // filtered
    tx.unbounded_send(Sequenced::new(person_bob()))?; // kept -> window [alice, bob]
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        vec![person_alice(), person_bob()]
    );

    tx.unbounded_send(Sequenced::new(animal_cat()))?; // filtered
    tx.unbounded_send(Sequenced::new(person_charlie()))?; // kept
    tx.unbounded_send(Sequenced::new(person_alice()))?; // kept -> window [charlie, alice]
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        vec![person_charlie(), person_alice()]
    );

    Ok(())
}

#[tokio::test]
async fn test_window_by_count_chain_different_sizes() -> anyhow::Result<()> {
    // Arrange: Window of 2, then window of 2 (creates windows of windows)
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream
        .window_by_count::<Sequenced<Vec<TestData>>>(2)
        .window_by_count::<Sequenced<Vec<Vec<TestData>>>>(2);

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?; // first inner window
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    tx.unbounded_send(Sequenced::new(animal_cat()))?; // second inner window -> outer window

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        vec![
            vec![person_alice(), person_bob()],
            vec![animal_dog(), animal_cat()]
        ]
    );

    Ok(())
}

#[tokio::test]
async fn test_window_by_count_with_combine_with_previous() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream
        .window_by_count::<Sequenced<Vec<TestData>>>(2)
        .combine_with_previous();

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    tx.unbounded_send(Sequenced::new(animal_cat()))?;

    // Assert
    let first = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(
        (first.current.value, first.previous),
        (vec![person_alice(), person_bob()], None)
    );

    let second = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(
        (second.current.value, second.previous.map(|p| p.value)),
        (
            vec![animal_dog(), animal_cat()],
            Some(vec![person_alice(), person_bob()])
        )
    );

    Ok(())
}

#[tokio::test]
async fn test_window_by_count_preserves_timestamp_ordering() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream.window_by_count::<Sequenced<Vec<TestData>>>(2);

    // Act & Assert
    tx.unbounded_send(Sequenced::with_timestamp(person_alice(), 100))?;
    tx.unbounded_send(Sequenced::with_timestamp(person_bob(), 200))?; // Window ts = 200
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).timestamp(),
        200
    );

    tx.unbounded_send(Sequenced::with_timestamp(animal_dog(), 300))?;
    tx.unbounded_send(Sequenced::with_timestamp(animal_cat(), 400))?; // Window ts = 400
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).timestamp(),
        400
    );

    Ok(())
}

#[tokio::test]
async fn test_window_by_count_with_map_extracts_first() -> anyhow::Result<()> {
    // Arrange: Use map to extract first item of each window
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream
        .window_by_count::<Sequenced<Vec<TestData>>>(2)
        .map_ordered(|window: Sequenced<Vec<TestData>>| {
            let first = window.value.first().cloned().unwrap_or(person_alice());
            Sequenced::with_timestamp(first, window.timestamp())
        });

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    tx.unbounded_send(Sequenced::new(animal_cat()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        person_alice()
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        animal_dog()
    );

    Ok(())
}

#[tokio::test]
async fn test_window_by_count_with_tap() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let tapped_values = Arc::new(Mutex::new(Vec::new()));
    let tapped_clone = Arc::clone(&tapped_values);

    let mut result =
        stream
            .window_by_count::<Sequenced<Vec<TestData>>>(2)
            .tap(move |window: &Vec<TestData>| {
                tapped_clone.lock().push(window.clone());
            });

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    tx.unbounded_send(Sequenced::new(animal_cat()))?;

    // Consume results
    let _ = unwrap_stream(&mut result, 100).await;
    let _ = unwrap_stream(&mut result, 100).await;

    // Assert: tap saw the windows
    assert_eq!(
        *tapped_values.lock(),
        vec![
            vec![person_alice(), person_bob()],
            vec![animal_dog(), animal_cat()]
        ]
    );

    Ok(())
}
