// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, HasTimestamp, StreamItem};
use fluxion_stream::prelude::*;
use fluxion_stream::take_latest_when::TakeLatestWhenExt;
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, test_channel_with_errors, unwrap_stream},
    sequenced::Sequenced,
    test_data::{
        animal, animal_ant, animal_cat, animal_dog, animal_spider, person, person_alice,
        person_bob, person_charlie, TestData,
    },
};

#[tokio::test]
async fn test_take_latest_when_propagates_error_from_mapped_source() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<TestData>>();

    // Source chain: map_ordered -> take_latest_when
    // Map: Append " Jr." to person name
    let mapped_source = source_stream.map_ordered(|x| {
        if let TestData::Person(p) = &x.value {
            Sequenced::with_timestamp(person(format!("{} Jr.", p.name), p.age), x.timestamp())
        } else {
            x.clone()
        }
    });

    // Trigger when Animal has > 2 legs
    let mut result = mapped_source.take_latest_when(trigger_stream, |t| {
        if let TestData::Animal(a) = t {
            a.legs > 2
        } else {
            false
        }
    });

    // Act & Assert
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal_dog(),
        2,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if matches!(&v.value, TestData::Person(p) if p.name == "Alice Jr.")
    ));

    source_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Source Error",
    )))?;

    // Error should be propagated immediately
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(e) if e.to_string().contains("Source Error")
    ));

    // Ensure source update (ts=3) is processed BEFORE trigger (ts=4)
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_bob(),
        3,
    )))?;
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal_cat(),
        4,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if matches!(&v.value, TestData::Person(p) if p.name == "Bob Jr.")
    ));

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_propagates_error_from_filtered_trigger() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<TestData>>();

    // Trigger chain: filter_ordered -> take_latest_when (as trigger)
    // We filter trigger values (Animals) with legs > 4.
    let filtered_trigger = trigger_stream.filter_ordered(|x| {
        if let TestData::Animal(a) = x {
            a.legs > 4
        } else {
            false
        }
    });

    // Predicate always true, but trigger stream itself is filtered
    let mut result = source_stream.take_latest_when(filtered_trigger, |_| true);

    // Act & Assert
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;

    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal_dog(),
        2,
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal_ant(),
        3,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if matches!(&v.value, TestData::Person(p) if p.name == "Alice")
    ));

    trigger_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Trigger Error",
    )))?;

    // Error from trigger should propagate
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(e) if e.to_string().contains("Trigger Error")
    ));

    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_bob(),
        4,
    )))?;
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal_spider(),
        5,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if matches!(&v.value, TestData::Person(p) if p.name == "Bob")
    ));

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_complex_chain_with_scan_and_map() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<TestData>>();

    // Source: scan (sum of ages)
    // Input: Sequenced<TestData>
    // Output: Sequenced<TestData> (Person with name "Sum" and age = sum)
    let scanned_source = source_stream.scan_ordered(0u32, |acc, x| {
        if let TestData::Person(p) = x {
            *acc += p.age;
        }
        person("Sum".to_string(), *acc)
    });

    // Trigger: map (double legs)
    // Input: Sequenced<TestData>
    // Output: Sequenced<TestData>
    let mapped_trigger = trigger_stream.map_ordered(|x| {
        if let TestData::Animal(a) = &x.value {
            Sequenced::with_timestamp(animal(a.species.clone(), a.legs * 2), x.timestamp())
        } else {
            x.clone()
        }
    });

    // Predicate: trigger value (Animal) legs > 10
    let mut result = scanned_source.take_latest_when(mapped_trigger, |t| {
        if let TestData::Animal(a) = t {
            a.legs > 10
        } else {
            false
        }
    });

    // Act & Assert
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_bob(),
        2,
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal_dog(),
        3,
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal_ant(),
        4,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if matches!(&v.value, TestData::Person(p) if p.name == "Sum" && p.age == 55)
    ));

    // 4. Error in source (scan)
    // 4. Error in source (scan)
    source_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Scan Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(e) if e.to_string().contains("Scan Error")
    ));

    // 5. Error in trigger (map)
    // 5. Error in trigger (map)
    trigger_tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Map Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(e) if e.to_string().contains("Map Error")
    ));

    // 6. Recovery
    // 6. Recovery
    // Source state should be preserved (55)
    // Add Charlie(35) -> 90
    source_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_charlie(),
        5,
    )))?;
    // Trigger with Spider(8 legs) -> mapped to 16 legs -> true
    trigger_tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        animal_spider(),
        6,
    )))?;

    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if matches!(&v.value, TestData::Person(p) if p.name == "Sum" && p.age == 90)
    ));

    Ok(())
}
