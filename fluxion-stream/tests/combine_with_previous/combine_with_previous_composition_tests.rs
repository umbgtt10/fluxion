// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::Timestamped;
use fluxion_stream::{CombinedState, FluxionStream};
use fluxion_test_utils::{
    helpers::unwrap_stream,
    test_channel,
    test_data::{animal_dog, person_alice, person_bob, person_charlie, person_dave, TestData},
    unwrap_value, Sequenced,
};

static FILTER: fn(&TestData) -> bool = |_| true;
static COMBINE_FILTER: fn(&CombinedState<TestData, u64>) -> bool = |_| true;

#[tokio::test]
async fn test_take_latest_when_combine_with_previous() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut composed = FluxionStream::new(source_stream)
        .take_latest_when(filter_stream, FILTER)
        .combine_with_previous();

    // Act & Assert - send source first, then filter triggers emission
    source_tx.send(Sequenced::new(person_alice()))?;
    filter_tx.send(Sequenced::new(person_alice()))?;

    let element = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert!(
        element.previous.is_none(),
        "First emission should have no previous"
    );
    assert_eq!(&element.current.value, &person_alice());

    // Update source, then trigger with filter
    source_tx.send(Sequenced::new(person_bob()))?;
    filter_tx.send(Sequenced::new(person_bob()))?;
    let element = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert_eq!(&element.previous.unwrap().value, &person_alice());
    assert_eq!(&element.current.value, &person_bob());

    // Update source, then trigger with filter
    source_tx.send(Sequenced::new(person_charlie()))?;
    filter_tx.send(Sequenced::new(person_charlie()))?;
    let element = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert_eq!(&element.previous.unwrap().value, &person_bob());
    assert_eq!(&element.current.value, &person_charlie());

    // Update source, then trigger with filter
    source_tx.send(Sequenced::new(person_dave()))?;
    filter_tx.send(Sequenced::new(person_dave()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert_eq!(&item.previous.unwrap().value, &person_charlie());
    assert_eq!(&item.current.value, &person_dave());

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_combine_with_previous() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();

    let person_stream = person_rx;
    let animal_stream = animal_rx;

    let mut composed = FluxionStream::new(person_stream)
        .ordered_merge(vec![FluxionStream::new(animal_stream)])
        .combine_with_previous();

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert!(item.previous.is_none());
    assert_eq!(&item.current.value, &person_alice());

    animal_tx.send(Sequenced::new(animal_dog()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert_eq!(&item.previous.unwrap().value, &person_alice());
    assert_eq!(&item.current.value, &animal_dog());

    person_tx.send(Sequenced::new(person_bob()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert_eq!(&item.previous.unwrap().value, &animal_dog());
    assert_eq!(&item.current.value, &person_bob());

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_combine_with_previous() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();

    let person_stream = person_rx;
    let animal_stream = animal_rx;

    let mut composed = FluxionStream::new(person_stream)
        .combine_latest(vec![animal_stream], COMBINE_FILTER)
        .combine_with_previous();

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;

    let item = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert!(item.previous.is_none());
    let curr_binding = item.current;
    let inner = curr_binding.clone().into_inner();
    let curr_state = inner.values();
    assert_eq!(&curr_state[0], &person_alice());
    assert_eq!(&curr_state[1], &animal_dog());

    person_tx.send(Sequenced::new(person_bob()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    let prev_seq = item.previous.unwrap();
    let prev_binding = prev_seq;
    let inner = prev_binding.clone().into_inner();
    let prev_state = inner.values();
    assert_eq!(&prev_state[0], &person_alice());
    assert_eq!(&prev_state[1], &animal_dog());

    let curr_binding = item.current;
    let inner = curr_binding.clone().into_inner();
    let curr_state = inner.values();
    assert_eq!(&curr_state[0], &person_bob());
    assert_eq!(&curr_state[1], &animal_dog());

    Ok(())
}
