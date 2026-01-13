// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::HasTimestamp;
use fluxion_stream::prelude::*;
use fluxion_stream::DistinctUntilChangedExt;
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, assert_stream_ended, unwrap_stream},
    test_channel,
    test_data::{person_alice, person_bob, person_charlie, TestData},
    Sequenced,
};

#[tokio::test]
async fn test_distinct_until_changed_basic() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut distinct = stream.distinct_until_changed();

    // Act & Assert: First value always emitted
    tx.try_send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        person_alice()
    );

    // Duplicate - filtered
    tx.try_send(Sequenced::new(person_alice()))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // New value - emitted
    tx.try_send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        person_bob()
    );

    // Another duplicate - filtered
    tx.try_send(Sequenced::new(person_bob()))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // New value - emitted
    tx.try_send(Sequenced::new(person_charlie()))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        person_charlie()
    );

    // Return to previous value - emitted (different from charlie)
    tx.try_send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_boolean_toggle() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<bool>>();
    let mut distinct = stream.distinct_until_changed();

    // Act & Assert: Initial state
    tx.try_send(Sequenced::new(false))?;
    assert!(!unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner());

    // Same state - filtered
    tx.try_send(Sequenced::new(false))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Toggle to true
    tx.try_send(Sequenced::new(true))?;
    assert!(unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner());

    // Same state - filtered
    tx.try_send(Sequenced::new(true))?;
    tx.try_send(Sequenced::new(true))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Toggle back to false
    tx.try_send(Sequenced::new(false))?;
    assert!(!unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner());

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_many_duplicates() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut distinct = stream.distinct_until_changed();

    // Act: Send many duplicates
    tx.try_send(Sequenced::new(person_alice()))?;
    for _ in 0..100 {
        tx.try_send(Sequenced::new(person_alice()))?;
    }
    tx.try_send(Sequenced::new(person_bob()))?;

    // Assert: Only two values emitted (alice and bob)
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        person_alice()
    );
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        person_bob()
    );
    assert_no_element_emitted(&mut distinct, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_alternating() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut distinct = stream.distinct_until_changed();

    // Act: Alternating values - all should be emitted
    tx.try_send(Sequenced::new(person_alice()))?;
    tx.try_send(Sequenced::new(person_bob()))?;
    tx.try_send(Sequenced::new(person_alice()))?;
    tx.try_send(Sequenced::new(person_bob()))?;
    tx.try_send(Sequenced::new(person_alice()))?;

    // Assert: All values emitted (each different from previous)
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        person_alice()
    );
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        person_bob()
    );
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        person_alice()
    );
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        person_bob()
    );
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        person_alice()
    );

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_different_types() -> anyhow::Result<()> {
    use fluxion_test_utils::test_data::{animal_dog, plant_rose};

    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut distinct = stream.distinct_until_changed();

    // Act & Assert
    tx.try_send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        person_alice()
    );

    tx.try_send(Sequenced::new(person_alice()))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    tx.try_send(Sequenced::new(animal_dog()))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        animal_dog()
    );

    tx.try_send(Sequenced::new(animal_dog()))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    tx.try_send(Sequenced::new(plant_rose()))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        plant_rose()
    );

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_fresh_timestamps() -> anyhow::Result<()> {
    use std::time::Duration;

    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut distinct = stream.distinct_until_changed();

    // Act: Send first value
    tx.try_send(Sequenced::new(person_alice()))?;
    let first = unwrap_stream(&mut distinct, 500).await.unwrap();
    let ts1 = first.timestamp();

    // Wait a bit
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Send duplicate (should be filtered)
    tx.try_send(Sequenced::new(person_alice()))?;

    // Wait a bit more
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Send new value
    tx.try_send(Sequenced::new(person_bob()))?;
    let second = unwrap_stream(&mut distinct, 500).await.unwrap();
    let ts2 = second.timestamp();

    // Assert: Timestamps are different (fresh generated)
    assert!(
        ts2 > ts1,
        "Expected fresh timestamp on distinct value emission"
    );

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_with_filter_ordered() -> anyhow::Result<()> {
    use fluxion_test_utils::test_data::{animal_dog, person_dave};

    // Arrange: Compose with filter_ordered - filter out people under 30
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut composed = stream
        .distinct_until_changed()
        .filter_ordered(|data| match data {
            TestData::Person(p) => p.age >= 30,
            _ => true,
        });

    // Act & Assert
    tx.try_send(Sequenced::new(person_alice()))?; // age=25, emitted by distinct, filtered by filter_ordered
    tx.try_send(Sequenced::new(person_alice()))?; // Filtered by distinct
    tx.try_send(Sequenced::new(person_bob()))?; // age=30, emitted by both
    assert_eq!(
        unwrap_stream(&mut composed, 500)
            .await
            .unwrap()
            .into_inner(),
        person_bob()
    );

    tx.try_send(Sequenced::new(person_bob()))?; // Filtered by distinct
    assert_no_element_emitted(&mut composed, 100).await;

    tx.try_send(Sequenced::new(person_charlie()))?; // age=35, emitted by both
    assert_eq!(
        unwrap_stream(&mut composed, 500)
            .await
            .unwrap()
            .into_inner(),
        person_charlie()
    );

    tx.try_send(Sequenced::new(person_dave()))?; // age=28, emitted by distinct, filtered by filter_ordered
    assert_no_element_emitted(&mut composed, 100).await;

    tx.try_send(Sequenced::new(animal_dog()))?; // Emitted by both (not a Person)
    assert_eq!(
        unwrap_stream(&mut composed, 500)
            .await
            .unwrap()
            .into_inner(),
        animal_dog()
    );

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_empty_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut distinct = stream.distinct_until_changed();

    // Act: Close stream without sending anything
    drop(tx);

    // Assert: Stream ends with no values
    assert_stream_ended(&mut distinct, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_single_value() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut distinct = stream.distinct_until_changed();

    // Act
    tx.try_send(Sequenced::new(person_alice()))?;

    // Assert: Single value emitted
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        person_alice()
    );

    drop(tx);
    assert_stream_ended(&mut distinct, 100).await;

    Ok(())
}
