// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::time::Duration;

use fluxion_stream::PartitionExt;
use fluxion_test_utils::test_data::{
    animal_bird, animal_cat, animal_dog, animal_spider, person_alice, person_bob, person_charlie,
    person_dave, person_diane, plant_fern, plant_rose, plant_sunflower, TestData,
};
use fluxion_test_utils::Sequenced;
use fluxion_test_utils::{assert_stream_ended, test_channel};
use fluxion_test_utils::{helpers::unwrap_stream, unwrap_value};
use tokio::time::sleep;

#[tokio::test]
async fn test_partition_basic_predicate() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let (mut persons, mut non_persons) =
        stream.partition(|data| matches!(data, TestData::Person(_)));

    // Act & Assert
    tx.try_send(Sequenced::new(person_alice()))?;
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut persons, 500).await)).value,
        &person_alice()
    );

    tx.try_send(Sequenced::new(animal_dog()))?;
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut non_persons, 500).await)).value,
        &animal_dog()
    );

    tx.try_send(Sequenced::new(person_bob()))?;
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut persons, 500).await)).value,
        &person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_partition_by_animal_legs() -> anyhow::Result<()> {
    // Arrange - partition animals by number of legs (4 vs not 4)
    let (tx, stream) = test_channel();
    let (mut four_legged, mut other_legged) = stream.partition(|data| match data {
        TestData::Animal(a) => a.legs == 4,
        _ => false,
    });

    // Act
    tx.try_send(Sequenced::new(animal_dog()))?; // 4 legs
    tx.try_send(Sequenced::new(animal_spider()))?; // 8 legs
    tx.try_send(Sequenced::new(animal_cat()))?; // 4 legs
    tx.try_send(Sequenced::new(animal_bird()))?; // 2 legs

    // Assert
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut four_legged, 500).await)).value,
        &animal_dog()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut other_legged, 500).await)).value,
        &animal_spider()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut four_legged, 500).await)).value,
        &animal_cat()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut other_legged, 500).await)).value,
        &animal_bird()
    );

    Ok(())
}

#[tokio::test]
async fn test_partition_age_threshold() -> anyhow::Result<()> {
    // Arrange - partition people by age > 30
    let (tx, stream) = test_channel();
    let (mut over_30, mut under_or_equal_30) = stream.partition(|data| match data {
        TestData::Person(p) => p.age > 30,
        _ => false,
    });

    // Act
    tx.try_send(Sequenced::new(person_alice()))?; // 30 - under or equal
    tx.try_send(Sequenced::new(person_charlie()))?; // 35 - over
    tx.try_send(Sequenced::new(person_diane()))?; // 40 - over
    tx.try_send(Sequenced::new(person_bob()))?; // 25 - under

    // Assert
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut under_or_equal_30, 500).await)).value,
        &person_alice()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut over_30, 500).await)).value,
        &person_charlie()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut over_30, 500).await)).value,
        &person_diane()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut under_or_equal_30, 500).await)).value,
        &person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_partition_empty_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let (mut true_stream, mut false_stream) = stream.partition(|_| true);

    // Act
    drop(tx); // Close the channel

    // Assert - both streams should end
    assert_stream_ended(&mut true_stream, 500).await;
    assert_stream_ended(&mut false_stream, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_partition_all_to_true() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let (mut true_stream, mut false_stream) = stream.partition(|_: &TestData| true);

    // Act
    tx.try_send(Sequenced::new(person_alice()))?;
    tx.try_send(Sequenced::new(person_bob()))?;
    tx.try_send(Sequenced::new(animal_dog()))?;
    drop(tx);

    // Assert - all go to true stream
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut true_stream, 500).await)).value,
        &person_alice()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut true_stream, 500).await)).value,
        &person_bob()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut true_stream, 500).await)).value,
        &animal_dog()
    );
    assert_stream_ended(&mut true_stream, 500).await;
    assert_stream_ended(&mut false_stream, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_partition_all_to_false() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let (mut true_stream, mut false_stream) = stream.partition(|_: &TestData| false);

    // Act
    tx.try_send(Sequenced::new(person_alice()))?;
    tx.try_send(Sequenced::new(person_bob()))?;
    tx.try_send(Sequenced::new(animal_dog()))?;
    drop(tx);

    // Assert - all go to false stream
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut false_stream, 500).await)).value,
        &person_alice()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut false_stream, 500).await)).value,
        &person_bob()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut false_stream, 500).await)).value,
        &animal_dog()
    );
    assert_stream_ended(&mut false_stream, 500).await;
    assert_stream_ended(&mut true_stream, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_partition_preserves_temporal_order() -> anyhow::Result<()> {
    // Arrange - partition with custom timestamps
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let (mut adults, mut young) = stream.partition(|data| match data {
        TestData::Person(p) => p.age >= 30,
        _ => false,
    });

    // Act - send with specific sequence numbers
    tx.try_send((person_bob(), 1).into())?; // age 30, adult, seq 1
    tx.try_send((person_alice(), 2).into())?; // age 25, young, seq 2
    tx.try_send((person_charlie(), 3).into())?; // age 35, adult, seq 3
    tx.try_send((person_dave(), 4).into())?; // age 28, young, seq 4

    // Assert - values arrive in original order within each partition
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut adults, 500).await)).value,
        person_bob()
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut young, 500).await)).value,
        person_alice()
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut adults, 500).await)).value,
        person_charlie()
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut young, 500).await)).value,
        person_dave()
    );

    Ok(())
}

#[tokio::test]
async fn test_partition_by_type() -> anyhow::Result<()> {
    // Arrange - partition by type (Person vs Animal vs Plant)
    let (tx, stream) = test_channel();
    let (mut animals, mut non_animals) =
        stream.partition(|data| matches!(data, TestData::Animal(_)));

    // Act
    tx.try_send(Sequenced::new(person_alice()))?;
    tx.try_send(Sequenced::new(animal_dog()))?;
    tx.try_send(Sequenced::new(plant_rose()))?;
    tx.try_send(Sequenced::new(animal_spider()))?;
    tx.try_send(Sequenced::new(person_bob()))?;

    // Assert
    // Non-animals: Alice, Rose, Bob
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut non_animals, 500).await)).value,
        &person_alice()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut non_animals, 500).await)).value,
        &plant_rose()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut non_animals, 500).await)).value,
        &person_bob()
    );

    // Animals: Dog, Spider
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut animals, 500).await)).value,
        &animal_dog()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut animals, 500).await)).value,
        &animal_spider()
    );

    Ok(())
}

#[tokio::test]
async fn test_partition_plant_height_threshold() -> anyhow::Result<()> {
    // Arrange - partition plants by height threshold (100cm)
    let (tx, stream) = test_channel();
    let height_threshold = 100;
    let (mut tall_plants, mut short_plants) = stream.partition(move |data| match data {
        TestData::Plant(p) => p.height >= height_threshold,
        _ => false,
    });

    // Act
    tx.try_send(Sequenced::new(plant_rose()))?; // height 15 - short
    tx.try_send(Sequenced::new(plant_sunflower()))?; // height 180 - tall
    tx.try_send(Sequenced::new(plant_fern()))?; // height 150 - tall

    // Assert
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut short_plants, 500).await)).value,
        &plant_rose()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut tall_plants, 500).await)).value,
        &plant_sunflower()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut tall_plants, 500).await)).value,
        &plant_fern()
    );

    Ok(())
}

#[tokio::test]
async fn test_partition_completes_both_on_close() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let (mut persons, mut non_persons) =
        stream.partition(|data| matches!(data, TestData::Person(_)));

    // Act
    tx.try_send(Sequenced::new(person_alice()))?;
    tx.try_send(Sequenced::new(animal_dog()))?;
    drop(tx); // Close the source

    // Assert - drain values first
    let _ = unwrap_stream(&mut persons, 500).await;
    let _ = unwrap_stream(&mut non_persons, 500).await;

    // Both should now be ended
    assert_stream_ended(&mut persons, 500).await;
    assert_stream_ended(&mut non_persons, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_partition_multiple_types_complex() -> anyhow::Result<()> {
    // Arrange - complex predicate involving multiple checks
    let (tx, stream) = test_channel();
    let (mut valid, mut _invalid) = stream.partition(|data| match data {
        TestData::Person(p) => p.age >= 18 && !p.name.is_empty(),
        TestData::Animal(a) => a.legs > 0,
        TestData::Plant(p) => p.height > 0,
    });

    // Act
    tx.try_send(Sequenced::new(person_alice()))?; // valid (age 30)
    tx.try_send(Sequenced::new(animal_dog()))?; // valid (4 legs)
    tx.try_send(Sequenced::new(plant_rose()))?; // valid (height > 0)
    tx.try_send(Sequenced::new(person_dave()))?; // valid (age 28)

    // Assert - all should be valid in this test
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut valid, 500).await)).value,
        &person_alice()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut valid, 500).await)).value,
        &animal_dog()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut valid, 500).await)).value,
        &plant_rose()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut valid, 500).await)).value,
        &person_dave()
    );

    Ok(())
}

#[tokio::test]
async fn test_partition_drop_one_stream_early() -> anyhow::Result<()> {
    // Arrange - partition by type, then drop one stream and continue reading the other
    let (tx, stream) = test_channel();
    let (mut persons, non_persons) = stream.partition(|data| matches!(data, TestData::Person(_)));

    // Drop the non_persons stream immediately
    drop(non_persons);

    // Act - send items of both types
    tx.try_send(Sequenced::new(person_alice()))?;
    tx.try_send(Sequenced::new(animal_dog()))?; // Goes to dropped stream - should be discarded
    tx.try_send(Sequenced::new(person_bob()))?;
    tx.try_send(Sequenced::new(animal_cat()))?; // Goes to dropped stream - should be discarded
    tx.try_send(Sequenced::new(person_charlie()))?;
    drop(tx);

    // Assert - persons stream should still work correctly
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut persons, 500).await)).value,
        &person_alice()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut persons, 500).await)).value,
        &person_bob()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut persons, 500).await)).value,
        &person_charlie()
    );
    assert_stream_ended(&mut persons, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_partition_drop_both_streams_gracefully() -> anyhow::Result<()> {
    // Arrange - create partition and drop both streams
    // This tests that dropping both streams doesn't panic or leak resources
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let (persons, non_persons) = stream.partition(|data| matches!(data, TestData::Person(_)));

    // Send some items before dropping
    tx.try_send(Sequenced::new(person_alice()))?;
    tx.try_send(Sequenced::new(animal_dog()))?;

    // Drop both partition streams - task should be cancelled gracefully
    drop(persons);
    drop(non_persons);

    // Give the task time to notice cancellation and clean up
    sleep(Duration::from_millis(50)).await;

    // Test passes if no panic occurred - resources are cleaned up properly
    Ok(())
}
