// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, HasTimestamp, StreamItem};
use fluxion_stream::{take_latest_when::TakeLatestWhenExt, FluxionStream};
use fluxion_test_utils::{
    assert_no_element_emitted, test_channel_with_errors,
    test_data::{
        animal, animal_ant, animal_cat, animal_dog, animal_spider, person, person_alice,
        person_bob, person_charlie, TestData,
    },
    unwrap_stream, Sequenced,
};

#[tokio::test]
async fn test_take_latest_when_propagates_error_from_mapped_source() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let (trigger_tx, trigger_stream) = test_channel_with_errors::<Sequenced<TestData>>();

    // Source chain: map_ordered -> take_latest_when
    // Map: Append " Jr." to person name
    let mapped_source = FluxionStream::new(source_stream).map_ordered(|x| {
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
    // 1. Send value to source (buffered)
    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    // 2. Trigger emission (Dog has 4 legs > 2)
    trigger_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        animal_dog(),
        2,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if matches!(&v.value, TestData::Person(p) if p.name == "Alice Jr.")
    ));

    // 3. Send error to source
    source_tx.send(StreamItem::Error(FluxionError::stream_error(
        "Source Error",
    )))?;

    // Error should be propagated immediately
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(e) if e.to_string().contains("Source Error")
    ));

    // 4. Continue after error
    // Ensure source update (ts=3) is processed BEFORE trigger (ts=4)
    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_bob(),
        3,
    )))?;
    trigger_tx.send(StreamItem::Value(Sequenced::with_timestamp(
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
    let filtered_trigger = FluxionStream::new(trigger_stream).filter_ordered(|x| {
        if let TestData::Animal(a) = x {
            a.legs > 4
        } else {
            false
        }
    });

    // Predicate always true, but trigger stream itself is filtered
    let mut result = source_stream.take_latest_when(filtered_trigger, |_| true);

    // Act & Assert
    // 1. Send value to source
    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;

    // 2. Send ignored trigger value (Dog legs=4 <= 4) - filtered out by filter_ordered
    trigger_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        animal_dog(),
        2,
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    // 3. Send valid trigger value (Ant legs=6 > 4)
    trigger_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        animal_ant(),
        3,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if matches!(&v.value, TestData::Person(p) if p.name == "Alice")
    ));

    // 4. Send error to trigger stream
    trigger_tx.send(StreamItem::Error(FluxionError::stream_error(
        "Trigger Error",
    )))?;

    // Error from trigger should propagate
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(e) if e.to_string().contains("Trigger Error")
    ));

    // 5. Continue
    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_bob(),
        4,
    )))?;
    trigger_tx.send(StreamItem::Value(Sequenced::with_timestamp(
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
    let scanned_source = FluxionStream::new(source_stream).scan_ordered(0u32, |acc, x| {
        if let TestData::Person(p) = x {
            *acc += p.age;
        }
        person("Sum".to_string(), *acc)
    });

    // Trigger: map (double legs)
    // Input: Sequenced<TestData>
    // Output: Sequenced<TestData>
    let mapped_trigger = FluxionStream::new(trigger_stream).map_ordered(|x| {
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
    // 1. Build up source state: Alice(25) + Bob(30) = 55
    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_alice(),
        1,
    )))?;
    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_bob(),
        2,
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    // 2. Trigger with Dog(4 legs) -> mapped to 8 legs -> predicate (8 > 10) is false
    trigger_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        animal_dog(),
        3,
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    // 3. Trigger with Ant(6 legs) -> mapped to 12 legs -> predicate (12 > 10) is true
    trigger_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        animal_ant(),
        4,
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if matches!(&v.value, TestData::Person(p) if p.name == "Sum" && p.age == 55)
    ));

    // 4. Error in source (scan)
    source_tx.send(StreamItem::Error(FluxionError::stream_error("Scan Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(e) if e.to_string().contains("Scan Error")
    ));

    // 5. Error in trigger (map)
    trigger_tx.send(StreamItem::Error(FluxionError::stream_error("Map Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(e) if e.to_string().contains("Map Error")
    ));

    // 6. Recovery
    // Source state should be preserved (55)
    // Add Charlie(35) -> 90
    source_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        person_charlie(),
        5,
    )))?;
    // Trigger with Spider(8 legs) -> mapped to 16 legs -> true
    trigger_tx.send(StreamItem::Value(Sequenced::with_timestamp(
        animal_spider(),
        6,
    )))?;

    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(v) if matches!(&v.value, TestData::Person(p) if p.name == "Sum" && p.age == 90)
    ));

    Ok(())
}
