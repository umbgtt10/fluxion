// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Basic tests for `tap` operator.

use fluxion_core::HasTimestamp;
use fluxion_stream::TapExt;
use fluxion_test_utils::test_data::{
    animal_bird, animal_cat, animal_dog, person_alice, person_bob, person_charlie, plant_rose,
    TestData,
};
use fluxion_test_utils::{assert_no_element_emitted, test_channel};
use fluxion_test_utils::{helpers::unwrap_stream, unwrap_value, Sequenced};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[tokio::test]
async fn test_tap_values_pass_through_unchanged() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut result = stream.tap(|_| {});

    // Act & Assert
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &person_alice()
    );

    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &animal_dog()
    );

    tx.unbounded_send(Sequenced::new(plant_rose()))?;
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &plant_rose()
    );

    Ok(())
}

#[tokio::test]
async fn test_tap_side_effect_called_for_each_value() -> anyhow::Result<()> {
    // Arrange
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream.tap(move |_| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    unwrap_stream(&mut result, 500).await;

    tx.unbounded_send(Sequenced::new(person_bob()))?;
    unwrap_stream(&mut result, 500).await;

    tx.unbounded_send(Sequenced::new(person_charlie()))?;
    unwrap_stream(&mut result, 500).await;

    // Assert - side effect called 3 times
    assert_eq!(counter.load(Ordering::SeqCst), 3);

    Ok(())
}

#[tokio::test]
async fn test_tap_receives_correct_values() -> anyhow::Result<()> {
    // Arrange
    let observed = Arc::new(parking_lot::Mutex::new(Vec::new()));
    let observed_clone = observed.clone();

    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream.tap(move |value| {
        observed_clone.lock().push(value.clone());
    });

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    unwrap_stream(&mut result, 500).await;

    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    unwrap_stream(&mut result, 500).await;

    tx.unbounded_send(Sequenced::new(plant_rose()))?;
    unwrap_stream(&mut result, 500).await;

    // Assert - correct values observed
    assert_eq!(
        *observed.lock(),
        [person_alice(), animal_dog(), plant_rose()]
    );

    Ok(())
}

#[tokio::test]
async fn test_tap_empty_stream() -> anyhow::Result<()> {
    // Arrange
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let (_tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream.tap(move |_| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    // Act & Assert - stream stays open but no items sent
    assert_no_element_emitted(&mut result, 500).await;
    assert_eq!(counter.load(Ordering::SeqCst), 0);

    Ok(())
}

#[tokio::test]
async fn test_tap_preserves_timestamps() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut result = stream.tap(|_| {});

    // Act & Assert
    let item1 = Sequenced::new(person_alice());
    let timestamp1 = item1.timestamp();
    tx.unbounded_send(item1)?;

    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).timestamp(),
        timestamp1
    );

    let item2 = Sequenced::new(person_bob());
    let timestamp2 = item2.timestamp();
    tx.unbounded_send(item2)?;

    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).timestamp(),
        timestamp2
    );

    Ok(())
}

#[tokio::test]
async fn test_tap_multiple_types() -> anyhow::Result<()> {
    // Arrange
    let observed = Arc::new(parking_lot::Mutex::new(Vec::new()));
    let observed_clone = observed.clone();

    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream.tap(move |value| {
        observed_clone.lock().push(value.clone());
    });

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    unwrap_stream(&mut result, 500).await;

    tx.unbounded_send(Sequenced::new(animal_cat()))?;
    unwrap_stream(&mut result, 500).await;

    tx.unbounded_send(Sequenced::new(animal_bird()))?;
    unwrap_stream(&mut result, 500).await;

    // Assert
    assert!(matches!(
        *observed.lock().as_slice(),
        [
            TestData::Person(_),
            TestData::Animal(_),
            TestData::Animal(_)
        ]
    ));

    Ok(())
}

#[tokio::test]
async fn test_tap_maintains_order() -> anyhow::Result<()> {
    // Arrange
    let observed = Arc::new(parking_lot::Mutex::new(Vec::new()));
    let observed_clone = observed.clone();

    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream.tap(move |value| {
        observed_clone.lock().push(value.clone());
    });

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Consume all
    unwrap_stream(&mut result, 500).await;
    unwrap_stream(&mut result, 500).await;
    unwrap_stream(&mut result, 500).await;

    // Assert
    assert_eq!(
        *observed.lock(),
        [person_alice(), person_bob(), person_charlie()]
    );

    Ok(())
}
