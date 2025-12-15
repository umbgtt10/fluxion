// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::prelude::*;
use fluxion_test_utils::{
    assert_stream_ended,
    helpers::unwrap_stream,
    test_channel,
    test_data::{
        animal_bird, animal_cat, animal_dog, animal_spider, person_alice, person_bob,
        person_charlie, person_dave, person_diane, plant_rose, TestData,
    },
    unwrap_value, Sequenced,
};

#[tokio::test]
async fn test_partition_then_filter_ordered() -> anyhow::Result<()> {
    // Arrange - partition by type, then filter each partition further
    let (tx, stream) = test_channel();
    let (persons, non_persons) = stream.partition(|data| matches!(data, TestData::Person(_)));

    // Filter persons for age > 25
    let mut older_persons = persons.filter_ordered(|data| match data {
        TestData::Person(p) => p.age > 25,
        _ => false,
    });

    // Filter non-persons for animals only
    let mut animals = non_persons.filter_ordered(|data| matches!(data, TestData::Animal(_)));

    // Act
    tx.send(Sequenced::new(person_alice()))?; // age 25 - filtered out
    tx.send(Sequenced::new(person_bob()))?; // age 30 - kept
    tx.send(Sequenced::new(animal_dog()))?; // animal - kept
    tx.send(Sequenced::new(plant_rose()))?; // plant - filtered out
    tx.send(Sequenced::new(person_charlie()))?; // age 35 - kept

    // Assert
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut older_persons, 500).await)).value,
        &person_bob()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut animals, 500).await)).value,
        &animal_dog()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut older_persons, 500).await)).value,
        &person_charlie()
    );

    Ok(())
}

#[tokio::test]
async fn test_partition_then_map_ordered() -> anyhow::Result<()> {
    // Arrange - partition by type, then extract names from each
    let (tx, stream) = test_channel();
    let (persons, animals) = stream.partition(|data| matches!(data, TestData::Person(_)));

    // Map persons to their names
    let mut person_names = persons.map_ordered(|s: Sequenced<TestData>| match s.into_inner() {
        TestData::Person(p) => Sequenced::new(p.name),
        _ => Sequenced::new(String::new()),
    });

    // Map animals to their species
    let mut animal_species = animals.map_ordered(|s: Sequenced<TestData>| match s.into_inner() {
        TestData::Animal(a) => Sequenced::new(a.species),
        _ => Sequenced::new(String::new()),
    });

    // Act
    tx.send(Sequenced::new(person_alice()))?;
    tx.send(Sequenced::new(animal_dog()))?;
    tx.send(Sequenced::new(person_bob()))?;
    tx.send(Sequenced::new(animal_cat()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut person_names, 500).await)).into_inner(),
        "Alice"
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut animal_species, 500).await)).into_inner(),
        "Dog"
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut person_names, 500).await)).into_inner(),
        "Bob"
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut animal_species, 500).await)).into_inner(),
        "Cat"
    );

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_then_partition() -> anyhow::Result<()> {
    // Arrange - filter for adults only, then partition by age threshold
    let (tx, stream) = test_channel();

    // Filter for adults (age >= 18), then partition into young adults vs seniors
    let adults = stream.filter_ordered(|data| match data {
        TestData::Person(p) => p.age >= 18,
        _ => false,
    });
    let (mut seniors, mut young_adults) = adults.partition(|data| match data {
        TestData::Person(p) => p.age >= 35,
        _ => false,
    });

    // Act
    tx.send(Sequenced::new(person_alice()))?; // 25 - young adult
    tx.send(Sequenced::new(person_charlie()))?; // 35 - senior
    tx.send(Sequenced::new(person_dave()))?; // 28 - young adult
    tx.send(Sequenced::new(person_diane()))?; // 40 - senior

    // Assert - all are adults, partitioned by age threshold
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut young_adults, 500).await)).value,
        &person_alice()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut seniors, 500).await)).value,
        &person_charlie()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut young_adults, 500).await)).value,
        &person_dave()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut seniors, 500).await)).value,
        &person_diane()
    );

    Ok(())
}

#[tokio::test]
async fn test_map_ordered_then_partition() -> anyhow::Result<()> {
    // Arrange - extract leg count from animals, then partition by leg count
    let (tx, stream) = test_channel();

    // Extract leg count from animals, then partition by threshold (>= 4 legs)
    let leg_counts = stream.map_ordered(|s: Sequenced<TestData>| match s.into_inner() {
        TestData::Animal(a) => Sequenced::new(a.legs),
        _ => Sequenced::new(0),
    });
    let (mut many_legs, mut few_legs) = leg_counts.partition(|&legs| legs >= 4);

    // Act
    tx.send(Sequenced::new(animal_bird()))?; // 2 legs - few
    tx.send(Sequenced::new(animal_dog()))?; // 4 legs - many
    tx.send(Sequenced::new(animal_spider()))?; // 8 legs - many

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut few_legs, 500).await)).into_inner(),
        2
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut many_legs, 500).await)).into_inner(),
        4
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut many_legs, 500).await)).into_inner(),
        8
    );

    Ok(())
}

#[tokio::test]
async fn test_partition_then_ordered_merge() -> anyhow::Result<()> {
    // Arrange - partition by type, then merge back together
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let (persons, non_persons) = stream.partition(|data| matches!(data, TestData::Person(_)));

    // Merge them back - will maintain temporal order
    let mut merged = persons.ordered_merge(vec![non_persons]);

    // Act
    tx.send((person_alice(), 1).into())?;
    tx.send((animal_dog(), 2).into())?;
    tx.send((person_bob(), 3).into())?;
    tx.send((animal_cat(), 4).into())?;
    drop(tx);

    // Assert - items arrive in original order
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut merged, 500).await)).into_inner(),
        person_alice()
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut merged, 500).await)).into_inner(),
        animal_dog()
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut merged, 500).await)).into_inner(),
        person_bob()
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut merged, 500).await)).into_inner(),
        animal_cat()
    );

    Ok(())
}

#[tokio::test]
async fn test_partition_then_skip_items() -> anyhow::Result<()> {
    // Arrange - partition by type, then skip first N items in each partition
    let (tx, stream) = test_channel();
    let (persons, animals) = stream.partition(|data| matches!(data, TestData::Person(_)));

    let mut persons_skip1 = persons.skip_items(1);
    let mut animals_skip1 = animals.skip_items(1);

    // Act
    tx.send(Sequenced::new(person_alice()))?; // skipped
    tx.send(Sequenced::new(animal_dog()))?; // skipped
    tx.send(Sequenced::new(person_bob()))?; // kept
    tx.send(Sequenced::new(animal_cat()))?; // kept
    tx.send(Sequenced::new(person_charlie()))?; // kept

    // Assert
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut persons_skip1, 500).await)).value,
        &person_bob()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut animals_skip1, 500).await)).value,
        &animal_cat()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut persons_skip1, 500).await)).value,
        &person_charlie()
    );

    Ok(())
}

#[tokio::test]
async fn test_partition_then_take_items() -> anyhow::Result<()> {
    // Arrange - partition by type, then take first N items from each
    let (tx, stream) = test_channel();
    let (persons, animals) = stream.partition(|data| matches!(data, TestData::Person(_)));

    let mut persons_take2 = persons.take_items(2);
    let mut animals_take1 = animals.take_items(1);

    // Act
    tx.send(Sequenced::new(person_alice()))?;
    tx.send(Sequenced::new(animal_dog()))?;
    tx.send(Sequenced::new(person_bob()))?;
    tx.send(Sequenced::new(animal_cat()))?;
    tx.send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut animals_take1, 500).await)).value,
        &animal_dog()
    );
    assert_stream_ended(&mut animals_take1, 500).await;

    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut persons_take2, 500).await)).value,
        &person_alice()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut persons_take2, 500).await)).value,
        &person_bob()
    );
    assert_stream_ended(&mut persons_take2, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_partition_then_scan_ordered() -> anyhow::Result<()> {
    // Arrange - partition by type, then accumulate ages/legs in each partition
    let (tx, stream) = test_channel();
    let (persons, animals) = stream.partition(|data| matches!(data, TestData::Person(_)));

    // Sum person ages
    let mut age_sum = persons.scan_ordered(0u32, |acc: &mut u32, data: &TestData| {
        if let TestData::Person(p) = data {
            *acc += p.age;
        }
        *acc
    });

    // Sum animal legs
    let mut legs_sum = animals.scan_ordered(0u32, |acc: &mut u32, data: &TestData| {
        if let TestData::Animal(a) = data {
            *acc += a.legs;
        }
        *acc
    });

    // Act
    tx.send(Sequenced::new(person_alice()))?; // age 25, sum: 25
    tx.send(Sequenced::new(animal_dog()))?; // 4 legs, sum: 4
    tx.send(Sequenced::new(person_bob()))?; // age 30, sum: 55
    tx.send(Sequenced::new(animal_bird()))?; // 2 legs, sum: 6

    // Assert
    assert_eq!(
        unwrap_value(Some(
            unwrap_stream::<Sequenced<u32>, _>(&mut age_sum, 500).await
        ))
        .into_inner(),
        25
    );
    assert_eq!(
        unwrap_value(Some(
            unwrap_stream::<Sequenced<u32>, _>(&mut legs_sum, 500).await
        ))
        .into_inner(),
        4
    );
    assert_eq!(
        unwrap_value(Some(
            unwrap_stream::<Sequenced<u32>, _>(&mut age_sum, 500).await
        ))
        .into_inner(),
        55
    );
    assert_eq!(
        unwrap_value(Some(
            unwrap_stream::<Sequenced<u32>, _>(&mut legs_sum, 500).await
        ))
        .into_inner(),
        6
    );

    Ok(())
}

#[tokio::test]
async fn test_partition_then_distinct_until_changed() -> anyhow::Result<()> {
    // Arrange - partition by type, then deduplicate consecutive values
    let (tx, stream) = test_channel();
    let (persons, animals) = stream.partition(|data| matches!(data, TestData::Person(_)));

    let mut persons_distinct = persons.distinct_until_changed();
    let mut animals_distinct = animals.distinct_until_changed();

    // Act
    tx.send(Sequenced::new(person_alice()))?;
    tx.send(Sequenced::new(animal_dog()))?;
    tx.send(Sequenced::new(person_alice()))?; // duplicate person - suppressed
    tx.send(Sequenced::new(animal_dog()))?; // duplicate animal - suppressed
    tx.send(Sequenced::new(person_bob()))?; // new person
    tx.send(Sequenced::new(animal_cat()))?; // new animal

    // Assert
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut persons_distinct, 500).await)).value,
        &person_alice()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut animals_distinct, 500).await)).value,
        &animal_dog()
    );
    // Duplicates are skipped
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut persons_distinct, 500).await)).value,
        &person_bob()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut animals_distinct, 500).await)).value,
        &animal_cat()
    );

    Ok(())
}

#[tokio::test]
async fn test_share_then_partition() -> anyhow::Result<()> {
    // Arrange - share a stream, then partition from the shared source
    let (tx, rx) = test_channel();
    let shared = rx.share();

    // Two independent partitions from shared source
    let sub1 = shared.subscribe().unwrap();
    let (mut adults1, mut young1) = sub1.partition(|data| match data {
        TestData::Person(p) => p.age >= 30,
        _ => false,
    });

    let sub2 = shared.subscribe().unwrap();
    let (mut persons2, _non_persons2) = sub2.partition(|data| matches!(data, TestData::Person(_)));

    // Act
    tx.send(Sequenced::new(person_alice()))?; // age 25 - young, person
    tx.send(Sequenced::new(person_charlie()))?; // age 35 - adult, person

    // Assert - both partitions receive from shared source
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut young1, 500).await)).value,
        &person_alice()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut persons2, 500).await)).value,
        &person_alice()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut adults1, 500).await)).value,
        &person_charlie()
    );

    Ok(())
}
