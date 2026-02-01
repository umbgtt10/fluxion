// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::HasTimestamp;
use fluxion_stream::prelude::*;
use fluxion_stream::DistinctUntilChangedExt;
use fluxion_test_utils::test_data::animal_dog;
use fluxion_test_utils::test_data::plant_rose;
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, assert_stream_ended, test_channel, unwrap_stream},
    sequenced::Sequenced,
    test_data::{person_alice, person_bob, person_charlie, TestData},
};

#[tokio::test]
async fn test_distinct_until_changed_basic() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut distinct = stream.distinct_until_changed();

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        person_alice()
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_no_element_emitted(&mut distinct, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        person_bob()
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_no_element_emitted(&mut distinct, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        person_charlie()
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
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

    // Act
    tx.unbounded_send(Sequenced::new(false))?;

    // Assert
    assert!(!unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner());

    // Act
    tx.unbounded_send(Sequenced::new(false))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(true))?;

    // Assert
    assert!(unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner());

    // Act
    tx.unbounded_send(Sequenced::new(true))?;
    tx.unbounded_send(Sequenced::new(true))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(false))?;

    // Assert
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

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    for _ in 0..100 {
        tx.unbounded_send(Sequenced::new(person_alice()))?;
    }
    tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
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

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
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
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut distinct = stream.distinct_until_changed();

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        person_alice()
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        animal_dog()
    );

    // Act
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(plant_rose()))?;

    // Assert
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

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    let first = unwrap_stream(&mut distinct, 500).await.unwrap();
    let ts1 = first.timestamp();

    // Act
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Act
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Act
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    let second = unwrap_stream(&mut distinct, 500).await.unwrap();
    let ts2 = second.timestamp();

    // Assert
    assert!(
        ts2 > ts1,
        "Expected fresh timestamp on distinct value emission"
    );

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_with_filter_ordered() -> anyhow::Result<()> {
    use fluxion_test_utils::test_data::{animal_dog, person_dave};

    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut composed = stream
        .distinct_until_changed()
        .filter_ordered(|data| match data {
            TestData::Person(p) => p.age >= 30,
            _ => true,
        });

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?; // age=25, emitted by distinct, filtered by filter_ordered
    tx.unbounded_send(Sequenced::new(person_alice()))?; // Filtered by distinct
    tx.unbounded_send(Sequenced::new(person_bob()))?; // age=30, emitted by both

    // Assert
    assert_eq!(
        unwrap_stream(&mut composed, 500)
            .await
            .unwrap()
            .into_inner(),
        person_bob()
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_bob()))?; // Filtered by distinct
    assert_no_element_emitted(&mut composed, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(person_charlie()))?; // age=35, emitted by both

    // Assert
    assert_eq!(
        unwrap_stream(&mut composed, 500)
            .await
            .unwrap()
            .into_inner(),
        person_charlie()
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_dave()))?; // age=28, emitted by distinct, filtered by filter_ordered
    assert_no_element_emitted(&mut composed, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(animal_dog()))?; // Emitted by both (not a Person)

    // Assert
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

    // Act
    drop(tx);

    // Assert
    assert_stream_ended(&mut distinct, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_single_value() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut distinct = stream.distinct_until_changed();

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
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
