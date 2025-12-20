// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Basic tests for `sample_ratio` operator.

use fluxion_stream::SampleRatioExt;
use fluxion_test_utils::test_data::{
    animal_bird, animal_cat, animal_dog, animal_spider, person_alice, person_bob, person_charlie,
    person_dave, person_diane, plant_fern, plant_rose, plant_sunflower, TestData,
};
use fluxion_test_utils::{assert_no_element_emitted, unwrap_all, Sequenced};
use fluxion_test_utils::{assert_stream_ended, test_channel};
use fluxion_test_utils::{helpers::unwrap_stream, unwrap_value};

#[tokio::test]
async fn test_sample_ratio_one_emits_all() -> anyhow::Result<()> {
    // Arrange - ratio 1.0 should emit all items
    let (tx, stream) = test_channel();
    let mut result = stream.sample_ratio(1.0, 42);

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
async fn test_sample_ratio_zero_emits_nothing() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream.sample_ratio(0.0, 42);

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Assert
    assert_no_element_emitted(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_sample_ratio_send_dropped_empty_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream.sample_ratio(0.5, 42);

    // Act
    drop(tx); // Close the channel

    // Assert
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_sample_ratio_deterministic_with_same_seed() -> anyhow::Result<()> {
    // Arrange
    let items = vec![
        person_alice(),
        person_bob(),
        person_charlie(),
        person_dave(),
        person_diane(),
        animal_dog(),
        animal_cat(),
        animal_bird(),
        animal_spider(),
        plant_rose(),
        plant_fern(),
        plant_sunflower(),
    ];
    let ratio = 0.5;
    let seed = 12345;

    let (tx1, stream1) = test_channel();
    let (tx2, stream2) = test_channel();
    let mut result1 = stream1.sample_ratio(ratio, seed);
    let mut result2 = stream2.sample_ratio(ratio, seed);

    // Act
    for item in &items {
        tx1.unbounded_send(Sequenced::new(item.clone()))?;
        tx2.unbounded_send(Sequenced::new(item.clone()))?;
    }

    // Assert
    assert_eq!(
        unwrap_all(&mut result1, 100)
            .await
            .into_iter()
            .map(|s| s.value)
            .collect::<Vec<_>>(),
        unwrap_all(&mut result2, 100)
            .await
            .into_iter()
            .map(|s| s.value)
            .collect::<Vec<_>>(),
        "Same seeds should usually produce the same results"
    );

    Ok(())
}

#[tokio::test]
async fn test_sample_ratio_different_seeds_produce_different_results() -> anyhow::Result<()> {
    // Arrange
    let items = vec![
        person_alice(),
        person_bob(),
        person_charlie(),
        person_dave(),
        person_diane(),
        animal_dog(),
        animal_cat(),
        animal_bird(),
        animal_spider(),
        plant_rose(),
        plant_fern(),
        plant_sunflower(),
    ];
    let ratio = 0.5;

    let (tx1, stream1) = test_channel();
    let (tx2, stream2) = test_channel();
    let mut result1 = stream1.sample_ratio(ratio, 111);
    let mut result2 = stream2.sample_ratio(ratio, 222);

    // Act
    for item in &items {
        tx1.unbounded_send(Sequenced::new(item.clone()))?;
        tx2.unbounded_send(Sequenced::new(item.clone()))?;
    }

    // Assert
    assert_ne!(
        unwrap_all(&mut result1, 100)
            .await
            .into_iter()
            .map(|s| s.value)
            .collect::<Vec<_>>(),
        unwrap_all(&mut result2, 100)
            .await
            .into_iter()
            .map(|s| s.value)
            .collect::<Vec<_>>(),
        "Different seeds should usually produce different results"
    );

    Ok(())
}

#[tokio::test]
async fn test_sample_ratio_preserves_item_order() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut result = stream.sample_ratio(1.0, 42);

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &person_alice()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &person_bob()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        &person_charlie()
    );

    Ok(())
}

#[tokio::test]
async fn test_sample_ratio_boundary_values() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel();
    let (tx2, stream2) = test_channel();
    let mut result1 = stream1.sample_ratio(0.0, 42);
    let mut result2 = stream2.sample_ratio(1.0, 42);

    // Act
    tx1.unbounded_send(Sequenced::new(person_alice()))?;
    tx2.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_no_element_emitted(&mut result1, 500).await;
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut result2, 500).await)).value,
        &person_alice()
    );

    Ok(())
}

#[tokio::test]
#[should_panic(expected = "ratio must be between 0.0 and 1.0")]
async fn test_sample_ratio_panics_on_negative() {
    let (_, stream) = test_channel::<Sequenced<TestData>>();
    let _ = stream.sample_ratio(-0.1, 42);
}

#[tokio::test]
#[should_panic(expected = "ratio must be between 0.0 and 1.0")]
async fn test_sample_ratio_panics_on_greater_than_one() {
    let (_, stream) = test_channel::<Sequenced<TestData>>();
    let _ = stream.sample_ratio(1.1, 42);
}
