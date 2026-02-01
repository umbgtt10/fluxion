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
        animal_dog, person_alice, person_bob, person_charlie, person_dave, person_diane,
        plant_rose, TestData,
    },
};

static LATEST_FILTER: fn(&TestData) -> bool = |_| true;
static COMBINE_FILTER: fn(&CombinedState<TestData, u64>) -> bool = |_| true;

#[tokio::test]
async fn test_take_latest_when_take_while_with() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_rx) = test_channel::<Sequenced<TestData>>();
    let (latest_filter_tx, latest_filter_rx) = test_channel::<Sequenced<TestData>>();
    let (while_filter_tx, while_filter_rx) = test_channel::<Sequenced<bool>>();

    let mut composed = source_rx
        .take_latest_when(latest_filter_rx, LATEST_FILTER)
        .take_while_with(while_filter_rx, |f| *f);

    // Act
    while_filter_tx.unbounded_send(Sequenced::new(true))?;
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    latest_filter_tx
        .unbounded_send(Sequenced::new(person_alice()))
        .unwrap();

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut composed, 500).await)).value,
        person_alice()
    );

    // Act
    source_tx.unbounded_send(Sequenced::new(person_bob()))?;
    latest_filter_tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut composed, 500).await)).value,
        person_bob()
    );

    // Act
    while_filter_tx.unbounded_send(Sequenced::new(false))?;
    source_tx.unbounded_send(Sequenced::new(person_charlie()))?;
    latest_filter_tx.unbounded_send(Sequenced::new(person_charlie()))?;
    // Assert
    assert_no_element_emitted(&mut composed, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_take_while_with() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_rx) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_rx) = test_channel::<Sequenced<bool>>();

    let mut composed = person_rx
        .combine_latest(vec![animal_rx, plant_rx], COMBINE_FILTER)
        .take_while_with(filter_rx, |f| *f);

    // Act
    filter_tx.unbounded_send(Sequenced::new(true))?;
    person_tx.unbounded_send(Sequenced::new(person_alice()))?;
    animal_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    plant_tx.unbounded_send(Sequenced::new(plant_rose()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut composed, 500).await))
            .into_inner()
            .values(),
        [person_alice(), animal_dog(), plant_rose()]
    );

    // Act
    person_tx.unbounded_send(Sequenced::new(person_bob()))?;
    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut composed, 500).await))
            .into_inner()
            .values(),
        [person_bob(), animal_dog(), plant_rose()]
    );

    // Act
    filter_tx.unbounded_send(Sequenced::new(false))?;
    person_tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_no_element_emitted(&mut composed, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_filter_ordered_take_while_with() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_rx) = test_channel::<Sequenced<TestData>>();
    let (other_tx, other_rx) = test_channel::<Sequenced<TestData>>();
    let (predicate_tx, predicate_rx) = test_channel::<Sequenced<TestData>>();

    let mut stream = source_rx
        .ordered_merge(vec![other_rx])
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .take_while_with(predicate_rx, |_| true);

    // Act
    predicate_tx.unbounded_send(Sequenced::new(person_alice()))?;
    source_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    source_tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        person_bob()
    );

    // Act
    other_tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        person_charlie()
    );

    Ok(())
}
#[tokio::test]
async fn test_ordered_merge_take_while_with() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_rx) = test_channel::<Sequenced<bool>>();

    let mut composed = person_rx
        .ordered_merge(vec![animal_rx])
        .take_while_with(filter_rx, |f| *f);

    // Act
    filter_tx.unbounded_send(Sequenced::new(true))?;
    person_tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut composed, 500).await)).value,
        person_alice()
    );

    // Act
    animal_tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut composed, 500).await)).value,
        animal_dog()
    );

    // Act
    person_tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut composed, 500).await)).value,
        person_bob()
    );

    // Act
    filter_tx.unbounded_send(Sequenced::new(false))?;
    person_tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_no_element_emitted(&mut composed, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_map_ordered_combine_with_previous_take_while_with(
) -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let (predicate_tx, predicate_rx) = test_channel::<Sequenced<bool>>();

    let mut stream = stream
        .filter_ordered(|data| match data {
            TestData::Person(p) => p.age >= 30,
            _ => false,
        })
        .combine_with_previous()
        .map_ordered(|stream_item| {
            let item = stream_item;
            let current = match &item.current.value {
                TestData::Person(p) => p.name.clone(),
                _ => unreachable!(),
            };
            let previous = item.previous.map(|prev| match &prev.value {
                TestData::Person(p) => p.name.clone(),
                _ => unreachable!(),
            });
            Sequenced::new(format!("Current: {}, Previous: {:?}", current, previous))
        })
        .take_while_with(predicate_rx, |p| *p);

    // Act
    predicate_tx.unbounded_send(Sequenced::new(true))?;

    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Current: Bob, Previous: None"
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Current: Charlie, Previous: Some(\"Bob\")"
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_dave()))?;
    tx.unbounded_send(Sequenced::new(person_diane()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Current: Diane, Previous: Some(\"Charlie\")"
    );

    // Act
    predicate_tx.unbounded_send(Sequenced::new(false))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_no_element_emitted(&mut stream, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_combine_with_previous_map_ordered_take_while_with_age_difference(
) -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let (predicate_tx, predicate_rx) = test_channel::<Sequenced<bool>>();

    let mut stream = stream
        .combine_with_previous()
        .map_ordered(|stream_item| {
            let item = stream_item;
            let current_age = match &item.current.value {
                TestData::Person(p) => p.age,
                _ => 0,
            };
            let previous_age = item.previous.and_then(|prev| match &prev.value {
                TestData::Person(p) => Some(p.age),
                _ => None,
            });

            Sequenced::new(match previous_age {
                Some(prev) => current_age as i32 - prev as i32,
                None => 0,
            })
        })
        .take_while_with(predicate_rx, |p| *p);

    // Act
    predicate_tx.unbounded_send(Sequenced::new(true))?;
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        0
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        5
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_dave()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        -2
    );

    // Act
    predicate_tx.unbounded_send(Sequenced::new(false))?;
    tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_no_element_emitted(&mut stream, 100).await;

    Ok(())
}
