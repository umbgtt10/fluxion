// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::Timestamped;
use fluxion_stream_multi::prelude::*;
use fluxion_stream_multi::CombinedState;
use fluxion_test_utils::{
    assert_no_element_emitted, assert_stream_ended,
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
    tx.unbounded_send(Sequenced::new(person_alice()))?; // age 25 - filtered out
    tx.unbounded_send(Sequenced::new(person_bob()))?; // age 30 - kept
    tx.unbounded_send(Sequenced::new(animal_dog()))?; // animal - kept
    tx.unbounded_send(Sequenced::new(plant_rose()))?; // plant - filtered out
    tx.unbounded_send(Sequenced::new(person_charlie()))?; // age 35 - kept

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
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(animal_cat()))?;

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
    tx.unbounded_send(Sequenced::new(person_alice()))?; // 25 - young adult
    tx.unbounded_send(Sequenced::new(person_charlie()))?; // 35 - senior
    tx.unbounded_send(Sequenced::new(person_dave()))?; // 28 - young adult
    tx.unbounded_send(Sequenced::new(person_diane()))?; // 40 - senior

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
    tx.unbounded_send(Sequenced::new(animal_bird()))?; // 2 legs - few
    tx.unbounded_send(Sequenced::new(animal_dog()))?; // 4 legs - many
    tx.unbounded_send(Sequenced::new(animal_spider()))?; // 8 legs - many

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
    tx.unbounded_send((person_alice(), 1).into())?;
    tx.unbounded_send((animal_dog(), 2).into())?;
    tx.unbounded_send((person_bob(), 3).into())?;
    tx.unbounded_send((animal_cat(), 4).into())?;
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
    tx.unbounded_send(Sequenced::new(person_alice()))?; // skipped
    tx.unbounded_send(Sequenced::new(animal_dog()))?; // skipped
    tx.unbounded_send(Sequenced::new(person_bob()))?; // kept
    tx.unbounded_send(Sequenced::new(animal_cat()))?; // kept
    tx.unbounded_send(Sequenced::new(person_charlie()))?; // kept

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
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(animal_cat()))?;
    tx.unbounded_send(Sequenced::new(person_charlie()))?;

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
    tx.unbounded_send(Sequenced::new(person_alice()))?; // age 25, sum: 25
    tx.unbounded_send(Sequenced::new(animal_dog()))?; // 4 legs, sum: 4
    tx.unbounded_send(Sequenced::new(person_bob()))?; // age 30, sum: 55
    tx.unbounded_send(Sequenced::new(animal_bird()))?; // 2 legs, sum: 6

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
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    tx.unbounded_send(Sequenced::new(person_alice()))?; // duplicate person - suppressed
    tx.unbounded_send(Sequenced::new(animal_dog()))?; // duplicate animal - suppressed
    tx.unbounded_send(Sequenced::new(person_bob()))?; // new person
    tx.unbounded_send(Sequenced::new(animal_cat()))?; // new animal

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
    tx.unbounded_send(Sequenced::new(person_alice()))?; // age 25 - young, person
    tx.unbounded_send(Sequenced::new(person_charlie()))?; // age 35 - adult, person

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

#[tokio::test]
async fn test_partition_then_combine_latest() -> anyhow::Result<()> {
    // Arrange - partition persons and animals, then combine_latest between them
    let (tx, stream) = test_channel();
    let (persons, animals) = stream.partition(|data| matches!(data, TestData::Person(_)));

    // combine_latest requires all streams to have emitted before producing output
    let mut combined = persons.combine_latest(vec![animals], |_| true);

    // Act - send one of each to satisfy combine_latest requirements
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Assert - should have both values combined
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut combined, 500).await))
            .into_inner()
            .values(),
        &[person_alice(), animal_dog()]
    );

    // Act - update person, combined should emit with latest animal
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut combined, 500).await))
            .into_inner()
            .values(),
        &[person_bob(), animal_dog()]
    );

    // Act - update animal, combined should emit with latest person
    tx.unbounded_send(Sequenced::new(animal_cat()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut combined, 500).await))
            .into_inner()
            .values(),
        &[person_bob(), animal_cat()]
    );

    Ok(())
}

#[tokio::test]
async fn test_partition_with_subject_and_filter() -> anyhow::Result<()> {
    use fluxion_core::{FluxionSubject, StreamItem};

    // Arrange - use a subject as the source, partition, then filter
    let subject: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();

    let stream = subject.subscribe().unwrap();
    let (persons, animals) = stream.partition(|data| matches!(data, TestData::Person(_)));

    // Filter persons for age > 30
    let mut older_persons = persons.filter_ordered(|data| match data {
        TestData::Person(p) => p.age > 30,
        _ => false,
    });

    // Filter animals for legs >= 4
    let mut four_legged = animals.filter_ordered(|data| match data {
        TestData::Animal(a) => a.legs >= 4,
        _ => false,
    });

    // Act - send through subject
    subject.send(StreamItem::Value(Sequenced::new(person_alice())))?; // age 25 - filtered
    subject.send(StreamItem::Value(Sequenced::new(animal_bird())))?; // 2 legs - filtered
    subject.send(StreamItem::Value(Sequenced::new(person_charlie())))?; // age 35 - kept
    subject.send(StreamItem::Value(Sequenced::new(animal_dog())))?; // 4 legs - kept
    subject.send(StreamItem::Value(Sequenced::new(person_diane())))?; // age 40 - kept
    subject.send(StreamItem::Value(Sequenced::new(animal_spider())))?; // 8 legs - kept

    // Assert
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut older_persons, 500).await)).value,
        &person_charlie()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut four_legged, 500).await)).value,
        &animal_dog()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut older_persons, 500).await)).value,
        &person_diane()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut four_legged, 500).await)).value,
        &animal_spider()
    );

    Ok(())
}

#[tokio::test]
async fn test_partition_share_then_combine_latest() -> anyhow::Result<()> {
    // Arrange - share a stream, partition from one sub, use another sub with combine_latest
    let (tx, rx) = test_channel();
    let shared = rx.share();

    // Subscriber 1: partition into persons and non-persons
    let sub1 = shared.subscribe().unwrap();
    let (persons1, _non_persons1) = sub1.partition(|data| matches!(data, TestData::Person(_)));

    // Subscriber 2: just filter for animals
    let sub2 = shared.subscribe().unwrap();
    let animals2 = sub2.filter_ordered(|data| matches!(data, TestData::Animal(_)));

    // combine_latest between partitioned persons and filtered animals
    let mut combined = persons1.combine_latest(vec![animals2], |_| true);

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Assert - both streams need to emit before combine_latest produces
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut combined, 500).await))
            .clone()
            .into_inner()
            .values(),
        [person_alice(), animal_dog()]
    );

    // Act - update both
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut combined, 500).await))
            .clone()
            .into_inner()
            .values(),
        [person_bob(), animal_dog()]
    );

    tx.unbounded_send(Sequenced::new(animal_cat()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut combined, 500).await))
            .clone()
            .into_inner()
            .values(),
        [person_bob(), animal_cat()]
    );

    Ok(())
}

#[tokio::test]
async fn test_partition_then_with_latest_from() -> anyhow::Result<()> {
    // Arrange - partition by type, use with_latest_from to sample animals when persons emit
    let (tx, stream) = test_channel();
    let (persons, animals) = stream.partition(|data| matches!(data, TestData::Person(_)));

    // When a person emits, sample the latest animal
    let mut person_with_animal = persons
        .with_latest_from(animals, |state: &CombinedState<TestData, u64>| {
            state.clone()
        });

    // Act - need animal first (secondary), then person (primary) triggers
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut person_with_animal, 500).await))
            .clone()
            .into_inner()
            .values(),
        [person_alice(), animal_dog()]
    );

    // Act - animal update alone shouldn't emit (only primary triggers)
    tx.unbounded_send(Sequenced::new(animal_cat()))?;
    assert_no_element_emitted(&mut person_with_animal, 100).await;

    // Act - person update triggers with latest animal
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut person_with_animal, 500).await))
            .clone()
            .into_inner()
            .values(),
        [person_bob(), animal_cat()]
    );

    Ok(())
}

#[tokio::test]
async fn test_partition_then_combine_with_previous() -> anyhow::Result<()> {
    // Arrange - partition by type, then track previous values in each partition
    let (tx, stream) = test_channel();
    let (persons, animals) = stream.partition(|data| matches!(data, TestData::Person(_)));

    let mut persons_with_prev = persons.combine_with_previous();
    let mut animals_with_prev = animals.combine_with_previous();

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(animal_cat()))?;

    // Assert - first emissions have no previous
    let item1 = unwrap_value(Some(unwrap_stream(&mut persons_with_prev, 500).await));
    assert!(item1.current.value == person_alice() && item1.previous.is_none());

    let item2 = unwrap_value(Some(unwrap_stream(&mut animals_with_prev, 500).await));
    assert!(item2.current.value == animal_dog() && item2.previous.is_none());

    // Second emissions have previous
    let item3 = unwrap_value(Some(unwrap_stream(&mut persons_with_prev, 500).await));
    assert!(item3.current.value == person_bob() && item3.previous.unwrap().value == person_alice());

    let item4 = unwrap_value(Some(unwrap_stream(&mut animals_with_prev, 500).await));
    assert!(item4.current.value == animal_cat() && item4.previous.unwrap().value == animal_dog());

    Ok(())
}
