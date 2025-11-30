// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::Timestamped;
use fluxion_stream::{CombinedState, FluxionStream};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel,
    test_data::{animal_dog, person_alice, person_bob, person_charlie, plant_rose, TestData},
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

    let mut composed = FluxionStream::new(source_stream)
        .take_latest_when(latest_filter_stream, LATEST_FILTER)
        .take_while_with(while_filter_stream, |f| *f);

    // Act & Assert
    while_filter_tx.send(Sequenced::new(true))?;
    source_tx.send(Sequenced::new(person_alice()))?;
    latest_filter_tx
        .send(Sequenced::new(person_alice()))
        .unwrap();
    let element = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert_eq!(&element.value, &person_alice());

    source_tx.send(Sequenced::new(person_bob()))?;
    latest_filter_tx.send(Sequenced::new(person_bob()))?;
    let element = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert_eq!(&element.value, &person_bob());

    while_filter_tx.send(Sequenced::new(false))?;
    source_tx.send(Sequenced::new(person_charlie()))?;
    latest_filter_tx.send(Sequenced::new(person_charlie()))?;
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

    let mut composed = FluxionStream::new(person_stream)
        .combine_latest(vec![animal_stream, plant_stream], COMBINE_FILTER)
        .take_while_with(filter_stream, |f| *f);

    // Act & Assert
    filter_tx.send(Sequenced::new(true))?;
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;
    plant_tx.send(Sequenced::new(plant_rose()))?;
    let result = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    let inner = result.clone().into_inner();
    let state = inner.values();
    assert_eq!(state.len(), 3);
    assert_eq!(&state[0], &person_alice());
    assert_eq!(&state[1], &animal_dog());
    assert_eq!(&state[2], &plant_rose());

    person_tx.send(Sequenced::new(person_bob()))?;
    let result = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    let inner = result.clone().into_inner();
    let state = inner.values();
    assert_eq!(&state[0], &person_bob());
    assert_eq!(&state[1], &animal_dog());
    assert_eq!(&state[2], &plant_rose());

    filter_tx.send(Sequenced::new(false))?;
    person_tx.send(Sequenced::new(person_charlie()))?;
    assert_no_element_emitted(&mut composed, 100).await;

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

    let mut composed = FluxionStream::new(person_stream)
        .ordered_merge(vec![FluxionStream::new(animal_stream)])
        .take_while_with(filter_stream, |f| *f);

    // Act & Assert
    filter_tx.send(Sequenced::new(true))?;
    person_tx.send(Sequenced::new(person_alice()))?;
    {
        let val = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
        assert_eq!(&val.value, &person_alice());
    }

    animal_tx.send(Sequenced::new(animal_dog()))?;
    {
        let val = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
        assert_eq!(&val.value, &animal_dog());
    }

    person_tx.send(Sequenced::new(person_bob()))?;
    {
        let val = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
        assert_eq!(&val.value, &person_bob());
    }

    filter_tx.send(Sequenced::new(false))?;
    person_tx.send(Sequenced::new(person_charlie()))?;
    assert_no_element_emitted(&mut composed, 100).await;

    Ok(())
}
