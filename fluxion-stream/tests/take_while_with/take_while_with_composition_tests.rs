// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::Timestamped;
use fluxion_stream::prelude::*;
use fluxion_stream::CombinedState;
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel,
    test_data::{
        animal_dog, person_alice, person_bob, person_charlie, person_dave, person_diane,
        plant_rose, TestData,
    },
    unwrap_value, Sequenced,
};

static LATEST_FILTER: fn(&TestData) -> bool = |_| true;
static COMBINE_FILTER: fn(&CombinedState<TestData, u64>) -> bool = |_| true;

#[tokio::test]
async fn test_take_latest_when_take_while_with() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_rx) = test_channel::<Sequenced<TestData>>();
    let (latest_filter_tx, latest_filter_rx) = test_channel::<Sequenced<TestData>>();
    let (while_filter_tx, while_filter_rx) = test_channel::<Sequenced<bool>>();

    let source_stream = source_rx;
    let latest_filter_stream = latest_filter_rx;
    let while_filter_stream = while_filter_rx;

    let mut composed = source_stream
        .take_latest_when(latest_filter_stream, LATEST_FILTER)
        .take_while_with(while_filter_stream, |f| *f);

    // Act & Assert
    while_filter_tx.try_send(Sequenced::new(true))?;
    source_tx.try_send(Sequenced::new(person_alice()))?;
    latest_filter_tx
        .try_send(Sequenced::new(person_alice()))
        .unwrap();
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut composed, 500).await)).value,
        &person_alice()
    );

    source_tx.try_send(Sequenced::new(person_bob()))?;
    latest_filter_tx.try_send(Sequenced::new(person_bob()))?;
    assert_eq!(
        &unwrap_value(Some(unwrap_stream(&mut composed, 500).await)).value,
        &person_bob()
    );

    while_filter_tx.try_send(Sequenced::new(false))?;
    source_tx.try_send(Sequenced::new(person_charlie()))?;
    latest_filter_tx.try_send(Sequenced::new(person_charlie()))?;
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

    let person_stream = person_rx;
    let animal_stream = animal_rx;
    let plant_stream = plant_rx;
    let filter_stream = filter_rx;

    let mut composed = person_stream
        .combine_latest(vec![animal_stream, plant_stream], COMBINE_FILTER)
        .take_while_with(filter_stream, |f| *f);

    // Act & Assert
    filter_tx.try_send(Sequenced::new(true))?;
    person_tx.try_send(Sequenced::new(person_alice()))?;
    animal_tx.try_send(Sequenced::new(animal_dog()))?;
    plant_tx.try_send(Sequenced::new(plant_rose()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut composed, 500).await))
            .into_inner()
            .values(),
        &[person_alice(), animal_dog(), plant_rose()]
    );

    person_tx.try_send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut composed, 500).await))
            .into_inner()
            .values(),
        &[person_bob(), animal_dog(), plant_rose()]
    );

    filter_tx.try_send(Sequenced::new(false))?;
    person_tx.try_send(Sequenced::new(person_charlie()))?;
    assert_no_element_emitted(&mut composed, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_filter_ordered_take_while_with() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_rx) = test_channel::<Sequenced<TestData>>();
    let (other_tx, other_rx) = test_channel::<Sequenced<TestData>>();
    let (predicate_tx, predicate_rx) = test_channel::<Sequenced<TestData>>();

    let source_stream = source_rx;
    let other_stream = other_rx;
    let predicate_stream = predicate_rx;

    let mut stream = source_stream
        .ordered_merge(vec![other_stream])
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .take_while_with(predicate_stream, |_| true);

    // Act & Assert
    predicate_tx.try_send(Sequenced::new(person_alice()))?;
    source_tx.try_send(Sequenced::new(animal_dog()))?; // Filtered by filter_ordered
    source_tx.try_send(Sequenced::new(person_bob()))?; // Kept
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        person_bob()
    );

    other_tx.try_send(Sequenced::new(person_charlie()))?; // Kept
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

    let person_stream = person_rx;
    let animal_stream = animal_rx;
    let filter_stream = filter_rx;

    let mut composed = person_stream
        .ordered_merge(vec![animal_stream])
        .take_while_with(filter_stream, |f| *f);

    // Act & Assert
    filter_tx.try_send(Sequenced::new(true))?;
    person_tx.try_send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut composed, 500).await)).value,
        person_alice()
    );

    animal_tx.try_send(Sequenced::new(animal_dog()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut composed, 500).await)).value,
        animal_dog()
    );

    person_tx.try_send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut composed, 500).await)).value,
        person_bob()
    );

    filter_tx.try_send(Sequenced::new(false))?;
    person_tx.try_send(Sequenced::new(person_charlie()))?;
    assert_no_element_emitted(&mut composed, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_map_ordered_combine_with_previous_take_while_with(
) -> anyhow::Result<()> {
    // Arrange - complex pipeline: filter -> combine_with_previous -> map -> take_while_with
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

    // Act & Assert
    predicate_tx.try_send(Sequenced::new(true))?;

    tx.try_send(Sequenced::new(person_alice()))?; // 25 - filtered
    tx.try_send(Sequenced::new(person_bob()))?; // 30 - kept

    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Current: Bob, Previous: None"
    );

    tx.try_send(Sequenced::new(person_charlie()))?; // 35 - kept
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Current: Charlie, Previous: Some(\"Bob\")"
    );

    tx.try_send(Sequenced::new(person_dave()))?; // 28 - filtered
    tx.try_send(Sequenced::new(person_diane()))?; // 40 - kept
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Current: Diane, Previous: Some(\"Charlie\")"
    );

    // Now test take_while_with stopping the stream
    predicate_tx.try_send(Sequenced::new(false))?;

    // Send another valid person
    tx.try_send(Sequenced::new(person_bob()))?; // 30 - kept by filter, but should be stopped by take_while_with

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

    // Act & Assert
    predicate_tx.try_send(Sequenced::new(true))?;

    tx.try_send(Sequenced::new(person_alice()))?; // Age 25
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        0
    ); // No previous

    tx.try_send(Sequenced::new(person_bob()))?; // Age 30
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        5
    ); // 30 - 25 = 5

    tx.try_send(Sequenced::new(person_dave()))?; // Age 28
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        -2
    ); // 28 - 30 = -2

    predicate_tx.try_send(Sequenced::new(false))?;
    tx.try_send(Sequenced::new(person_charlie()))?; // Age 35
    assert_no_element_emitted(&mut stream, 100).await;

    Ok(())
}
