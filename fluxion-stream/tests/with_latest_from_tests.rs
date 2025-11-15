// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::Ordered;
use fluxion_stream::CombinedState;
use fluxion_stream::with_latest_from::WithLatestFromExt;
use fluxion_test_utils::sequenced::Sequenced;
use fluxion_test_utils::test_data::{
    TestData, animal, animal_cat, animal_dog, person_alice, person_bob,
};
use fluxion_test_utils::with_timeout;
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

// Identity selector for testing - returns the CombinedState as-is
fn result_selector(state: &CombinedState<TestData>) -> CombinedState<TestData> {
    state.clone()
}

#[tokio::test]
async fn test_with_latest_from_basic() {
    with_timeout!({
        // Arrange
        let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
        let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

        let animal_stream = UnboundedReceiverStream::new(animal_rx);
        let person_stream = UnboundedReceiverStream::new(person_rx);

        let combined_stream = animal_stream.with_latest_from(person_stream, result_selector);
        let mut combined_stream = Box::pin(combined_stream);

        // Act - send secondary first (no emission yet)
        person_tx.send(Sequenced::new(person_alice())).unwrap();
        // Send primary - should emit now
        animal_tx.send(Sequenced::new(animal_cat())).unwrap();

        // Assert - should get combined state with both values
        let result = combined_stream.next().await.unwrap();
        let combined_state = result.get();

        // CombinedState order: [primary (index 0), secondary (index 1)]
        assert_eq!(combined_state.get_state()[0], animal_cat());
        assert_eq!(combined_state.get_state()[1], person_alice());

        // Act - primary emits again
        animal_tx.send(Sequenced::new(animal_dog())).unwrap();

        // Assert - should emit with latest secondary (still Alice)
        let result = combined_stream.next().await.unwrap();
        let combined_state = result.get();
        assert_eq!(combined_state.get_state()[0], animal_dog());
        assert_eq!(combined_state.get_state()[1], person_alice());

        // Act - secondary emits Bob (should NOT emit because only primary triggers emissions)
        person_tx.send(Sequenced::new(person_bob())).unwrap();

        // Assert - should not have received anything
        assert!(
            combined_stream.next().now_or_never().is_none(),
            "Should not emit when only secondary stream emits"
        );
    });
}

#[tokio::test]
async fn test_with_latest_from_ordering_preserved() {
    with_timeout!({
        // Arrange
        let (primary_tx, primary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
        let (secondary_tx, secondary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

        let primary_stream = UnboundedReceiverStream::new(primary_rx);
        let secondary_stream = UnboundedReceiverStream::new(secondary_rx);

        let combined_stream = primary_stream.with_latest_from(secondary_stream, result_selector);
        let mut combined_stream = Box::pin(combined_stream);

        // Act - interleave emissions to test ordering
        secondary_tx.send(Sequenced::new(person_alice())).unwrap();
        primary_tx.send(Sequenced::new(animal_cat())).unwrap();
        primary_tx.send(Sequenced::new(animal_dog())).unwrap();
        secondary_tx.send(Sequenced::new(person_bob())).unwrap(); // should NOT emit

        // Assert - should get two emissions in order
        let result1 = combined_stream.next().await.unwrap();
        assert_eq!(result1.get().get_state()[0], animal_cat());
        assert_eq!(result1.get().get_state()[1], person_alice());

        let result2 = combined_stream.next().await.unwrap();
        assert_eq!(result2.get().get_state()[0], animal_dog());
        assert_eq!(result2.get().get_state()[1], person_alice());

        // Verify result2 comes after result1
        assert!(
            result2.order() > result1.order(),
            "Second emission should have higher order"
        );

        // No more emissions (Bob didn't trigger any)
        assert!(combined_stream.next().now_or_never().is_none());
    });
}

#[tokio::test]
async fn test_with_latest_from_custom_selector() {
    // Arrange
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let animal_stream = UnboundedReceiverStream::new(animal_rx);
    let person_stream = UnboundedReceiverStream::new(person_rx);

    // Custom selector that extracts just one field (e.g., the primary value as a String)
    let custom_selector = |state: &CombinedState<TestData>| {
        format!("{:?}", state.get_state()[0]) // Primary value (index 0) as string
    };

    let combined_stream = animal_stream.with_latest_from(person_stream, custom_selector);
    let mut combined_stream = Box::pin(combined_stream);

    // Act
    person_tx.send(Sequenced::new(person_alice())).unwrap();
    animal_tx.send(Sequenced::new(animal_cat())).unwrap();

    // Assert - should get a String result
    let result = combined_stream.next().await.unwrap();
    let string_result = result.get();

    // The result should be the string representation of animal_cat
    assert!(
        string_result.contains("Cat"),
        "Expected 'Cat' in result: {}",
        string_result
    );
}

#[tokio::test]
async fn test_with_latest_from_secondary_emits_first_no_output() {
    // Arrange
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (_person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let animal_stream = UnboundedReceiverStream::new(animal_rx);
    let person_stream = UnboundedReceiverStream::new(person_rx);

    let combined_stream = animal_stream.with_latest_from(person_stream, result_selector);
    let mut combined_stream = Box::pin(combined_stream);

    // Act - primary emits but no secondary value yet
    animal_tx.send(Sequenced::new(animal_cat())).unwrap();

    // Assert - should not emit because secondary hasn't emitted
    assert!(combined_stream.next().now_or_never().is_none());
}

#[tokio::test]
async fn test_with_latest_from_secondary_completes_early() {
    with_timeout!({
        // Arrange
        let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
        let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

        let animal_stream = UnboundedReceiverStream::new(animal_rx);
        let person_stream = UnboundedReceiverStream::new(person_rx);

        let combined_stream = animal_stream.with_latest_from(person_stream, result_selector);
        let mut combined_stream = Box::pin(combined_stream);

        // Act - secondary emits and completes
        person_tx.send(Sequenced::new(person_alice())).unwrap();
        drop(person_tx); // secondary completes

        // Primary emits
        animal_tx.send(Sequenced::new(animal_cat())).unwrap();

        // Assert - should still emit with latest secondary value (Alice)
        let result = combined_stream.next().await.unwrap();
        assert_eq!(result.get().get_state()[0], animal_cat());
        assert_eq!(result.get().get_state()[1], person_alice());

        // Primary emits again
        animal_tx.send(Sequenced::new(animal_dog())).unwrap();

        // Assert - should still emit with same secondary value
        let result = combined_stream.next().await.unwrap();
        assert_eq!(result.get().get_state()[0], animal_dog());
        assert_eq!(result.get().get_state()[1], person_alice());
    });
}

#[tokio::test]
async fn test_with_latest_from_primary_completes_early() {
    with_timeout!({
        // Arrange
        let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
        let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

        let animal_stream = UnboundedReceiverStream::new(animal_rx);
        let person_stream = UnboundedReceiverStream::new(person_rx);

        let combined_stream = animal_stream.with_latest_from(person_stream, result_selector);
        let mut combined_stream = Box::pin(combined_stream);

        // Act
        person_tx.send(Sequenced::new(person_alice())).unwrap();
        animal_tx.send(Sequenced::new(animal_cat())).unwrap();

        // Assert
        let result = combined_stream.next().await.unwrap();
        assert_eq!(result.get().get_state()[0], animal_cat());
        assert_eq!(result.get().get_state()[1], person_alice());

        // Act - primary completes
        drop(animal_tx);
        person_tx.send(Sequenced::new(person_bob())).unwrap();

        // The stream won't complete until the secondary also completes (ordered_merge behavior)
        // So we need to close the secondary stream too
        drop(person_tx);

        // Now the stream should complete
        assert!(combined_stream.next().await.is_none());
    });
}

#[tokio::test]
async fn test_with_latest_from_large_number_of_emissions() {
    with_timeout!({
        // Arrange
        let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
        let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

        let animal_stream = UnboundedReceiverStream::new(animal_rx);
        let person_stream = UnboundedReceiverStream::new(person_rx);

        let combined_stream = animal_stream.with_latest_from(person_stream, result_selector);
        let mut combined_stream = Box::pin(combined_stream);

        // Act - send secondary first
        person_tx.send(Sequenced::new(person_alice())).unwrap();

        // Send many primary emissions
        for i in 0..100 {
            animal_tx
                .send(Sequenced::new(animal(format!("Animal{}", i), 4)))
                .unwrap();
        }

        // Assert - should get 100 emissions, all with Alice
        for i in 0..100 {
            let result = combined_stream.next().await.unwrap();
            let state = result.get();
            assert_eq!(state.get_state()[1], person_alice());
            if let TestData::Animal(ref animal) = state.get_state()[0] {
                assert_eq!(animal.name, format!("Animal{}", i));
            } else {
                panic!("Expected Animal");
            }
        }
    });
}

#[tokio::test]
async fn test_with_latest_from_both_streams_close_before_emission() {
    with_timeout!({
        // Arrange
        let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
        let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

        let animal_stream = UnboundedReceiverStream::new(animal_rx);
        let person_stream = UnboundedReceiverStream::new(person_rx);

        let combined_stream = animal_stream.with_latest_from(person_stream, result_selector);
        let mut combined_stream = Box::pin(combined_stream);

        // Act - close both streams without emissions
        drop(animal_tx);
        drop(person_tx);

        // Assert - stream should complete with no emissions
        assert!(combined_stream.next().await.is_none());
    });
}

#[tokio::test]
async fn test_with_latest_from_secondary_updates_latest() {
    with_timeout!({
        // Arrange
        let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
        let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

        let animal_stream = UnboundedReceiverStream::new(animal_rx);
        let person_stream = UnboundedReceiverStream::new(person_rx);

        let combined_stream = animal_stream.with_latest_from(person_stream, result_selector);
        let mut combined_stream = Box::pin(combined_stream);

        // Act
        person_tx.send(Sequenced::new(person_alice())).unwrap();
        animal_tx.send(Sequenced::new(animal_cat())).unwrap();

        // Assert - first emission with Alice
        let result = combined_stream.next().await.unwrap();
        assert_eq!(result.get().get_state()[0], animal_cat());
        assert_eq!(result.get().get_state()[1], person_alice());

        // Act - secondary updates to Bob (no emission yet)
        person_tx.send(Sequenced::new(person_bob())).unwrap();
        assert!(combined_stream.next().now_or_never().is_none());

        // Act - primary emits again
        animal_tx.send(Sequenced::new(animal_dog())).unwrap();

        // Assert - should emit with latest secondary (Bob)
        let result = combined_stream.next().await.unwrap();
        assert_eq!(result.get().get_state()[0], animal_dog());
        assert_eq!(result.get().get_state()[1], person_bob());
    });
}

#[tokio::test]
async fn test_with_latest_from_multiple_concurrent_streams() {
    with_timeout!({
        // Test that multiple independent with_latest_from streams work correctly
        let (animal_tx1, animal_rx1) = mpsc::unbounded_channel::<Sequenced<TestData>>();
        let (person_tx1, person_rx1) = mpsc::unbounded_channel::<Sequenced<TestData>>();
        let (animal_tx2, animal_rx2) = mpsc::unbounded_channel::<Sequenced<TestData>>();
        let (person_tx2, person_rx2) = mpsc::unbounded_channel::<Sequenced<TestData>>();

        let stream1 = UnboundedReceiverStream::new(animal_rx1)
            .with_latest_from(UnboundedReceiverStream::new(person_rx1), result_selector);
        let stream2 = UnboundedReceiverStream::new(animal_rx2)
            .with_latest_from(UnboundedReceiverStream::new(person_rx2), result_selector);

        let mut stream1 = Box::pin(stream1);
        let mut stream2 = Box::pin(stream2);

        // Emit to stream1
        person_tx1.send(Sequenced::new(person_alice())).unwrap();
        animal_tx1.send(Sequenced::new(animal_cat())).unwrap();

        // Emit to stream2
        person_tx2.send(Sequenced::new(person_bob())).unwrap();
        animal_tx2.send(Sequenced::new(animal_dog())).unwrap();

        // Assert stream1
        let result1 = stream1.next().await.unwrap();
        assert_eq!(result1.get().get_state()[0], animal_cat());
        assert_eq!(result1.get().get_state()[1], person_alice());

        // Assert stream2
        let result2 = stream2.next().await.unwrap();
        assert_eq!(result2.get().get_state()[0], animal_dog());
        assert_eq!(result2.get().get_state()[1], person_bob());
    });
}
