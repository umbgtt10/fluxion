// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, HasTimestamp, StreamItem};
use fluxion_stream::prelude::*;
use fluxion_stream::CombinedState;
use fluxion_test_utils::{
    assert_no_element_emitted, test_channel_with_errors,
    test_data::{
        animal, animal_dog, animal_spider, person, person_alice, person_bob, person_charlie,
        TestData,
    },
    unwrap_stream, Sequenced,
};

#[tokio::test]
async fn test_emit_when_propagates_error_from_scanned_source() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<TestData>>();

    // Source chain: scan_ordered -> emit_when
    // Scan: Sum ages of persons
    let scanned_source = source_stream.scan_ordered(0u32, |acc, x| {
        if let TestData::Person(p) = x {
            *acc += p.age;
        }
        person("Sum".to_string(), *acc)
    });

    // Trigger: Animal stream
    // Filter: Emit if Sum of ages > Animal legs * 10
    let mut result =
        scanned_source.emit_when(trigger_stream, |state: &CombinedState<TestData, u64>| {
            let values = state.values();
            let source_val = &values[0]; // Scanned Person
            let trigger_val = &values[1]; // Animal

            match (source_val, trigger_val) {
                (TestData::Person(p), TestData::Animal(a)) => p.age > (a.legs * 10),
                _ => false,
            }
        });

    // Act & Assert
    // 1. Send value to source (Alice 25) -> Sum 25
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;

    // 2. Send value to trigger (Dog 4 legs) -> Threshold 40. 25 > 40 is False.
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal_dog(),
        2,
    )))?;

    assert_no_element_emitted(&mut result, 100).await;

    // 3. Send value to source (Bob 30) -> Sum 55.
    // Trigger is still Dog (4 legs) -> Threshold 40.
    // 55 > 40 is True. Should emit.
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_bob(),
        3,
    )))?;

    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if matches!(&v.value, TestData::Person(p) if p.name == "Sum" && p.age == 55)
    ));

    // 4. Send error to source
    source_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Source Error",
    )))?;

    // Error should propagate
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(e) if e.to_string().contains("Source Error")
    ));

    // 5. Recovery
    // Source state preserved (55).
    // Send Charlie (35) -> Sum 90.
    // Trigger still Dog (40). 90 > 40 is True.
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_charlie(),
        5,
    )))?;

    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if matches!(&v.value, TestData::Person(p) if p.name == "Sum" && p.age == 90)
    ));

    Ok(())
}

#[tokio::test]
async fn test_emit_when_propagates_error_from_mapped_trigger() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<TestData>>();

    // Trigger chain: map_ordered -> emit_when (as filter stream)
    // Map: Double the legs of the animal
    let mapped_trigger = trigger_stream.map_ordered(|x| {
        if let TestData::Animal(a) = &x.value {
            Sequenced::with_timestamp(animal(a.species.clone(), a.legs * 2), x.timestamp())
        } else {
            x.clone()
        }
    });

    // Source: Person stream
    // Filter: Person age > Mapped Animal legs
    let mut result =
        source_stream.emit_when(mapped_trigger, |state: &CombinedState<TestData, u64>| {
            let values = state.values();
            let source_val = &values[0]; // Person
            let trigger_val = &values[1]; // Mapped Animal

            match (source_val, trigger_val) {
                (TestData::Person(p), TestData::Animal(a)) => p.age > a.legs,
                _ => false,
            }
        });

    // Act & Assert
    // 1. Setup trigger (Dog 4 legs -> Mapped to 8 legs)
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal_dog(),
        1,
    )))?;

    // 2. Send source value (Alice 25). 25 > 8 is True.
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        2,
    )))?;

    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if matches!(&v.value, TestData::Person(p) if p.name == "Alice")
    ));

    // 3. Send error to trigger
    trigger_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Trigger Error",
    )))?;

    // Error from trigger should propagate
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(e) if e.to_string().contains("Trigger Error")
    ));

    // 4. Continue
    // Update trigger to Spider (8 legs -> 16 legs)
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal_spider(),
        4,
    )))?;

    // Re-emission of Alice because she satisfies the new condition
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if matches!(&v.value, TestData::Person(p) if p.name == "Alice")
    ));

    // Send source value (Bob 30). 30 > 16 is True.
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_bob(),
        5,
    )))?;

    let item = unwrap_stream(&mut result, 100).await;
    assert!(matches!(
        item,
        StreamItem::Value(v) if matches!(&v.value, TestData::Person(p) if p.name == "Bob")
    ));

    Ok(())
}
