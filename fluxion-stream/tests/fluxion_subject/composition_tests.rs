// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionSubject, HasTimestamp, StreamItem, Timestamped};
use fluxion_stream::merge_with::MergedStream;
use fluxion_stream::FluxionStream;
use fluxion_test_utils::animal::Animal;
use fluxion_test_utils::person::Person;
use fluxion_test_utils::plant::Plant;
use fluxion_test_utils::test_data::{
    animal_ant, animal_bird, animal_cat, animal_dog, animal_spider, person_alice, person_bob,
    person_charlie, person_diane, plant_fern, plant_rose, plant_sunflower, TestData,
};
use fluxion_test_utils::{assert_no_element_emitted, test_channel, unwrap_stream, Sequenced};

#[tokio::test]
async fn subject_at_start_map_and_filter() -> anyhow::Result<()> {
    // Arrange
    let subject: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();
    let (other_tx, other_rx) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(subject.subscribe().unwrap())
        .combine_latest(vec![other_rx], |_| true)
        .map_ordered(|combined| {
            let person = combined.values()[0].clone();
            let updated = match person {
                TestData::Person(p) => TestData::Person(Person::new(p.name, p.age + 5)),
                other => other,
            };
            Sequenced::new(updated)
        })
        .filter_ordered(|data| matches!(data, TestData::Person(person) if person.age > 30));

    // Act
    subject.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    subject.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    subject.send(StreamItem::Value(Sequenced::new(person_charlie())))?;
    other_tx.send(Sequenced::new(person_alice()))?;
    assert!(matches!(
        unwrap_stream(&mut stream, 200).await.unwrap().into_inner(),
        TestData::Person(ref person) if person.name == "Charlie" && person.age == 40
    ));

    // Assert
    other_tx.send(Sequenced::new(person_bob()))?;
    assert!(matches!(
        unwrap_stream(&mut stream, 200).await.unwrap().into_inner(),
        TestData::Person(ref person) if person.name == "Charlie" && person.age == 40
    ));

    other_tx.send(Sequenced::new(person_charlie()))?;
    assert!(matches!(
        unwrap_stream(&mut stream, 200).await.unwrap().into_inner(),
        TestData::Person(ref person) if person.name == "Charlie" && person.age == 40
    ));

    assert_no_element_emitted(&mut stream, 100).await;
    Ok(())
}

#[tokio::test]
async fn subject_combined_with_stream_via_combine_latest() -> anyhow::Result<()> {
    // Arrange
    let (primary_tx, primary_rx) = test_channel::<Sequenced<TestData>>();
    let gate_subject: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();

    let mut combined = FluxionStream::new(primary_rx)
        .combine_latest(vec![gate_subject.subscribe().unwrap()], |_| true);

    // Act
    gate_subject.send(StreamItem::Value(Sequenced::new(plant_sunflower())))?;
    primary_tx.send(Sequenced::new(person_alice()))?; // first combined output

    // Assert first emission
    let first = unwrap_stream(&mut combined, 200)
        .await
        .unwrap()
        .into_inner();
    let vals = first.values();
    assert!(
        matches!(vals[0], TestData::Person(ref p) if p.name == "Alice")
            && matches!(vals[1], TestData::Plant(ref plant) if plant.species == "Sunflower")
    );

    // Assert second emission
    primary_tx.send(Sequenced::new(person_bob()))?; // second combined output
    let second = unwrap_stream(&mut combined, 200)
        .await
        .unwrap()
        .into_inner();
    let vals = second.values();
    assert!(
        matches!(vals[0], TestData::Person(ref p) if p.name == "Bob")
            && matches!(vals[1], TestData::Plant(ref plant) if plant.species == "Sunflower")
    );
    Ok(())
}

#[tokio::test]
async fn subject_in_middle_take_latest_when() -> anyhow::Result<()> {
    // Arrange
    let (src_tx, src_rx) = test_channel::<Sequenced<TestData>>();
    let gate_subject: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();

    let source = FluxionStream::new(src_rx);
    let gate = gate_subject.subscribe().unwrap();

    let mut stream = source.take_latest_when(
        gate,
        |signal| matches!(signal, TestData::Animal(animal) if animal.legs >= 4),
    );

    // Act
    src_tx.send(Sequenced::new(person_alice()))?;
    gate_subject.send(StreamItem::Value(Sequenced::new(animal_spider())))?;

    // Assert
    assert!(
        matches!(unwrap_stream(&mut stream, 200).await.unwrap().into_inner(), TestData::Person(ref person) if person.name == "Alice")
    );

    // Act
    src_tx.send(Sequenced::new(person_bob()))?;
    gate_subject.send(StreamItem::Value(Sequenced::new(animal_bird())))?;
    assert_no_element_emitted(&mut stream, 100).await;

    gate_subject.send(StreamItem::Value(Sequenced::new(animal_dog())))?;

    // Assert
    assert!(
        matches!(unwrap_stream(&mut stream, 200).await.unwrap().into_inner(), TestData::Person(ref person) if person.name == "Bob")
    );
    Ok(())
}

#[tokio::test]
async fn subject_combines_with_latest_from() -> anyhow::Result<()> {
    // Arrange
    let (primary_tx, primary_rx) = test_channel::<Sequenced<TestData>>();
    let secondary_subject: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();

    let primary = FluxionStream::new(primary_rx);

    let mut stream = primary.with_latest_from(secondary_subject.subscribe().unwrap(), |state| {
        let values = state.values();
        let age = person_age(&values[0]);
        let legs = animal_legs(&values[1]);
        Sequenced::new(age * legs)
    });

    // Act
    secondary_subject.send(StreamItem::Value(Sequenced::new(animal_cat())))?;
    primary_tx.send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut stream, 200).await.unwrap().into_inner(),
        140
    );
    Ok(())
}

#[tokio::test]
async fn subject_as_filter_for_emit_when() -> anyhow::Result<()> {
    // Arrange
    let (src_tx, src_rx) = test_channel::<Sequenced<TestData>>();
    let filter_subject: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();

    let mut stream = FluxionStream::new(src_rx).emit_when(
        filter_subject.subscribe().unwrap(),
        |state| matches!(state.values()[1], TestData::Plant(ref plant) if plant.height >= 100),
    );

    // Act
    src_tx.send(Sequenced::new(person_alice()))?;
    filter_subject.send(StreamItem::Value(Sequenced::new(plant_rose())))?;
    assert_no_element_emitted(&mut stream, 100).await;

    filter_subject.send(StreamItem::Value(Sequenced::new(plant_sunflower())))?;

    // Assert
    assert!(
        matches!(unwrap_stream(&mut stream, 200).await.unwrap().into_inner(), TestData::Person(ref person) if person.name == "Alice")
    );

    // Act
    src_tx.send(Sequenced::new(person_bob()))?;
    filter_subject.send(StreamItem::Value(Sequenced::new(plant_fern())))?;

    // Assert
    assert!(
        matches!(unwrap_stream(&mut stream, 200).await.unwrap().into_inner(), TestData::Person(ref person) if person.name == "Bob")
    );
    Ok(())
}

#[tokio::test]
async fn subject_chain_with_filter_and_map() -> anyhow::Result<()> {
    // Arrange
    let subject: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();
    let mut stream = FluxionStream::new(subject.subscribe().unwrap())
        .filter_ordered(|data| {
            matches!(
                data,
                TestData::Animal(animal) if animal.legs % 2 == 0 && animal.legs >= 4
            )
        })
        .map_ordered(|item| {
            let ts = item.timestamp();
            let mapped = match item.into_inner() {
                TestData::Animal(animal) => {
                    TestData::Animal(Animal::new(animal.species, animal.legs * 2))
                }
                other => other,
            };
            Sequenced::with_timestamp(mapped, ts)
        });

    // Act
    subject.send(StreamItem::Value(Sequenced::new(animal_spider())))?;
    subject.send(StreamItem::Value(Sequenced::new(animal_bird())))?;
    subject.send(StreamItem::Value(Sequenced::new(animal_cat())))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut stream, 200).await.unwrap().into_inner(),
        TestData::Animal(ref animal) if animal.species == "Spider" && animal.legs == 16
    ));

    assert!(matches!(
        unwrap_stream(&mut stream, 200).await.unwrap().into_inner(),
        TestData::Animal(ref animal) if animal.species == "Cat" && animal.legs == 8
    ));
    Ok(())
}

#[tokio::test]
async fn subject_with_previous_computes_age_deltas() -> anyhow::Result<()> {
    let subject: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();

    let mut deltas = FluxionStream::new(subject.subscribe().unwrap())
        .combine_with_previous()
        .map_ordered(|pair| {
            let ts = pair.current.timestamp();
            let current_age = match pair.current.into_inner() {
                TestData::Person(p) => p.age,
                _ => 0,
            };
            let prev_age = match pair.previous.as_ref() {
                Some(prev) => match prev.clone().into_inner() {
                    TestData::Person(p) => p.age,
                    _ => 0,
                },
                None => 0,
            };
            Sequenced::with_timestamp(current_age - prev_age, ts)
        });

    subject.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    subject.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    subject.send(StreamItem::Value(Sequenced::new(person_diane())))?;

    // First emission has no previous, delta is 0 - discard
    let _ = unwrap_stream(&mut deltas, 200).await;
    assert_eq!(
        unwrap_stream(&mut deltas, 200).await.unwrap().into_inner(),
        5
    );
    assert_eq!(
        unwrap_stream(&mut deltas, 200).await.unwrap().into_inner(),
        10
    );
    Ok(())
}

#[tokio::test]
async fn subject_take_while_with_stops_on_short_plants() -> anyhow::Result<()> {
    let (src_tx, src_rx) = test_channel::<Sequenced<TestData>>();
    let gate: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();

    let mut stream = FluxionStream::new(src_rx).take_while_with(
        gate.subscribe().unwrap(),
        |plant| matches!(plant, TestData::Plant(ref p) if p.height >= 100),
    );

    // Initialize gate before first source so emission is allowed
    gate.send(StreamItem::Value(Sequenced::new(plant_sunflower())))?;
    src_tx.send(Sequenced::new(person_alice()))?;
    assert!(matches!(
        unwrap_stream(&mut stream, 200).await.unwrap().into_inner(),
        TestData::Person(ref person) if person.name == "Alice"
    ));

    // Drop gate below threshold, then send next source to trigger termination
    gate.send(StreamItem::Value(Sequenced::new(plant_rose())))?;
    src_tx.send(Sequenced::new(person_bob()))?;

    // After predicate becomes false, stream stays silent; ensure no further output is emitted
    assert_no_element_emitted(&mut stream, 200).await;
    Ok(())
}

#[tokio::test]
async fn subject_merge_with_stateful_height_bonus() -> anyhow::Result<()> {
    // Merge a subject-driven plant stream with a secondary subject to apply a running height bonus.
    let plants: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();
    let bonuses: FluxionSubject<Sequenced<u32>> = FluxionSubject::new();

    let mut merged = MergedStream::seed::<Sequenced<TestData>>(0u32)
        // Apply bonuses first so state is updated before plants are processed
        .merge_with(bonuses.subscribe().unwrap(), |bonus, state| {
            *state = bonus;
            TestData::Plant(Plant::new("BonusMarker".to_string(), *state))
        })
        .merge_with(plants.subscribe().unwrap(), |plant, bonus| match plant {
            TestData::Plant(mut p) => {
                p.height += *bonus;
                TestData::Plant(p)
            }
            other => other,
        })
        .into_fluxion_stream()
        .filter_ordered(
            |item| !matches!(item, TestData::Plant(ref p) if p.species == "BonusMarker"),
        );

    // Use explicit timestamps to enforce merge order: bonus first, then plant
    bonuses.send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;
    plants.send(StreamItem::Value(Sequenced::with_timestamp(
        plant_rose(),
        2,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut merged, 200).await.unwrap().into_inner(),
        TestData::Plant(ref p) if p.species == "Rose" && p.height == 25
    ));

    bonuses.send(StreamItem::Value(Sequenced::with_timestamp(5, 3)))?;
    plants.send(StreamItem::Value(Sequenced::with_timestamp(
        plant_sunflower(),
        4,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut merged, 200).await.unwrap().into_inner(),
        TestData::Plant(ref p) if p.species == "Sunflower" && p.height == 185
    ));
    Ok(())
}

#[tokio::test]
async fn subject_scan_and_distinct_detects_unique_leg_totals() -> anyhow::Result<()> {
    // Combine scan_ordered with distinct_until_changed to emit unique cumulative legs.
    let subject: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();

    let mut uniques = FluxionStream::new(subject.subscribe().unwrap())
        .scan_ordered::<Sequenced<u32>, _, _>(0u32, |acc, data| {
            let legs = match data {
                TestData::Animal(animal) => animal.legs,
                _ => 0,
            };
            *acc += legs;
            *acc
        })
        .distinct_until_changed();

    subject.send(StreamItem::Value(Sequenced::new(animal_dog())))?;
    assert_eq!(
        unwrap_stream(&mut uniques, 200).await.unwrap().into_inner(),
        4
    );

    subject.send(StreamItem::Value(Sequenced::new(animal_ant())))?;
    assert_eq!(
        unwrap_stream(&mut uniques, 200).await.unwrap().into_inner(),
        10
    );

    subject.send(StreamItem::Value(Sequenced::new(animal_ant())))?;
    assert_eq!(
        unwrap_stream(&mut uniques, 200).await.unwrap().into_inner(),
        16
    );

    subject.send(StreamItem::Value(Sequenced::new(animal_ant())))?;
    assert_eq!(
        unwrap_stream(&mut uniques, 200).await.unwrap().into_inner(),
        22
    );

    subject.send(StreamItem::Value(Sequenced::new(animal_spider())))?;
    assert_eq!(
        unwrap_stream(&mut uniques, 200).await.unwrap().into_inner(),
        30
    );

    Ok(())
}

#[tokio::test]
async fn subject_start_with_take_items_preserves_temporal_merge() -> anyhow::Result<()> {
    let plants: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();

    let mut merged = FluxionStream::new(plants.subscribe().unwrap())
        .start_with(vec![
            StreamItem::Value(Sequenced::with_timestamp(plant_rose(), 1)),
            StreamItem::Value(Sequenced::with_timestamp(plant_sunflower(), 2)),
        ])
        .ordered_merge(vec![animal_rx])
        .take_items(3);

    animal_tx.send(Sequenced::with_timestamp(animal_dog(), 3))?;
    assert!(
        matches!(unwrap_stream(&mut merged, 200).await.unwrap().into_inner(), TestData::Plant(ref p) if p.species == "Rose")
    );

    assert!(
        matches!(unwrap_stream(&mut merged, 200).await.unwrap().into_inner(), TestData::Plant(ref p) if p.species == "Sunflower")
    );

    assert!(
        matches!(unwrap_stream(&mut merged, 200).await.unwrap().into_inner(), TestData::Animal(ref a) if a.species == "Dog")
    );
    Ok(())
}

#[tokio::test]
async fn subject_skip_items_with_latest_from_ignores_prelude() -> anyhow::Result<()> {
    let primary: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();
    let context: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();

    let mut stream = FluxionStream::new(primary.subscribe().unwrap())
        .start_with(vec![
            StreamItem::Value(Sequenced::with_timestamp(person_alice(), 1)),
            StreamItem::Value(Sequenced::with_timestamp(person_bob(), 2)),
        ])
        .skip_items(2)
        .with_latest_from(context.subscribe().unwrap(), |state| {
            let values = state.values();
            let age = person_age(&values[0]);
            let legs = animal_legs(&values[1]);
            Sequenced::new(age * legs)
        });

    context.send(StreamItem::Value(Sequenced::with_timestamp(
        animal_dog(),
        3,
    )))?;
    primary.send(StreamItem::Value(Sequenced::with_timestamp(
        person_charlie(),
        4,
    )))?;

    assert_eq!(
        unwrap_stream(&mut stream, 200).await.unwrap().into_inner(),
        140
    );
    Ok(())
}

#[tokio::test]
async fn subject_ordered_merge_distinct_by_species() -> anyhow::Result<()> {
    let plants: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();
    let (other_tx, other_rx) = test_channel::<Sequenced<TestData>>();

    let mut merged = FluxionStream::new(plants.subscribe().unwrap())
        .ordered_merge(vec![other_rx])
        .distinct_until_changed_by(|prev, current| match (prev, current) {
            (TestData::Plant(p1), TestData::Plant(p2)) => p1.species == p2.species,
            _ => false,
        });

    plants.send(StreamItem::Value(Sequenced::with_timestamp(
        plant_rose(),
        1,
    )))?;
    other_tx.send(Sequenced::with_timestamp(plant_rose(), 2))?;
    plants.send(StreamItem::Value(Sequenced::with_timestamp(
        plant_sunflower(),
        3,
    )))?;

    assert!(
        matches!(unwrap_stream(&mut merged, 200).await.unwrap().into_inner(), TestData::Plant(ref p) if p.species == "Rose")
    );

    assert!(
        matches!(unwrap_stream(&mut merged, 200).await.unwrap().into_inner(), TestData::Plant(ref p) if p.species == "Sunflower")
    );
    Ok(())
}

#[tokio::test]
async fn subject_combine_latest_then_take_items_limits_output() -> anyhow::Result<()> {
    let primary: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();
    let (secondary_tx, secondary_rx) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(primary.subscribe().unwrap())
        .combine_latest(vec![secondary_rx], |_| true)
        .take_items(2);

    secondary_tx.send(Sequenced::new(plant_fern()))?;
    primary.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    primary.send(StreamItem::Value(Sequenced::new(person_bob())))?;

    let first = unwrap_stream(&mut stream, 200).await.unwrap().into_inner();
    let vals = first.values();
    assert!(matches!(vals[0], TestData::Person(ref p) if p.name == "Alice"));
    assert!(matches!(vals[1], TestData::Plant(ref p) if p.species == "Fern"));

    let second = unwrap_stream(&mut stream, 200).await.unwrap().into_inner();
    let vals = second.values();
    assert!(matches!(vals[0], TestData::Person(ref p) if p.name == "Bob"));
    assert!(matches!(vals[1], TestData::Plant(ref p) if p.species == "Fern"));
    Ok(())
}

fn person_age(data: &TestData) -> u32 {
    match data {
        TestData::Person(person) => person.age,
        other => panic!("expected person, got {other:?}"),
    }
}

fn animal_legs(data: &TestData) -> u32 {
    match data {
        TestData::Animal(animal) => animal.legs,
        other => panic!("expected animal, got {other:?}"),
    }
}
