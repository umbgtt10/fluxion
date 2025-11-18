// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::{FluxionStream, Ordered};
use fluxion_test_utils::sequenced::Sequenced;
use fluxion_test_utils::test_channel;
use fluxion_test_utils::test_data::{
    animal_dog, animal_spider, person_alice, person_bob, person_charlie, person_dave, person_diane,
    plant_rose, TestData,
};
use fluxion_test_utils::unwrap_value;
use futures::StreamExt;

#[tokio::test]
async fn test_filter_ordered_basic_predicate() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut stream =
        FluxionStream::new(stream).filter_ordered(|data| matches!(data, TestData::Person(_)));

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?;
    let result = unwrap_value(stream.next().await);
    assert_eq!(result.get(), &person_alice());

    tx.send(Sequenced::new(animal_dog()))?;
    tx.send(Sequenced::new(person_bob()))?;

    let result = unwrap_value(stream.next().await);
    assert_eq!(result.get(), &person_bob()); // Animal filtered out
    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_age_threshold() -> anyhow::Result<()> {
    // Arrange - filter people by age > 30
    let (tx, stream) = test_channel();
    let mut stream = FluxionStream::new(stream).filter_ordered(|data| match data {
        TestData::Person(p) => p.age > 30,
        _ => false,
    });

    // Act
    tx.send(Sequenced::new(person_alice()))?; // 25 - filtered
    tx.send(Sequenced::new(person_bob()))?; // 30 - filtered
    tx.send(Sequenced::new(person_charlie()))?; // 35 - kept
    tx.send(Sequenced::new(person_diane()))?; // 40 - kept

    // Assert
    let result = unwrap_value(stream.next().await);
    assert_eq!(result.get(), &person_charlie());

    let result = unwrap_value(stream.next().await);
    assert_eq!(result.get(), &person_diane());
    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_empty_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut stream = FluxionStream::new(stream).filter_ordered(|_| true);

    // Act
    drop(tx); // Close the channel

    // Assert
    assert!(stream.next().await.is_none());
    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_all_filtered_out() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut stream = FluxionStream::new(stream).filter_ordered(|_| false); // Filter everything

    // Act
    tx.send(Sequenced::new(person_alice()))?;
    tx.send(Sequenced::new(person_bob()))?;
    tx.send(Sequenced::new(animal_dog()))?;
    drop(tx);

    // Assert
    assert!(stream.next().await.is_none());
    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_none_filtered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut stream = FluxionStream::new(stream).filter_ordered(|_| true); // Keep everything

    // Act
    tx.send(Sequenced::new(person_alice()))?;
    tx.send(Sequenced::new(animal_dog()))?;
    tx.send(Sequenced::new(plant_rose()))?;

    // Assert
    assert_eq!(unwrap_value(stream.next().await).get(), &person_alice());
    assert_eq!(unwrap_value(stream.next().await).get(), &animal_dog());
    assert_eq!(unwrap_value(stream.next().await).get(), &plant_rose());
    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_preserves_ordering() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();

    // Keep only people with even ages
    let mut stream = FluxionStream::new(stream).filter_ordered(|data| match data {
        TestData::Person(p) => p.age % 2 == 0,
        _ => false,
    });

    // Act - send in sequence order
    tx.send(Sequenced::new(person_alice()))?; // 25 - odd, filtered
    tx.send(Sequenced::new(person_bob()))?; // 30 - even, kept
    tx.send(Sequenced::new(person_charlie()))?; // 35 - odd, filtered
    tx.send(Sequenced::new(person_diane()))?; // 40 - even, kept
    tx.send(Sequenced::new(person_dave()))?; // 28 - even, kept

    // Assert - ordering preserved for kept items
    let r1 = unwrap_value(stream.next().await);
    let r2 = unwrap_value(stream.next().await);
    let r3 = unwrap_value(stream.next().await);

    assert_eq!(r1.get(), &person_bob());
    assert_eq!(r2.get(), &person_diane());
    assert_eq!(r3.get(), &person_dave());

    // Verify sequence numbers are in order
    assert!(r1.order() < r2.order());
    assert!(r2.order() < r3.order());
    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_multiple_types() -> anyhow::Result<()> {
    // Arrange - keep only animals
    let (tx, stream) = test_channel();
    let mut stream =
        FluxionStream::new(stream).filter_ordered(|data| matches!(data, TestData::Animal(_)));

    // Act
    tx.send(Sequenced::new(person_alice()))?;
    tx.send(Sequenced::new(animal_dog()))?;
    tx.send(Sequenced::new(plant_rose()))?;
    tx.send(Sequenced::new(animal_spider()))?;
    tx.send(Sequenced::new(person_bob()))?;

    // Assert
    let result = unwrap_value(stream.next().await);
    assert_eq!(result.get(), &animal_dog());

    let result = unwrap_value(stream.next().await);
    assert_eq!(result.get(), &animal_spider());
    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_complex_predicate() -> anyhow::Result<()> {
    // Arrange - complex predicate: people with age between 30-40 OR animals
    let (tx, stream) = test_channel();
    let mut stream = FluxionStream::new(stream).filter_ordered(|data| match data {
        TestData::Person(p) => p.age >= 30 && p.age <= 40,
        TestData::Animal(_) => true,
        _ => false,
    });

    // Act
    tx.send(Sequenced::new(person_alice()))?; // 25 - filtered
    tx.send(Sequenced::new(person_bob()))?; // 30 - kept
    tx.send(Sequenced::new(animal_dog()))?; // kept
    tx.send(Sequenced::new(plant_rose()))?; // filtered
    tx.send(Sequenced::new(person_diane()))?; // 40 - kept

    // Assert
    assert_eq!(unwrap_value(stream.next().await).get(), &person_bob());
    assert_eq!(unwrap_value(stream.next().await).get(), &animal_dog());
    assert_eq!(unwrap_value(stream.next().await).get(), &person_diane());
    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_single_item() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let mut stream =
        FluxionStream::new(stream).filter_ordered(|data| matches!(data, TestData::Person(_)));

    // Act
    tx.send(Sequenced::new(person_alice()))?;
    drop(tx);

    // Assert
    let result = unwrap_value(stream.next().await);
    assert_eq!(result.get(), &person_alice());
    assert!(stream.next().await.is_none());
    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_with_pattern_matching() -> anyhow::Result<()> {
    // Arrange - filter by name pattern
    let (tx, stream) = test_channel();
    let mut stream = FluxionStream::new(stream).filter_ordered(|data| match data {
        TestData::Person(p) => p.name.starts_with('A') || p.name.starts_with('D'),
        _ => false,
    });

    // Act
    tx.send(Sequenced::new(person_alice()))?; // Alice - kept
    tx.send(Sequenced::new(person_bob()))?; // Bob - filtered
    tx.send(Sequenced::new(person_charlie()))?; // Charlie - filtered
    tx.send(Sequenced::new(person_dave()))?; // Dave - kept
    tx.send(Sequenced::new(person_diane()))?; // Diane - kept

    // Assert
    assert_eq!(unwrap_value(stream.next().await).get(), &person_alice());
    assert_eq!(unwrap_value(stream.next().await).get(), &person_dave());
    assert_eq!(unwrap_value(stream.next().await).get(), &person_diane());
    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_alternating_pattern() -> anyhow::Result<()> {
    // Arrange - Keep every other person by creating a stateful filter
    let (tx, stream) = test_channel();

    let mut count = 0;
    let mut stream = FluxionStream::new(stream).filter_ordered(move |data| {
        if matches!(data, TestData::Person(_)) {
            count += 1;
            count % 2 == 1 // Keep odd-numbered people
        } else {
            false
        }
    });

    // Act
    tx.send(Sequenced::new(person_alice()))?; // 1st person - kept
    tx.send(Sequenced::new(person_bob()))?; // 2nd person - filtered
    tx.send(Sequenced::new(animal_dog()))?; // not a person - filtered
    tx.send(Sequenced::new(person_charlie()))?; // 3rd person - kept
    tx.send(Sequenced::new(person_diane()))?; // 4th person - filtered

    // Assert
    assert_eq!(unwrap_value(stream.next().await).get(), &person_alice());
    assert_eq!(unwrap_value(stream.next().await).get(), &person_charlie());
    Ok(())
}
