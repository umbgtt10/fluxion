// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::Timestamped;
use fluxion_stream::{CombinedState, DistinctUntilChangedExt, FluxionStream};
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

#[tokio::test]
async fn test_complex_composition_ordered_merge_and_combine_with_previous() -> anyhow::Result<()> {
    // Arrange: ordered_merge -> combine_with_previous
    let (person1_tx, person1_rx) = test_channel::<Sequenced<TestData>>();
    let (person2_tx, person2_rx) = test_channel::<Sequenced<TestData>>();

    let person1_stream = person1_rx;
    let person2_stream = person2_rx;

    let mut stream = FluxionStream::new(person1_stream)
        .ordered_merge(vec![FluxionStream::new(person2_stream)])
        .combine_with_previous();

    // Act & Assert
    person1_tx.send(Sequenced::new(person_alice()))?; // 25

    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert!(result.previous.is_none());
    assert_eq!(&result.current.value, &person_alice());

    person2_tx.send(Sequenced::new(person_bob()))?; // 30
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&result.previous.unwrap().value, &person_alice());
    assert_eq!(&result.current.value, &person_bob());

    person1_tx.send(Sequenced::new(person_charlie()))?; // 35
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&result.previous.unwrap().value, &person_bob());
    assert_eq!(&result.current.value, &person_charlie());

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_combine_with_previous() -> anyhow::Result<()> {
    // Arrange - filter for adults (age > 25), then track changes
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(stream)
        .filter_ordered(|test_data| match test_data {
            TestData::Person(p) => p.age > 25,
            _ => false,
        })
        .combine_with_previous();

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?; // 25 - filtered
    tx.send(Sequenced::new(person_bob()))?; // 30 - kept
    let item = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert!(item.previous.is_none());
    assert_eq!(&item.current.value, &person_bob());

    tx.send(Sequenced::new(person_charlie()))?; // 35 - kept
    let item = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&item.previous.unwrap().value, &person_bob());
    assert_eq!(&item.current.value, &person_charlie());

    tx.send(Sequenced::new(person_dave()))?; // 28 - kept
    let item = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&item.previous.unwrap().value, &person_charlie());
    assert_eq!(&item.current.value, &person_dave());

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_with_distinct_until_changed_composition() -> anyhow::Result<()> {
    // Arrange
    let (stream1_tx, stream1) = test_channel::<Sequenced<TestData>>();
    let (stream2_tx, stream2) = test_channel::<Sequenced<TestData>>();

    // Composition: combine_latest -> map to combined age -> distinct_until_changed -> combine_with_previous
    let mut result = FluxionStream::new(stream1)
        .combine_latest(vec![stream2], |_| true)
        .map_ordered(|state| {
            let age1 = match &state.values()[0] {
                TestData::Person(p) => p.age,
                _ => 0,
            };
            let age2 = match &state.values()[1] {
                TestData::Person(p) => p.age,
                _ => 0,
            };
            let sum = age1 + age2;
            Sequenced::new(sum)
        })
        .distinct_until_changed()
        .combine_with_previous();

    // Act & Assert
    // Initial values: Alice (25) + Bob (30) = 55
    stream1_tx.send(Sequenced::new(person_alice()))?;
    stream2_tx.send(Sequenced::new(person_bob()))?;
    let combined = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(combined.current.value, 55);
    assert_eq!(combined.previous, None);

    // Update stream1: Bob (30) + Bob (30) = 60 (different, should emit)
    stream1_tx.send(Sequenced::new(person_bob()))?;
    let combined = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(combined.current.value, 60);
    assert_eq!(combined.previous.as_ref().unwrap().value, 55);

    // Update stream2: Bob (30) + Charlie (35) = 65 (different, should emit)
    stream2_tx.send(Sequenced::new(person_charlie()))?;
    let combined = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(combined.current.value, 65);
    assert_eq!(combined.previous.as_ref().unwrap().value, 60);

    // Update stream1: Bob (30) + Charlie (35) = 65 (same, filtered by distinct)
    stream1_tx.send(Sequenced::new(person_bob()))?;
    // No emission expected

    // Update stream2: Bob (30) + Dave (28) = 58 (different, should emit)
    stream2_tx.send(Sequenced::new(person_dave()))?;
    let combined = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(combined.current.value, 58);
    assert_eq!(combined.previous.as_ref().unwrap().value, 65);

    drop(stream1_tx);
    drop(stream2_tx);

    Ok(())
}
