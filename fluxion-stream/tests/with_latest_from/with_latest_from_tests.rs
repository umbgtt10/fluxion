// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{HasTimestamp, Timestamped};
use fluxion_stream::with_latest_from::WithLatestFromExt;
use fluxion_stream::CombinedState;
use fluxion_test_utils::helpers::{
    assert_no_element_emitted, assert_stream_ended, test_channel, unwrap_stream, unwrap_value,
};
use fluxion_test_utils::sequenced::Sequenced;
use fluxion_test_utils::test_data::{
    animal, animal_cat, animal_dog, person_alice, person_bob, TestData,
};
use fluxion_test_utils::test_wrapper::TestWrapper;
use futures::{FutureExt, StreamExt};

fn result_selector(state: &CombinedState<TestData, u64>) -> CombinedState<TestData, u64> {
    state.clone()
}

#[tokio::test]
async fn test_with_latest_from_basic() -> anyhow::Result<()> {
    // Arrange
    let (animal_tx, animal_stream) = test_channel();
    let (person_tx, person_stream) = test_channel();

    let mut result = animal_stream.with_latest_from(person_stream, result_selector);

    // Act
    person_tx.unbounded_send(Sequenced::new(person_alice()))?;
    animal_tx.unbounded_send(Sequenced::new(animal_cat()))?;

    // Assert
    let element = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(element.clone().into_inner().values()[0], animal_cat());
    assert_eq!(element.clone().into_inner().values()[1], person_alice());

    // Act
    animal_tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Assert
    let element = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(element.clone().into_inner().values()[0], animal_dog());
    assert_eq!(element.clone().into_inner().values()[1], person_alice());

    // Act
    person_tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert!(result.next().now_or_never().is_none());

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_ordering_preserved() -> anyhow::Result<()> {
    // Arrange
    let (primary_tx, primary_stream) = test_channel();
    let (secondary_tx, secondary_stream) = test_channel();

    let mut result = primary_stream.with_latest_from(secondary_stream, result_selector);

    // Act
    secondary_tx.unbounded_send(Sequenced::new(person_alice()))?;
    primary_tx.unbounded_send(Sequenced::new(animal_cat()))?;
    primary_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    secondary_tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    let element1 = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(element1.clone().into_inner().values()[0], animal_cat());
    assert_eq!(element1.clone().into_inner().values()[1], person_alice());

    let element2 = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(element2.clone().into_inner().values()[0], animal_dog());
    assert_eq!(element2.clone().into_inner().values()[1], person_alice());
    assert!(element2.timestamp() > element1.timestamp());
    assert_no_element_emitted(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_custom_selector() -> anyhow::Result<()> {
    // Arrange
    let (animal_tx, animal_stream) = test_channel();
    let (person_tx, person_stream) = test_channel();

    let custom_selector = |state: &CombinedState<TestData, u64>| {
        let description = format!("{:?}", state.values()[0]);
        TestWrapper::new(description, state.timestamp())
    };

    let mut result = animal_stream.with_latest_from(person_stream, custom_selector);

    // Act
    person_tx.unbounded_send(Sequenced::new(person_alice()))?;
    animal_tx.unbounded_send(Sequenced::new(animal_cat()))?;

    // Assert
    assert!(unwrap_value(Some(unwrap_stream(&mut result, 500).await))
        .value()
        .contains("Cat"));

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_secondary_emits_first_no_output() -> anyhow::Result<()> {
    // Arrange
    let (animal_tx, animal_stream) = test_channel();
    let (_person_tx, person_stream) = test_channel();

    let mut result = animal_stream.with_latest_from(person_stream, result_selector);

    // Act
    animal_tx.unbounded_send(Sequenced::new(animal_cat()))?;

    // Assert
    assert!(result.next().now_or_never().is_none());

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_secondary_completes_early() -> anyhow::Result<()> {
    // Arrange
    let (animal_tx, animal_stream) = test_channel();
    let (person_tx, person_stream) = test_channel();

    let mut result = animal_stream.with_latest_from(person_stream, result_selector);

    // Act
    person_tx.unbounded_send(Sequenced::new(person_alice()))?;
    drop(person_tx);
    animal_tx.unbounded_send(Sequenced::new(animal_cat()))?;

    // Assert
    let element = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(element.clone().into_inner().values()[0], animal_cat());
    assert_eq!(element.clone().into_inner().values()[1], person_alice());

    // Act
    animal_tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Assert
    let element = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(element.clone().into_inner().values()[0], animal_dog());
    assert_eq!(element.clone().into_inner().values()[1], person_alice());

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_primary_completes_early() -> anyhow::Result<()> {
    // Arrange
    let (animal_tx, animal_stream) = test_channel();
    let (person_tx, person_stream) = test_channel();

    let mut result = animal_stream.with_latest_from(person_stream, result_selector);

    // Act
    person_tx.unbounded_send(Sequenced::new(person_alice()))?;
    animal_tx.unbounded_send(Sequenced::new(animal_cat()))?;

    // Assert
    let element = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(element.clone().into_inner().values()[0], animal_cat());
    assert_eq!(element.clone().into_inner().values()[1], person_alice());

    // Act
    drop(animal_tx);
    person_tx.unbounded_send(Sequenced::new(person_bob()))?;

    drop(person_tx);

    // Assert
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_large_number_of_emissions() -> anyhow::Result<()> {
    // Arrange
    let (animal_tx, animal_stream) = test_channel();
    let (person_tx, person_stream) = test_channel();

    let mut result = animal_stream.with_latest_from(person_stream, result_selector);

    // Act
    person_tx.unbounded_send(Sequenced::new(person_alice()))?;
    for i in 0..100 {
        animal_tx.unbounded_send(Sequenced::new(animal(format!("Animal{}", i), 4)))?;
    }

    // Assert
    for i in 0..100 {
        let element = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
        assert_eq!(element.clone().into_inner().values()[1], person_alice());
        if let TestData::Animal(ref animal) = element.clone().into_inner().values()[0] {
            assert_eq!(animal.species, format!("Animal{}", i));
        } else {
            panic!("Expected Animal");
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_both_streams_close_before_emission() -> anyhow::Result<()> {
    // Arrange
    let (animal_tx, animal_stream) = test_channel::<Sequenced<TestData>>();
    let (person_tx, person_stream) = test_channel::<Sequenced<TestData>>();

    let mut result = animal_stream.with_latest_from(person_stream, result_selector);

    // Act
    drop(animal_tx);
    drop(person_tx);

    // Assert
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_secondary_updates_latest() -> anyhow::Result<()> {
    // Arrange
    let (animal_tx, animal_stream) = test_channel();
    let (person_tx, person_stream) = test_channel();

    let mut result = animal_stream.with_latest_from(person_stream, result_selector);

    // Act
    person_tx.unbounded_send(Sequenced::new(person_alice()))?;
    animal_tx.unbounded_send(Sequenced::new(animal_cat()))?;

    // Assert
    let element = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(element.clone().into_inner().values()[0], animal_cat());
    assert_eq!(element.clone().into_inner().values()[1], person_alice());

    // Act
    person_tx.unbounded_send(Sequenced::new(person_bob()))?;
    assert!(result.next().now_or_never().is_none());

    animal_tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Assert
    let element = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(element.clone().into_inner().values()[0], animal_dog());
    assert_eq!(element.clone().into_inner().values()[1], person_bob());

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_multiple_concurrent_streams() -> anyhow::Result<()> {
    // Arrange
    let (animal_tx1, animal_stream1) = test_channel();
    let (person_tx1, person_stream1) = test_channel();
    let (animal_tx2, animal_stream2) = test_channel();
    let (person_tx2, person_stream2) = test_channel();

    let stream1 = animal_stream1.with_latest_from(person_stream1, result_selector);
    let stream2 = animal_stream2.with_latest_from(person_stream2, result_selector);

    let mut stream1 = stream1;
    let mut stream2 = stream2;

    // Act
    person_tx1.unbounded_send(Sequenced::new(person_alice()))?;
    animal_tx1.unbounded_send(Sequenced::new(animal_cat()))?;

    person_tx2.unbounded_send(Sequenced::new(person_bob()))?;
    animal_tx2.unbounded_send(Sequenced::new(animal_dog()))?;

    // Assert
    let element1 = unwrap_value(Some(unwrap_stream(&mut stream1, 500).await));
    assert_eq!(element1.clone().into_inner().values()[0], animal_cat());
    assert_eq!(element1.clone().into_inner().values()[1], person_alice());

    let element2 = unwrap_value(Some(unwrap_stream(&mut stream2, 500).await));
    assert_eq!(element2.clone().into_inner().values()[0], animal_dog());
    assert_eq!(element2.clone().into_inner().values()[1], person_bob());

    Ok(())
}
