// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use fluxion_core::{StreamItem, Timestamped};
use fluxion_stream::prelude::*;
use fluxion_stream::CombinedState;
use fluxion_test_utils::{
    assert_no_element_emitted,
    helpers::unwrap_stream,
    test_channel,
    test_data::{animal_dog, person_alice, person_bob, person_charlie, person_dave, TestData},
    Sequenced,
};

static FILTER: fn(&TestData) -> bool = |_| true;
static COMBINE_FILTER: fn(&CombinedState<TestData, u64>) -> bool = |_| true;

#[tokio::test]
async fn test_take_latest_when_combine_with_previous() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut composed = source_stream
        .take_latest_when(filter_stream, FILTER)
        .combine_with_previous();

    // Act & Assert - send source first, then filter triggers emission
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;

    assert!(matches!(
        unwrap_stream(&mut composed, 500).await,
        StreamItem::Value(val) if val.previous.is_none() && val.current.value == person_alice()
    ));

    // Update source, then trigger with filter
    source_tx.unbounded_send(Sequenced::new(person_bob()))?;
    filter_tx.unbounded_send(Sequenced::new(person_bob()))?;
    assert!(matches!(
        unwrap_stream(&mut composed, 500).await,
        StreamItem::Value(val) if val.current.value == person_bob() && matches!(&val.previous, Some(prev) if prev.value == person_alice())
    ));

    // Update source, then trigger with filter
    source_tx.unbounded_send(Sequenced::new(person_charlie()))?;
    filter_tx.unbounded_send(Sequenced::new(person_charlie()))?;
    assert!(matches!(
        unwrap_stream(&mut composed, 500).await,
        StreamItem::Value(val) if val.current.value == person_charlie() && matches!(&val.previous, Some(prev) if prev.value == person_bob())
    ));

    // Update source, then trigger with filter
    source_tx.unbounded_send(Sequenced::new(person_dave()))?;
    filter_tx.unbounded_send(Sequenced::new(person_dave()))?;
    assert!(matches!(
        unwrap_stream(&mut composed, 500).await,
        StreamItem::Value(val) if val.current.value == person_dave() && matches!(&val.previous, Some(prev) if prev.value == person_charlie())
    ));

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_combine_with_previous() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();

    let person_stream = person_rx;
    let animal_stream = animal_rx;

    let mut composed = person_stream
        .ordered_merge(vec![animal_stream])
        .combine_with_previous();

    // Act & Assert
    person_tx.unbounded_send(Sequenced::new(person_alice()))?;
    assert!(matches!(
        unwrap_stream(&mut composed, 500).await,
        StreamItem::Value(val) if val.previous.is_none() && val.current.value == person_alice()
    ));

    animal_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    assert!(matches!(
        unwrap_stream(&mut composed, 500).await,
        StreamItem::Value(val) if val.current.value == animal_dog() && matches!(&val.previous, Some(prev) if prev.value == person_alice())
    ));

    person_tx.unbounded_send(Sequenced::new(person_bob()))?;
    assert!(matches!(
        unwrap_stream(&mut composed, 500).await,
        StreamItem::Value(val) if val.current.value == person_bob() && matches!(&val.previous, Some(prev) if prev.value == animal_dog())
    ));

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_combine_with_previous() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();

    let person_stream = person_rx;
    let animal_stream = animal_rx;

    let mut composed = person_stream
        .combine_latest(vec![animal_stream], COMBINE_FILTER)
        .combine_with_previous();

    // Act & Assert
    person_tx.unbounded_send(Sequenced::new(person_alice()))?;
    animal_tx.unbounded_send(Sequenced::new(animal_dog()))?;

    assert!(matches!(
        unwrap_stream(&mut composed, 500).await,
        StreamItem::Value(val) if val.previous.is_none() && {
            let state = val.current.clone().into_inner();
            let values = state.values();
            values[0] == person_alice() && values[1] == animal_dog()
        }
    ));

    person_tx.unbounded_send(Sequenced::new(person_bob()))?;
    assert!(matches!(
        unwrap_stream(&mut composed, 500).await,
        StreamItem::Value(val) if {
            let prev_state = val.previous.clone().unwrap().into_inner();
            let prev_values = prev_state.values();
            prev_values[0] == person_alice() && prev_values[1] == animal_dog()
        } && {
            let curr_state = val.current.clone().into_inner();
            let curr_values = curr_state.values();
            curr_values[0] == person_bob() && curr_values[1] == animal_dog()
        }
    ));

    Ok(())
}

#[tokio::test]
async fn test_complex_composition_ordered_merge_and_combine_with_previous() -> anyhow::Result<()> {
    // Arrange: ordered_merge -> combine_with_previous
    let (person1_tx, person1_rx) = test_channel::<Sequenced<TestData>>();
    let (person2_tx, person2_rx) = test_channel::<Sequenced<TestData>>();

    let person1_stream = person1_rx;
    let person2_stream = person2_rx;

    let mut stream = person1_stream
        .ordered_merge(vec![person2_stream])
        .combine_with_previous();

    // Act & Assert
    person1_tx.unbounded_send(Sequenced::new(person_alice()))?; // 25
    assert!(matches!(
        unwrap_stream(&mut stream, 500).await,
        StreamItem::Value(val) if val.previous.is_none() && val.current.value == person_alice()
    ));

    person2_tx.unbounded_send(Sequenced::new(person_bob()))?; // 30
    assert!(matches!(
        unwrap_stream(&mut stream, 500).await,
        StreamItem::Value(val) if val.current.value == person_bob() && matches!(&val.previous, Some(prev) if prev.value == person_alice())
    ));

    person1_tx.unbounded_send(Sequenced::new(person_charlie()))?; // 35
    assert!(matches!(
        unwrap_stream(&mut stream, 500).await,
        StreamItem::Value(val) if val.current.value == person_charlie() && matches!(&val.previous, Some(prev) if prev.value == person_bob())
    ));

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_combine_with_previous() -> anyhow::Result<()> {
    // Arrange - filter for adults (age > 25), then track changes
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut stream = stream
        .filter_ordered(|test_data| match test_data {
            TestData::Person(p) => p.age > 25,
            _ => false,
        })
        .combine_with_previous();

    // Act & Assert
    tx.unbounded_send(Sequenced::new(person_alice()))?; // 25 - filtered
    assert_no_element_emitted(&mut stream, 500).await;

    tx.unbounded_send(Sequenced::new(person_bob()))?; // 30 - kept
    assert!(matches!(
        unwrap_stream(&mut stream, 500).await,
        StreamItem::Value(val) if val.previous.is_none() && val.current.value == person_bob()
    ));

    tx.unbounded_send(Sequenced::new(person_charlie()))?; // 35 - kept
    assert!(matches!(
        unwrap_stream(&mut stream, 500).await,
        StreamItem::Value(val) if val.current.value == person_charlie() && matches!(&val.previous, Some(prev) if prev.value == person_bob())
    ));

    tx.unbounded_send(Sequenced::new(person_dave()))?; // 28 - kept
    assert!(matches!(
        unwrap_stream(&mut stream, 500).await,
        StreamItem::Value(val) if val.current.value == person_dave() && matches!(&val.previous, Some(prev) if prev.value == person_charlie())
    ));

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_with_distinct_until_changed_composition() -> anyhow::Result<()> {
    // Arrange
    let (stream1_tx, stream1) = test_channel::<Sequenced<TestData>>();
    let (stream2_tx, stream2) = test_channel::<Sequenced<TestData>>();

    // Composition: combine_latest -> map to combined age -> distinct_until_changed -> combine_with_previous
    let mut result = stream1
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
    stream1_tx.unbounded_send(Sequenced::new(person_alice()))?;
    stream2_tx.unbounded_send(Sequenced::new(person_bob()))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(val) if val.current.value == 55 && val.previous.is_none()
    ));

    stream1_tx.unbounded_send(Sequenced::new(person_bob()))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(val) if val.current.value == 60 && matches!(&val.previous, Some(prev) if prev.value == 55)
    ));

    stream2_tx.unbounded_send(Sequenced::new(person_charlie()))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(val) if val.current.value == 65 && matches!(&val.previous, Some(prev) if prev.value == 60)
    ));

    stream1_tx.unbounded_send(Sequenced::new(person_bob()))?;
    assert_no_element_emitted(&mut result, 100).await;

    // Update stream2: Bob (30) + Dave (28) = 58 (different, should emit)
    stream2_tx.unbounded_send(Sequenced::new(person_dave()))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(val) if val.current.value == 58 && matches!(&val.previous, Some(prev) if prev.value == 65)
    ));

    drop(stream1_tx);
    drop(stream2_tx);

    Ok(())
}
