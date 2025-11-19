// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{Ordered, OrderedWrapper, StreamItem};
use fluxion_stream::with_latest_from::WithLatestFromExt;
use fluxion_stream::CombinedState;
use fluxion_test_utils::sequenced::Sequenced;
use fluxion_test_utils::test_data::{
    animal, animal_cat, animal_dog, person_alice, person_bob, TestData,
};
use fluxion_test_utils::{assert_no_element_emitted, unwrap_value};
use fluxion_test_utils::{helpers::unwrap_stream, test_channel};
use futures::{FutureExt, Stream, StreamExt};

// Identity selector for testing - returns the CombinedState as-is
fn result_selector(state: &CombinedState<TestData>) -> CombinedState<TestData> {
    state.clone()
}

#[tokio::test]
async fn test_with_latest_from_basic() -> anyhow::Result<()> {
    // Arrange
    let (animal_tx, animal_stream) = test_channel();
    let (person_tx, person_stream) = test_channel();

    let mut combined_stream = animal_stream.with_latest_from(person_stream, result_selector);

    // Act
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_cat()))?;

    // Assert CombinedState order: [primary (index 0), secondary (index 1)]
    let element = extract_element(&mut combined_stream).await;
    assert_eq!(element.values()[0], animal_cat());
    assert_eq!(element.values()[1], person_alice());

    // Act - primary emits again
    animal_tx.send(Sequenced::new(animal_dog()))?;
    let element = extract_element(&mut combined_stream).await;
    assert_eq!(element.values()[0], animal_dog());
    assert_eq!(element.values()[1], person_alice());

    // Act - secondary emits Bob (should NOT emit because only primary triggers emissions)
    person_tx.send(Sequenced::new(person_bob()))?;
    assert!(
        combined_stream.next().now_or_never().is_none(),
        "Should not emit when only secondary stream emits"
    );

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_ordering_preserved() -> anyhow::Result<()> {
    // Arrange
    let (primary_tx, primary_stream) = test_channel();
    let (secondary_tx, secondary_stream) = test_channel();

    let mut combined_stream = primary_stream.with_latest_from(secondary_stream, result_selector);

    // Act - interleave emissions to test ordering
    secondary_tx.send(Sequenced::new(person_alice()))?;
    primary_tx.send(Sequenced::new(animal_cat()))?;
    primary_tx.send(Sequenced::new(animal_dog()))?;
    secondary_tx.send(Sequenced::new(person_bob()))?; // should NOT emit

    // Assert - should get two emissions in order
    let element1 = extract_element(&mut combined_stream).await;
    assert_eq!(element1.values()[0], animal_cat());
    assert_eq!(element1.values()[1], person_alice());

    let element2 = extract_element(&mut combined_stream).await;
    assert_eq!(element2.values()[0], animal_dog());
    assert_eq!(element2.values()[1], person_alice());

    // Verify element2 comes after element1
    assert!(
        element2.order() > element1.order(),
        "Second emission should have higher order"
    );

    // No more emissions (Bob didn't trigger any)
    assert_no_element_emitted(&mut combined_stream, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_custom_selector() -> anyhow::Result<()> {
    // Arrange
    let (animal_tx, animal_stream) = test_channel();
    let (person_tx, person_stream) = test_channel();

    // Custom selector that extracts just one field (e.g., the primary value as a String)
    let custom_selector = |state: &CombinedState<TestData>| {
        format!("{:?}", state.values()[0]) // Primary value (index 0) as string
    };

    let mut combined_stream = animal_stream.with_latest_from(person_stream, custom_selector);

    // Act
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_cat()))?;

    // Assert - should get a String result
    let element = extract_element(&mut combined_stream).await;
    assert!(
        element.contains("Cat"),
        "Expected 'Cat' in result: {}",
        element
    );

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_secondary_emits_first_no_output() -> anyhow::Result<()> {
    // Arrange
    let (animal_tx, animal_stream) = test_channel();
    let (_person_tx, person_stream) = test_channel();

    let mut combined_stream = animal_stream.with_latest_from(person_stream, result_selector);

    // Act - primary emits but no secondary value yet
    animal_tx.send(Sequenced::new(animal_cat()))?;

    // Assert - should not emit because secondary hasn't emitted
    assert!(combined_stream.next().now_or_never().is_none());

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_secondary_completes_early() -> anyhow::Result<()> {
    // Arrange
    let (animal_tx, animal_stream) = test_channel();
    let (person_tx, person_stream) = test_channel();

    let mut combined_stream = animal_stream.with_latest_from(person_stream, result_selector);

    // Act - secondary emits and completes
    person_tx.send(Sequenced::new(person_alice()))?;
    drop(person_tx); // secondary completes

    // Primary emits
    animal_tx.send(Sequenced::new(animal_cat()))?;

    // Assert - should still emit with latest secondary value (Alice)
    let element = extract_element(&mut combined_stream).await;
    assert_eq!(element.values()[0], animal_cat());
    assert_eq!(element.values()[1], person_alice());

    // Primary emits again
    animal_tx.send(Sequenced::new(animal_dog()))?;

    // Assert - should still emit with same secondary value
    let element = extract_element(&mut combined_stream).await;
    assert_eq!(element.values()[0], animal_dog());
    assert_eq!(element.values()[1], person_alice());

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_primary_completes_early() -> anyhow::Result<()> {
    // Arrange
    let (animal_tx, animal_stream) = test_channel();
    let (person_tx, person_stream) = test_channel();

    let mut combined_stream = animal_stream.with_latest_from(person_stream, result_selector);

    // Act
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_cat()))?;

    // Assert
    let element = extract_element(&mut combined_stream).await;
    assert_eq!(element.values()[0], animal_cat());
    assert_eq!(element.values()[1], person_alice());

    // Act - primary completes
    drop(animal_tx);
    person_tx.send(Sequenced::new(person_bob()))?;

    // The stream won't complete until the secondary also completes (ordered_merge behavior)
    // So we need to close the secondary stream too
    drop(person_tx);

    // Now the stream should complete
    assert!(combined_stream.next().await.is_none());

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_large_number_of_emissions() -> anyhow::Result<()> {
    // Arrange
    let (animal_tx, animal_stream) = test_channel();
    let (person_tx, person_stream) = test_channel();

    let mut combined_stream = animal_stream.with_latest_from(person_stream, result_selector);

    // Act - send secondary first
    person_tx.send(Sequenced::new(person_alice()))?;

    // Send many primary emissions
    for i in 0..100 {
        animal_tx.send(Sequenced::new(animal(format!("Animal{}", i), 4)))?;
    }

    // Assert - should get 100 emissions, all with Alice
    for i in 0..100 {
        let element = extract_element(&mut combined_stream).await;
        assert_eq!(element.values()[1], person_alice());
        if let TestData::Animal(ref animal) = element.values()[0] {
            assert_eq!(animal.name, format!("Animal{}", i));
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

    let mut combined_stream = animal_stream.with_latest_from(person_stream, result_selector);

    // Act - close both streams without emissions
    drop(animal_tx);
    drop(person_tx);

    // Assert - stream should complete with no emissions
    assert!(combined_stream.next().await.is_none());

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_secondary_updates_latest() -> anyhow::Result<()> {
    // Arrange
    let (animal_tx, animal_stream) = test_channel();
    let (person_tx, person_stream) = test_channel();

    let mut combined_stream = animal_stream.with_latest_from(person_stream, result_selector);

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_cat()))?;
    let element = extract_element(&mut combined_stream).await;
    assert_eq!(element.values()[0], animal_cat());
    assert_eq!(element.values()[1], person_alice());

    // Act - secondary updates to Bob (no emission yet)
    person_tx.send(Sequenced::new(person_bob()))?;
    assert!(combined_stream.next().now_or_never().is_none());

    // Act - primary emits again
    animal_tx.send(Sequenced::new(animal_dog()))?;
    // Assert - should emit with latest secondary (Bob)
    let element = extract_element(&mut combined_stream).await;
    assert_eq!(element.values()[0], animal_dog());
    assert_eq!(element.values()[1], person_bob());

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_multiple_concurrent_streams() -> anyhow::Result<()> {
    // Test that multiple independent with_latest_from streams work correctly
    let (animal_tx1, animal_stream1) = test_channel();
    let (person_tx1, person_stream1) = test_channel();
    let (animal_tx2, animal_stream2) = test_channel();
    let (person_tx2, person_stream2) = test_channel();

    let stream1 = animal_stream1.with_latest_from(person_stream1, result_selector);
    let stream2 = animal_stream2.with_latest_from(person_stream2, result_selector);

    let mut stream1 = stream1;
    let mut stream2 = stream2;

    // Emit to stream1
    person_tx1.send(Sequenced::new(person_alice()))?;
    animal_tx1.send(Sequenced::new(animal_cat()))?;

    // Emit to stream2
    person_tx2.send(Sequenced::new(person_bob()))?;
    animal_tx2.send(Sequenced::new(animal_dog()))?;

    // Assert stream1
    let element1 = extract_element(&mut stream1).await;
    assert_eq!(element1.values()[0], animal_cat());
    assert_eq!(element1.values()[1], person_alice());

    // Assert stream2
    let element2 = extract_element(&mut stream2).await;
    assert_eq!(element2.values()[0], animal_dog());
    assert_eq!(element2.values()[1], person_bob());

    Ok(())
}

async fn extract_element<'a, T, S>(stream: &'a mut S) -> &'a T
where
    S: Stream<Item = StreamItem<OrderedWrapper<T>>> + Unpin,
    T: Clone + 'a,
{
    let value = unwrap_value(Some(unwrap_stream(stream, 500).await));
    value.get()
}
