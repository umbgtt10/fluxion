// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::take_while_with::TakeWhileExt;
use fluxion_test_utils::helpers::{assert_no_element_emitted, assert_stream_ended};
use fluxion_test_utils::test_data::{
    animal_cat, animal_dog, person_alice, person_bob, person_charlie, TestData,
};
use fluxion_test_utils::{test_channel, unwrap_stream, unwrap_value, Sequenced};

#[tokio::test]
async fn test_take_while_basic() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    // Predicate: Allow only if filter is a Person
    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act & Assert
    // 1. Set filter to True (Person)
    filter_tx.send(Sequenced::new(person_alice()))?;
    assert_no_element_emitted(&mut result, 100).await;

    // 2. Send source items (should pass)
    source_tx.send(Sequenced::new(animal_cat()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(item.value, animal_cat());

    source_tx.send(Sequenced::new(animal_dog()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(item.value, animal_dog());

    source_tx.send(Sequenced::new(person_alice()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(item.value, person_alice());

    // 3. Set filter to False (Animal)
    filter_tx.send(Sequenced::new(animal_dog()))?;

    // 4. Send source items (should fail predicate and terminate stream)
    source_tx.send(Sequenced::new(person_bob()))?;
    source_tx.send(Sequenced::new(person_charlie()))?;

    // Stream should have ended
    drop(source_tx);
    drop(filter_tx);
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_always_true() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act & Assert
    filter_tx.send(Sequenced::new(person_alice()))?;
    assert_no_element_emitted(&mut result, 100).await;

    source_tx.send(Sequenced::new(animal_cat()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(item.value, animal_cat());

    source_tx.send(Sequenced::new(animal_dog()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(item.value, animal_dog());

    source_tx.send(Sequenced::new(person_alice()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(item.value, person_alice());

    Ok(())
}

#[tokio::test]
async fn test_take_while_complex_predicate() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    // Predicate: Allow only if Person name is "Alice"
    let mut result = source_stream.take_while_with(filter_stream, |f| match f {
        TestData::Person(p) => p.name == "Alice",
        _ => false,
    });

    // Act & Assert
    // 1. Filter = Alice (True)
    filter_tx.send(Sequenced::new(person_alice()))?;
    assert_no_element_emitted(&mut result, 100).await;

    source_tx.send(Sequenced::new(animal_cat()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(item.value, animal_cat());

    source_tx.send(Sequenced::new(animal_dog()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(item.value, animal_dog());

    // 2. Filter = Bob (False)
    filter_tx.send(Sequenced::new(person_bob()))?;

    source_tx.send(Sequenced::new(person_alice()))?;
    source_tx.send(Sequenced::new(person_bob()))?;

    drop(source_tx);
    drop(filter_tx);
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_interleaved_updates() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act & Assert
    // 1. Filter True
    filter_tx.send(Sequenced::new(person_alice()))?;
    assert_no_element_emitted(&mut result, 100).await;

    source_tx.send(Sequenced::new(animal_cat()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(item.value, animal_cat());

    // 2. Filter True (update)
    filter_tx.send(Sequenced::new(person_bob()))?;
    source_tx.send(Sequenced::new(animal_dog()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(item.value, animal_dog());

    // 3. Filter True (update)
    filter_tx.send(Sequenced::new(person_charlie()))?;
    source_tx.send(Sequenced::new(person_alice()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(item.value, person_alice());

    // 4. Filter False
    filter_tx.send(Sequenced::new(animal_dog()))?;
    source_tx.send(Sequenced::new(person_bob()))?;
    source_tx.send(Sequenced::new(person_charlie()))?;

    drop(source_tx);
    drop(filter_tx);
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_empty_source() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act
    filter_tx.send(Sequenced::new(person_alice()))?;
    drop(source_tx);
    drop(filter_tx);

    // Assert
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_empty_filter() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (_, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act
    source_tx.send(Sequenced::new(animal_cat()))?;
    source_tx.send(Sequenced::new(animal_dog()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_filter_changes_back_to_true() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act & Assert
    // 1. True
    filter_tx.send(Sequenced::new(person_alice()))?;
    assert_no_element_emitted(&mut result, 100).await;
    source_tx.send(Sequenced::new(animal_cat()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(item.value, animal_cat());

    // 2. False -> Stream terminates
    filter_tx.send(Sequenced::new(animal_dog()))?;
    source_tx.send(Sequenced::new(animal_dog()))?;
    assert_no_element_emitted(&mut result, 500).await;

    // 3. True (ignored because stream ended)
    filter_tx.send(Sequenced::new(person_alice()))?;
    source_tx.send(Sequenced::new(person_alice()))?;

    drop(source_tx);
    drop(filter_tx);
    assert_stream_ended(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_multiple_source_items_same_filter() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act & Assert
    filter_tx.send(Sequenced::new(person_alice()))?;
    source_tx.send(Sequenced::new(animal_cat()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(item.value, animal_cat());

    source_tx.send(Sequenced::new(animal_dog()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(item.value, animal_dog());

    source_tx.send(Sequenced::new(person_alice()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(item.value, person_alice());

    source_tx.send(Sequenced::new(person_bob()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(item.value, person_bob());

    filter_tx.send(Sequenced::new(animal_dog()))?;
    source_tx.send(Sequenced::new(person_charlie()))?;

    drop(source_tx);
    drop(filter_tx);
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_filter_updates_without_source() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Act & Assert
    // Send False, then True, then True.
    // Since no source item is processed while filter is False, the stream should NOT terminate.
    filter_tx.send(Sequenced::new(animal_dog()))?; // False
    filter_tx.send(Sequenced::new(person_alice()))?; // True
    filter_tx.send(Sequenced::new(person_bob()))?; // True
    assert_no_element_emitted(&mut result, 100).await;

    source_tx.send(Sequenced::new(animal_cat()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(item.value, animal_cat());

    source_tx.send(Sequenced::new(animal_dog()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(item.value, animal_dog());

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_source_before_filter() -> anyhow::Result<()> {
    // Test the branch where source arrives but filter is None
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Send source first (filter is None) - should not emit
    source_tx.send(Sequenced::with_timestamp(person_alice(), 1))?;
    assert_no_element_emitted(&mut result, 100).await;

    // Now send filter value
    filter_tx.send(Sequenced::with_timestamp(person_alice(), 2))?;
    // Still no emission from the cached source (filter update doesn't emit)
    assert_no_element_emitted(&mut result, 100).await;

    // Send another source value - now should emit
    source_tx.send(Sequenced::with_timestamp(person_alice(), 3))?;

    let item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert!(matches!(item.value, TestData::Person(_)));

    drop(source_tx);
    drop(filter_tx);

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_filter_updates_then_source() -> anyhow::Result<()> {
    // Test multiple filter updates before source arrives
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Send multiple filter values (source is None)
    filter_tx.send(Sequenced::with_timestamp(person_alice(), 1))?;
    filter_tx.send(Sequenced::with_timestamp(animal_dog(), 2))?;
    filter_tx.send(Sequenced::with_timestamp(person_alice(), 3))?;

    assert_no_element_emitted(&mut result, 100).await;

    // Now send source value - should use latest filter (Person -> true)
    source_tx.send(Sequenced::with_timestamp(person_alice(), 4))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert!(matches!(item.value, TestData::Person(_)));

    drop(source_tx);
    drop(filter_tx);

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_already_terminated() -> anyhow::Result<()> {
    // Test the terminated flag branch
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Setup: filter passes, source emits
    filter_tx.send(Sequenced::with_timestamp(person_alice(), 1))?;
    source_tx.send(Sequenced::with_timestamp(person_alice(), 2))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert!(matches!(item.value, TestData::Person(_)));

    // Trigger termination: filter fails predicate (timestamp 3 < 4, so filter processes first)
    filter_tx.send(Sequenced::with_timestamp(animal_dog(), 3))?;
    source_tx.send(Sequenced::with_timestamp(person_alice(), 4))?;

    // Source at timestamp 4 should still emit because filter at timestamp 3 doesn't change state until processed
    // But after filter is processed and predicate fails, it should terminate
    // Actually the source at timestamp 4 will be processed and check against filter at timestamp 3 which fails
    // So no emission and termination
    assert_no_element_emitted(&mut result, 500).await;

    // Send more values - should still get nothing (terminated)
    source_tx.send(Sequenced::with_timestamp(person_alice(), 5))?;
    filter_tx.send(Sequenced::with_timestamp(person_alice(), 6))?;

    drop(source_tx);
    drop(filter_tx);
    assert_stream_ended(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_filter_false_on_first_source() -> anyhow::Result<()> {
    // Test termination on very first source value
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // Setup filter that will fail
    filter_tx.send(Sequenced::with_timestamp(animal_dog(), 1))?;

    // Send source - should terminate immediately
    source_tx.send(Sequenced::with_timestamp(person_alice(), 2))?;

    // No emission
    assert_no_element_emitted(&mut result, 100).await;

    // Drop channels
    drop(source_tx);
    drop(filter_tx);

    // Stream terminated
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_alternating_filter_values() -> anyhow::Result<()> {
    // Test filter changing but never failing predicate
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result =
        source_stream.take_while_with(filter_stream, |f| matches!(f, TestData::Person(_)));

    // All filter values pass predicate
    filter_tx.send(Sequenced::with_timestamp(person_alice(), 1))?;
    source_tx.send(Sequenced::with_timestamp(person_alice(), 2))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert!(matches!(item.value, TestData::Person(_)));

    filter_tx.send(Sequenced::with_timestamp(person_alice(), 3))?;
    source_tx.send(Sequenced::with_timestamp(person_alice(), 4))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert!(matches!(item.value, TestData::Person(_)));

    filter_tx.send(Sequenced::with_timestamp(person_alice(), 5))?;
    source_tx.send(Sequenced::with_timestamp(person_alice(), 6))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert!(matches!(item.value, TestData::Person(_)));

    drop(source_tx);
    drop(filter_tx);

    Ok(())
}
