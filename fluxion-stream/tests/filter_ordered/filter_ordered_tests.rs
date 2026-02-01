// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::HasTimestamp;

use fluxion_stream::FilterOrderedExt;
use fluxion_test_utils::helpers::{assert_stream_ended, test_channel, unwrap_stream, unwrap_value};
use fluxion_test_utils::sequenced::Sequenced;
use fluxion_test_utils::test_data::{
    animal_dog, animal_spider, person_alice, person_bob, person_charlie, person_dave, person_diane,
    plant_rose, TestData,
};

#[tokio::test]
async fn test_filter_ordered_basic_predicate() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut result = stream.filter_ordered(|data| matches!(data, TestData::Person(_)));

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        person_alice()
    );

    // Act
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_age_threshold() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut result = stream.filter_ordered(|data| match data {
        TestData::Person(p) => p.age > 30,
        _ => false,
    });

    // Act
    tx.unbounded_send(Sequenced::new(person_charlie()))?;
    tx.unbounded_send(Sequenced::new(person_diane()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        person_charlie()
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        person_diane()
    );

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_empty_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream.filter_ordered(|_| true);

    // Act
    drop(tx);

    // Assert
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_all_filtered_out() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut result = stream.filter_ordered(|_| false);

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    drop(tx);

    // Assert
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_none_filtered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut result = stream.filter_ordered(|_| true);

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    tx.unbounded_send(Sequenced::new(plant_rose()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        person_alice()
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_dog()
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        plant_rose()
    );

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_preserves_ordering() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();

    let mut result = stream.filter_ordered(|data| match data {
        TestData::Person(p) => p.age % 2 == 0,
        _ => false,
    });

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(person_charlie()))?;
    tx.unbounded_send(Sequenced::new(person_diane()))?;
    tx.unbounded_send(Sequenced::new(person_dave()))?;

    // Assert
    let r1 = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    let r2 = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    let r3 = unwrap_value(Some(unwrap_stream(&mut result, 500).await));

    assert_eq!(r1.value, person_bob());
    assert_eq!(r2.value, person_diane());
    assert_eq!(r3.value, person_dave());
    assert!(r1.timestamp() < r2.timestamp());
    assert!(r2.timestamp() < r3.timestamp());

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_multiple_types() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut result = stream.filter_ordered(|data| matches!(data, TestData::Animal(_)));

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    tx.unbounded_send(Sequenced::new(plant_rose()))?;
    tx.unbounded_send(Sequenced::new(animal_spider()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_dog()
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_spider()
    );

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_complex_predicate() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut result = stream.filter_ordered(|data| match data {
        TestData::Person(p) => p.age >= 30 && p.age <= 40,
        TestData::Animal(_) => true,
        _ => false,
    });

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    tx.unbounded_send(Sequenced::new(plant_rose()))?;
    tx.unbounded_send(Sequenced::new(person_diane()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        person_bob()
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        animal_dog()
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        person_diane()
    );

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_single_item() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut result = stream.filter_ordered(|data| matches!(data, TestData::Person(_)));

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    drop(tx);

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        person_alice()
    );
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_with_pattern_matching() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut result = stream.filter_ordered(|data| match data {
        TestData::Person(p) => p.name.starts_with('A') || p.name.starts_with('D'),
        _ => false,
    });

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(person_charlie()))?;
    tx.unbounded_send(Sequenced::new(person_dave()))?;
    tx.unbounded_send(Sequenced::new(person_diane()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        person_alice()
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        person_dave()
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        person_diane()
    );

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_alternating_pattern() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();

    let mut count = 0;
    let mut result = stream.filter_ordered(move |data| {
        if matches!(data, TestData::Person(_)) {
            count += 1;
            count % 2 == 1
        } else {
            false
        }
    });

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    tx.unbounded_send(Sequenced::new(person_charlie()))?;
    tx.unbounded_send(Sequenced::new(person_diane()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        person_alice()
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        person_charlie()
    );

    Ok(())
}
