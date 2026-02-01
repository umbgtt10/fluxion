// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::FluxionSubject;
use fluxion_core::StreamItem;
use fluxion_core::Timestamped;
use fluxion_stream::prelude::*;
use fluxion_stream::CombinedState;
use fluxion_test_utils::{
    helpers::{
        assert_no_element_emitted, assert_stream_ended, test_channel, unwrap_stream, unwrap_value,
    },
    sequenced::Sequenced,
    test_data::{
        animal_bird, animal_cat, animal_dog, animal_spider, person_alice, person_bob,
        person_charlie, person_dave, person_diane, plant_rose, TestData,
    },
};

#[tokio::test]
async fn test_partition_then_filter_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel();
    let (persons, non_persons) = stream.partition(|data| matches!(data, TestData::Person(_)));

    let mut older_persons = persons.filter_ordered(|data| match data {
        TestData::Person(p) => p.age > 25,
        _ => false,
    });

    let mut animals = non_persons.filter_ordered(|data| matches!(data, TestData::Animal(_)));

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    tx.unbounded_send(Sequenced::new(plant_rose()))?;
    tx.unbounded_send(Sequenced::new(person_charlie()))?;

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
    // Arrange
    let (tx, stream) = test_channel();
    let (persons, animals) = stream.partition(|data| matches!(data, TestData::Person(_)));

    let mut person_names = persons.map_ordered(|s: Sequenced<TestData>| match s.into_inner() {
        TestData::Person(p) => Sequenced::new(p.name),
        _ => Sequenced::new(String::new()),
    });

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
    // Arrange
    let (tx, stream) = test_channel();

    let adults = stream.filter_ordered(|data| match data {
        TestData::Person(p) => p.age >= 18,
        _ => false,
    });
    let (mut seniors, mut young_adults) = adults.partition(|data| match data {
        TestData::Person(p) => p.age >= 35,
        _ => false,
    });

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_charlie()))?;
    tx.unbounded_send(Sequenced::new(person_dave()))?;
    tx.unbounded_send(Sequenced::new(person_diane()))?;

    // Assert
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
    // Arrange
    let (tx, stream) = test_channel();

    let leg_counts = stream.map_ordered(|s: Sequenced<TestData>| match s.into_inner() {
        TestData::Animal(a) => Sequenced::new(a.legs),
        _ => Sequenced::new(0),
    });
    let (mut many_legs, mut few_legs) = leg_counts.partition(|&legs| legs >= 4);

    // Act
    tx.unbounded_send(Sequenced::new(animal_bird()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    tx.unbounded_send(Sequenced::new(animal_spider()))?;

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
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let (persons, non_persons) = stream.partition(|data| matches!(data, TestData::Person(_)));

    let mut merged = persons.ordered_merge(vec![non_persons]);

    // Act
    tx.unbounded_send((person_alice(), 1).into())?;
    tx.unbounded_send((animal_dog(), 2).into())?;
    tx.unbounded_send((person_bob(), 3).into())?;
    tx.unbounded_send((animal_cat(), 4).into())?;
    drop(tx);

    // Assert
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
    // Arrange
    let (tx, stream) = test_channel();
    let (persons, animals) = stream.partition(|data| matches!(data, TestData::Person(_)));

    let mut persons_skip1 = persons.skip_items(1);
    let mut animals_skip1 = animals.skip_items(1);

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(animal_cat()))?;
    tx.unbounded_send(Sequenced::new(person_charlie()))?;

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
    // Arrange
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
    // Arrange
    let (tx, stream) = test_channel();
    let (persons, animals) = stream.partition(|data| matches!(data, TestData::Person(_)));

    let mut age_sum = persons.scan_ordered(0u32, |acc: &mut u32, data: &TestData| {
        if let TestData::Person(p) = data {
            *acc += p.age;
        }
        *acc
    });

    let mut legs_sum = animals.scan_ordered(0u32, |acc: &mut u32, data: &TestData| {
        if let TestData::Animal(a) = data {
            *acc += a.legs;
        }
        *acc
    });

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(animal_bird()))?;

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
    // Arrange
    let (tx, stream) = test_channel();
    let (persons, animals) = stream.partition(|data| matches!(data, TestData::Person(_)));

    let mut persons_distinct = persons.distinct_until_changed();
    let mut animals_distinct = animals.distinct_until_changed();

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(animal_cat()))?;

    // Assert
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut persons_distinct, 500).await)).value,
        &person_alice()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut animals_distinct, 500).await)).value,
        &animal_dog()
    );
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
    // Arrange
    let (tx, rx) = test_channel();
    let shared = rx.share();

    let sub1 = shared.subscribe().unwrap();
    let (mut adults1, mut young1) = sub1.partition(|data| match data {
        TestData::Person(p) => p.age >= 30,
        _ => false,
    });

    let sub2 = shared.subscribe().unwrap();
    let (mut persons2, _non_persons2) = sub2.partition(|data| matches!(data, TestData::Person(_)));

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
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
    // Arrange
    let (tx, stream) = test_channel();
    let (persons, animals) = stream.partition(|data| matches!(data, TestData::Person(_)));

    let mut combined = persons.combine_latest(vec![animals], |_| true);

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut combined, 500).await))
            .into_inner()
            .values(),
        &[person_alice(), animal_dog()]
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut combined, 500).await))
            .into_inner()
            .values(),
        &[person_bob(), animal_dog()]
    );

    // Act
    tx.unbounded_send(Sequenced::new(animal_cat()))?;

    // Assert
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
    // Arrange
    let subject: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();
    let stream = subject.subscribe().unwrap();
    let (persons, animals) = stream.partition(|data| matches!(data, TestData::Person(_)));

    let mut older_persons = persons.filter_ordered(|data| match data {
        TestData::Person(p) => p.age > 30,
        _ => false,
    });

    let mut four_legged = animals.filter_ordered(|data| match data {
        TestData::Animal(a) => a.legs >= 4,
        _ => false,
    });

    // Act
    subject.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    subject.send(StreamItem::Value(Sequenced::new(animal_bird())))?;
    subject.send(StreamItem::Value(Sequenced::new(person_charlie())))?;
    subject.send(StreamItem::Value(Sequenced::new(animal_dog())))?;
    subject.send(StreamItem::Value(Sequenced::new(person_diane())))?;
    subject.send(StreamItem::Value(Sequenced::new(animal_spider())))?;

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
    //
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

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut combined, 500).await))
            .clone()
            .into_inner()
            .values(),
        [person_alice(), animal_dog()]
    );

    // Act
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
    let (tx, stream) = test_channel();
    let (persons, animals) = stream.partition(|data| matches!(data, TestData::Person(_)));

    let mut person_with_animal = persons
        .with_latest_from(animals, |state: &CombinedState<TestData, u64>| {
            state.clone()
        });

    // Act
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

    // Act
    tx.unbounded_send(Sequenced::new(animal_cat()))?;
    assert_no_element_emitted(&mut person_with_animal, 100).await;

    // Act
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
    let (tx, stream) = test_channel();
    let (persons, animals) = stream.partition(|data| matches!(data, TestData::Person(_)));

    let mut persons_with_prev = persons.combine_with_previous();
    let mut animals_with_prev = animals.combine_with_previous();

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(animal_cat()))?;

    // Assert
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
