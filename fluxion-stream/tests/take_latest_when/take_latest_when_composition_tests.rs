// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::Timestamped;
use fluxion_stream::{CombinedState, FluxionStream};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel,
    test_data::{
        animal_dog, person_alice, person_bob, person_charlie, person_dave, plant_rose, TestData,
    },
    unwrap_value, Sequenced,
};

static COMBINE_FILTER: fn(&CombinedState<TestData, u64>) -> bool = |_| true;

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
    let trigger_combined = FluxionStream::new(trigger_person_stream)
        .combine_latest(vec![trigger_animal_stream], COMBINE_FILTER);

    // Chain: combine_latest then take_latest_when
    let mut composed = FluxionStream::new(person_stream)
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
