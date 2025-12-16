// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::prelude::*;
use fluxion_test_utils::{
    assert_stream_ended,
    helpers::unwrap_stream,
    test_channel_with_errors,
    test_data::{animal_dog, person_alice, person_bob, TestData},
    unwrap_value, Sequenced,
};

#[tokio::test]
async fn test_partition_then_filter_ordered_error_propagation() -> anyhow::Result<()> {
    // Arrange - partition by type, then filter each partition
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (persons, animals) = stream.partition(|data| matches!(data, TestData::Person(_)));

    // Filter persons for age > 20, filter animals for legs > 2
    let mut filtered_persons = persons.filter_ordered(|data| match data {
        TestData::Person(p) => p.age > 20,
        _ => false,
    });
    let mut filtered_animals = animals.filter_ordered(|data| match data {
        TestData::Animal(a) => a.legs > 2,
        _ => false,
    });

    // Act - send values then error
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?; // age 25
    tx.send(StreamItem::Value(Sequenced::new(animal_dog())))?; // 4 legs
    tx.send(StreamItem::Error(FluxionError::stream_error("test error")))?;

    // Assert - values arrive before error
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut filtered_persons, 500).await)).value,
        &person_alice()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut filtered_animals, 500).await)).value,
        &animal_dog()
    );

    // Error propagates to both filtered streams
    assert!(matches!(
        unwrap_stream(&mut filtered_persons, 500).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut filtered_animals, 500).await,
        StreamItem::Error(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_partition_then_map_ordered_error_propagation() -> anyhow::Result<()> {
    // Arrange - partition by type, then extract names/species
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
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
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    tx.send(StreamItem::Value(Sequenced::new(animal_dog())))?;
    tx.send(StreamItem::Error(FluxionError::stream_error("map error")))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut person_names, 500).await)).into_inner(),
        "Alice"
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut animal_species, 500).await)).into_inner(),
        "Dog"
    );

    // Error propagates through map
    assert!(matches!(
        unwrap_stream(&mut person_names, 500).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut animal_species, 500).await,
        StreamItem::Error(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_filter_then_partition_error_propagation() -> anyhow::Result<()> {
    // Arrange - filter for persons only, then partition by age
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let persons_only = stream.filter_ordered(|data| matches!(data, TestData::Person(_)));
    let (mut adults, mut young) = persons_only.partition(|data| match data {
        TestData::Person(p) => p.age >= 30,
        _ => false,
    });

    // Act
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?; // age 25 - young
    tx.send(StreamItem::Value(Sequenced::new(person_bob())))?; // age 30 - adult
    tx.send(StreamItem::Error(FluxionError::stream_error(
        "filter chain error",
    )))?;

    // Assert - values arrive
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut young, 500).await)).value,
        &person_alice()
    );
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut adults, 500).await)).value,
        &person_bob()
    );

    // Error propagates from filter through partition to both outputs
    assert!(matches!(
        unwrap_stream(&mut adults, 500).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut young, 500).await,
        StreamItem::Error(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_map_then_partition_error_propagation() -> anyhow::Result<()> {
    // Arrange - extract ages from persons, then partition by threshold
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let ages = stream.map_ordered(|s: Sequenced<TestData>| match s.into_inner() {
        TestData::Person(p) => Sequenced::new(p.age),
        _ => Sequenced::new(0),
    });
    let (mut adults, mut young) = ages.partition(|&age| age >= 30);

    // Act
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?; // age 25 - young
    tx.send(StreamItem::Value(Sequenced::new(person_bob())))?; // age 30 - adult
    tx.send(StreamItem::Error(FluxionError::stream_error(
        "map chain error",
    )))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut young, 500).await)).into_inner(),
        25
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut adults, 500).await)).into_inner(),
        30
    );

    // Error propagates
    assert!(matches!(
        unwrap_stream(&mut young, 500).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut adults, 500).await,
        StreamItem::Error(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_partition_then_scan_ordered_error_propagation() -> anyhow::Result<()> {
    // Arrange - partition by type, then accumulate
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
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
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?; // age 25
    tx.send(StreamItem::Value(Sequenced::new(animal_dog())))?; // 4 legs
    tx.send(StreamItem::Error(FluxionError::stream_error("scan error")))?;

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

    // Error propagates through scan
    assert!(matches!(
        unwrap_stream(&mut age_sum, 500).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut legs_sum, 500).await,
        StreamItem::Error(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_partition_error_terminates_downstream_chains() -> anyhow::Result<()> {
    // Arrange - complex chain: partition -> map -> filter
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (persons, animals) = stream.partition(|data| matches!(data, TestData::Person(_)));

    // Extract person ages, filter for adults
    let mut persons_chain = persons
        .map_ordered(|s: Sequenced<TestData>| match s.into_inner() {
            TestData::Person(p) => Sequenced::new(p.age),
            _ => Sequenced::new(0),
        })
        .filter_ordered(|&age| age >= 25);

    // Extract animal legs, filter for > 2 legs
    let mut animals_chain = animals
        .map_ordered(|s: Sequenced<TestData>| match s.into_inner() {
            TestData::Animal(a) => Sequenced::new(a.legs),
            _ => Sequenced::new(0),
        })
        .filter_ordered(|&legs| legs > 2);

    // Act
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?; // age 25
    tx.send(StreamItem::Value(Sequenced::new(animal_dog())))?; // 4 legs
    tx.send(StreamItem::Error(FluxionError::stream_error(
        "chain termination",
    )))?;

    // Assert - values flow through chains
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut persons_chain, 500).await)).into_inner(),
        25
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut animals_chain, 500).await)).into_inner(),
        4
    );

    // Error terminates both chains
    assert!(matches!(
        unwrap_stream(&mut persons_chain, 500).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut animals_chain, 500).await,
        StreamItem::Error(_)
    ));

    // Both chains are ended
    assert_stream_ended(&mut persons_chain, 500).await;
    assert_stream_ended(&mut animals_chain, 500).await;

    Ok(())
}
