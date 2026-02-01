// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::Timestamped;
use fluxion_stream::prelude::*;
use fluxion_stream::CombinedState;
use fluxion_test_utils::helpers::test_channel;
use fluxion_test_utils::helpers::unwrap_value;
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    sequenced::Sequenced,
    test_data::{
        animal_dog, person_alice, person_bob, person_charlie, person_dave, plant_rose, TestData,
    },
};

static COMBINE_FILTER: fn(&CombinedState<TestData, u64>) -> bool = |_| true;
static FILTER: fn(&TestData) -> bool = |_| true;

#[tokio::test]
async fn test_combine_latest_take_latest_when() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();
    let (trigger_person_tx, trigger_person_rx) = test_channel::<Sequenced<TestData>>();
    let (trigger_animal_tx, trigger_animal_rx) = test_channel::<Sequenced<TestData>>();

    let person_stream = person_rx;
    let animal_stream = animal_rx;
    let trigger_person_stream = trigger_person_rx;
    let trigger_animal_stream = trigger_animal_rx;

    let trigger_combined =
        trigger_person_stream.combine_latest(vec![trigger_animal_stream], COMBINE_FILTER);

    let mut composed = person_stream
        .combine_latest(vec![animal_stream], COMBINE_FILTER)
        .take_latest_when(trigger_combined, |state| state.values().len() >= 2);

    // Act
    person_tx.unbounded_send(Sequenced::new(person_alice()))?;
    animal_tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Assert
    assert_no_element_emitted(&mut composed, 100).await;

    // Act
    trigger_person_tx.unbounded_send(Sequenced::new(person_bob()))?;
    trigger_animal_tx.unbounded_send(Sequenced::new(plant_rose()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut composed, 500).await))
            .into_inner()
            .values(),
        [person_alice(), animal_dog()]
    );

    // Act
    person_tx.unbounded_send(Sequenced::new(person_charlie()))?;
    trigger_person_tx.unbounded_send(Sequenced::new(person_dave()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut composed, 500).await))
            .into_inner()
            .values(),
        [person_charlie(), animal_dog()]
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_take_latest_when() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_rx) = test_channel::<Sequenced<TestData>>();

    let person_stream = person_rx;
    let animal_stream = animal_rx;
    let filter_stream = filter_rx;

    static LATEST_FILTER: fn(&TestData) -> bool = |_| true;

    let mut composed = person_stream
        .ordered_merge(vec![animal_stream])
        .take_latest_when(filter_stream, LATEST_FILTER);

    // Act
    person_tx.unbounded_send(Sequenced::new(person_alice()))?;
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut composed, 500).await)).value,
        person_alice()
    );

    // Act
    animal_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut composed, 500).await)).value,
        animal_dog()
    );

    // Act
    person_tx.unbounded_send(Sequenced::new(person_bob()))?;
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut composed, 500).await)).value,
        person_bob()
    );

    // Act
    drop(filter_tx);
    person_tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_no_element_emitted(&mut composed, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_take_latest_when() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_rx) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_rx) = test_channel::<Sequenced<TestData>>();

    let mut stream = source_rx
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .take_latest_when(filter_rx, FILTER);

    // Act
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    source_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    source_tx.unbounded_send(Sequenced::new(person_bob()))?;
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        person_bob()
    );

    // Act
    source_tx.unbounded_send(Sequenced::new(person_charlie()))?;
    source_tx.unbounded_send(Sequenced::new(plant_rose()))?;
    filter_tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        person_charlie()
    );

    Ok(())
}
