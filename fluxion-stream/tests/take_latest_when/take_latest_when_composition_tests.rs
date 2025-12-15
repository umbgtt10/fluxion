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
        animal_dog, person_alice, person_bob, person_charlie, person_dave, plant_rose, TestData,
    },
    unwrap_value, Sequenced,
};

static COMBINE_FILTER: fn(&CombinedState<TestData, u64>) -> bool = |_| true;
static FILTER: fn(&TestData) -> bool = |_| true;

#[tokio::test]
async fn test_combine_latest_take_latest_when() -> anyhow::Result<()> {
    // Arrange: Chain combine_latest with take_latest_when
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();
    let (trigger_person_tx, trigger_person_rx) = test_channel::<Sequenced<TestData>>();
    let (trigger_animal_tx, trigger_animal_rx) = test_channel::<Sequenced<TestData>>();

    let person_stream = person_rx;
    let animal_stream = animal_rx;
    let trigger_person_stream = trigger_person_rx;
    let trigger_animal_stream = trigger_animal_rx;

    // Create trigger combined stream first
    let trigger_combined =
        trigger_person_stream.combine_latest(vec![trigger_animal_stream], COMBINE_FILTER);

    // Chain: combine_latest then take_latest_when
    let mut composed = person_stream
        .combine_latest(vec![animal_stream], COMBINE_FILTER)
        .take_latest_when(
            trigger_combined,
            |state| state.values().len() >= 2, // Trigger when trigger stream has both values
        );

    // Act & Assert: First populate the source stream
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;

    // No emission yet - waiting for trigger
    assert_no_element_emitted(&mut composed, 100).await;

    // Now trigger emission by populating trigger streams
    trigger_person_tx.send(Sequenced::new(person_bob()))?;
    trigger_animal_tx.send(Sequenced::new(plant_rose()))?;

    // Should emit the latest from source stream
    let result = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    let inner = result.clone().into_inner();
    let values = inner.values();
    assert_eq!(values.len(), 2);
    assert_eq!(&values[0], &person_alice());
    assert_eq!(&values[1], &animal_dog());

    // Update source stream
    person_tx.send(Sequenced::new(person_charlie()))?;

    // Trigger another emission
    trigger_person_tx.send(Sequenced::new(person_dave()))?;

    let result = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    let inner = result.clone().into_inner();
    let values = inner.values();
    assert_eq!(values.len(), 2);
    assert_eq!(&values[0], &person_charlie());
    assert_eq!(&values[1], &animal_dog());

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

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice()))?;
    filter_tx.send(Sequenced::new(person_alice()))?;
    {
        let val = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
        assert_eq!(&val.value, &person_alice());
    }

    animal_tx.send(Sequenced::new(animal_dog()))?;
    filter_tx.send(Sequenced::new(person_alice()))?;
    {
        let val = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
        assert_eq!(&val.value, &animal_dog());
    }

    person_tx.send(Sequenced::new(person_bob()))?;
    filter_tx.send(Sequenced::new(person_alice()))?;
    {
        let val = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
        assert_eq!(&val.value, &person_bob());
    }

    drop(filter_tx);
    person_tx.send(Sequenced::new(person_charlie()))?;
    // After filter stream closes, no more emissions should occur
    assert_no_element_emitted(&mut composed, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_take_latest_when() -> anyhow::Result<()> {
    // Arrange - filter source stream, then apply take_latest_when
    let (source_tx, source_rx) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_rx) = test_channel::<Sequenced<TestData>>();

    let mut stream = source_rx
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .take_latest_when(filter_rx, FILTER);

    // Act & Assert
    source_tx.send(Sequenced::new(person_alice()))?;
    source_tx.send(Sequenced::new(animal_dog()))?; // Filtered
    source_tx.send(Sequenced::new(person_bob()))?;

    filter_tx.send(Sequenced::new(person_alice()))?; // Trigger emission

    {
        let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));

        assert_eq!(&val.value, &person_bob());
    }

    source_tx.send(Sequenced::new(person_charlie()))?;
    source_tx.send(Sequenced::new(plant_rose()))?; // Filtered

    filter_tx.send(Sequenced::new(person_bob()))?; // Trigger emission
    {
        let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
        assert_eq!(&val.value, &person_charlie());
    }

    Ok(())
}
