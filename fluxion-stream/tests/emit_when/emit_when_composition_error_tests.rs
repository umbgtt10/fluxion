// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, HasTimestamp, StreamItem};
use fluxion_stream::prelude::*;
use fluxion_stream::CombinedState;
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, test_channel_with_errors, unwrap_stream},
    sequenced::Sequenced,
    test_data::{
        animal, animal_dog, animal_spider, person, person_alice, person_bob, person_charlie,
        TestData,
    },
};

#[tokio::test]
async fn test_emit_when_propagates_error_from_scanned_source() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<TestData>>();

    let scanned_source = source_stream.scan_ordered(0u32, |acc, x| {
        if let TestData::Person(p) = x {
            *acc += p.age;
        }
        person("Sum".to_string(), *acc)
    });

    let mut result =
        scanned_source.emit_when(trigger_stream, |state: &CombinedState<TestData, u64>| {
            let values = state.values();
            let source_val = &values[0];
            let trigger_val = &values[1];

            match (source_val, trigger_val) {
                (TestData::Person(p), TestData::Animal(a)) => p.age > (a.legs * 10),
                _ => false,
            }
        });

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal_dog(),
        2,
    )))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_bob(),
        3,
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if matches!(&v.value, TestData::Person(p) if p.name == "Sum" && p.age == 55)
    ));

    // Act
    source_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Source Error",
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(e) if e.to_string().contains("Source Error")
    ));

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_charlie(),
        5,
    )))?;

    // Assert
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

    let mapped_trigger = trigger_stream.map_ordered(|x| {
        if let TestData::Animal(a) = &x.value {
            Sequenced::with_timestamp(animal(a.species.clone(), a.legs * 2), x.timestamp())
        } else {
            x.clone()
        }
    });

    let mut result =
        source_stream.emit_when(mapped_trigger, |state: &CombinedState<TestData, u64>| {
            let values = state.values();
            let source_val = &values[0];
            let trigger_val = &values[1];

            match (source_val, trigger_val) {
                (TestData::Person(p), TestData::Animal(a)) => p.age > a.legs,
                _ => false,
            }
        });

    // Act
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal_dog(),
        1,
    )))?;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        2,
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if matches!(&v.value, TestData::Person(p) if p.name == "Alice")
    ));

    // Act
    trigger_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Trigger Error",
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(e) if e.to_string().contains("Trigger Error")
    ));

    // Act
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal_spider(),
        4,
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if matches!(&v.value, TestData::Person(p) if p.name == "Alice")
    ));

    // Act
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_bob(),
        5,
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if matches!(&v.value, TestData::Person(p) if p.name == "Bob")
    ));

    Ok(())
}
