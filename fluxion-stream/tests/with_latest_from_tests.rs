// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::combine_latest::CombinedState;
use fluxion_stream::with_latest_from::WithLatestFromExt;
use fluxion_test_utils::test_data::{
    TestData, animal, animal_cat, animal_dog, person, person_alice, person_bob, person_charlie,
};
use fluxion_test_utils::helpers::{assert_no_element_emitted, expect_next_pair_unchecked};
use fluxion_test_utils::sequenced::Sequenced;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use futures::StreamExt;

static FILTER: fn(&CombinedState<TestData>) -> bool = |_: &CombinedState<TestData>| true;

#[tokio::test]
async fn test_with_latest_from_complete() {
    // Arrange
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let animal_stream = UnboundedReceiverStream::new(animal_rx);
    let person_stream = UnboundedReceiverStream::new(person_rx);

    let combined_stream = animal_stream.with_latest_from(person_stream, FILTER);

    // Act
    animal_tx.send(Sequenced::new(animal_cat())).unwrap();
    person_tx.send(Sequenced::new(person_alice())).unwrap();

    // Assert
    let mut combined_stream = Box::pin(combined_stream);

    expect_next_pair_unchecked(&mut combined_stream, person_alice(), animal_cat()).await;

    // Act
    animal_tx.send(Sequenced::new(animal_dog())).unwrap();

    // Assert
    expect_next_pair_unchecked(&mut combined_stream, person_alice(), animal_dog()).await;

    // Act
    person_tx.send(Sequenced::new(person_bob())).unwrap();

    // Assert
    expect_next_pair_unchecked(&mut combined_stream, person_bob(), animal_dog()).await;
}

#[tokio::test]
async fn test_with_latest_from_second_stream_does_not_emit_no_output() {
    // Arrange
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (_person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let animal_stream = UnboundedReceiverStream::new(animal_rx);
    let person_stream = UnboundedReceiverStream::new(person_rx);

    let combined_stream = animal_stream.with_latest_from(person_stream, FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act
    animal_tx.send(Sequenced::new(animal_cat())).unwrap();

    // Assert
    assert_no_element_emitted(&mut combined_stream, 100).await;
}

#[tokio::test]
async fn test_with_latest_from_primary_waits_for_secondary() {
    // Arrange: Secondary must publish first for any emission
    let (primary_tx, primary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (secondary_tx, secondary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let primary_stream = UnboundedReceiverStream::new(primary_rx);
    let secondary_stream = UnboundedReceiverStream::new(secondary_rx);

    let combined_stream = primary_stream.with_latest_from(secondary_stream, FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act: Primary publishes before secondary
    primary_tx.send(Sequenced::new(animal_cat())).unwrap();

    // Assert: No emission yet
    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Act: Secondary publishes
    secondary_tx.send(Sequenced::new(person_alice())).unwrap();

    // Assert: Now emission occurs pairing latest secondary with primary
    expect_next_pair_unchecked(&mut combined_stream, person_alice(), animal_cat()).await;
}

#[tokio::test]
async fn test_with_latest_from_secondary_completes_early() {
    // Arrange
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let animal_stream = UnboundedReceiverStream::new(animal_rx);
    let person_stream = UnboundedReceiverStream::new(person_rx);

    let combined_stream = animal_stream.with_latest_from(person_stream, FILTER);

    // Act
    person_tx.send(Sequenced::new(person_alice())).unwrap();
    drop(person_tx);

    // Assert
    let mut combined_stream = Box::pin(combined_stream);

    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Act
    animal_tx.send(Sequenced::new(animal_cat())).unwrap();

    // Assert
    expect_next_pair_unchecked(&mut combined_stream, person_alice(), animal_cat()).await;

    // Act
    animal_tx.send(Sequenced::new(animal_dog())).unwrap();

    // Assert
    expect_next_pair_unchecked(&mut combined_stream, person_alice(), animal_dog()).await;
}

#[tokio::test]
async fn test_with_latest_from_primary_completes_early() {
    // Arrange
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let animal_stream = UnboundedReceiverStream::new(animal_rx);
    let person_stream = UnboundedReceiverStream::new(person_rx);

    let combined_stream = animal_stream.with_latest_from(person_stream, FILTER);

    // Act
    animal_tx.send(Sequenced::new(animal_cat())).unwrap();
    person_tx.send(Sequenced::new(person_alice())).unwrap();

    // Assert
    let mut combined_stream = Box::pin(combined_stream);

    expect_next_pair_unchecked(&mut combined_stream, person_alice(), animal_cat()).await;

    // Act
    drop(animal_tx);
    person_tx.send(Sequenced::new(person_bob())).unwrap();

    // Assert
    expect_next_pair_unchecked(&mut combined_stream, person_bob(), animal_cat()).await;
}

#[tokio::test]
async fn test_large_number_of_emissions() {
    // Arrange
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let animal_stream = UnboundedReceiverStream::new(animal_rx);
    let person_stream = UnboundedReceiverStream::new(person_rx);

    let combined_stream = animal_stream.with_latest_from(person_stream, FILTER);

    // Act
    person_tx.send(Sequenced::new(person_alice())).unwrap();

    for i in 0..1000 {
        animal_tx.send(Sequenced::new(animal(format!("Animal{i}"), 4))).unwrap();
    }

    // Assert
    let mut combined_stream = Box::pin(combined_stream);

    for i in 0..1000 {
        let (p, a) = combined_stream.next().await.unwrap();
        assert_eq!(
            (p.value, a.value),
            (person_alice(), animal(format!("Animal{i}"), 4))
        );
    }
}

#[tokio::test]
async fn test_with_latest_from_rapid_primary_only_updates() {
    // Arrange: Test that rapid primary emissions pair with the latest secondary value
    let (primary_tx, primary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (secondary_tx, secondary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let primary_stream = UnboundedReceiverStream::new(primary_rx);
    let secondary_stream = UnboundedReceiverStream::new(secondary_rx);

    let combined_stream = primary_stream.with_latest_from(secondary_stream, FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act: Secondary publishes once
    secondary_tx.send(Sequenced::new(person_alice())).unwrap();

    // Act: Primary publishes rapidly multiple times (secondary stays Alice)
    primary_tx.send(Sequenced::new(animal_dog())).unwrap();
    primary_tx.send(Sequenced::new(animal_cat())).unwrap();

    // Assert: Each primary emission pairs with the same latest secondary value (Alice)
    // Note: Order is (secondary, primary) = (person, animal)
    expect_next_pair_unchecked(&mut combined_stream, person_alice(), animal_dog()).await;
    expect_next_pair_unchecked(&mut combined_stream, person_alice(), animal_cat()).await;

    // Act: Another rapid sequence of primary emissions (secondary still Alice)
    for i in 0..10 {
        primary_tx.send(Sequenced::new(animal(format!("Animal{i}"), 4))).unwrap();
    }

    // Assert: All primary emissions pair with Alice
    for i in 0..10 {
        let (p, a) = combined_stream.next().await.unwrap();
        assert_eq!(
            (p.value, a.value),
            (person_alice(), animal(format!("Animal{i}"), 4))
        );
    }
}

#[tokio::test]
async fn test_with_latest_from_both_streams_close_before_any_emission() {
    // Arrange: Both streams close without the secondary ever publishing
    let (primary_tx, primary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (secondary_tx, secondary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let primary_stream = UnboundedReceiverStream::new(primary_rx);
    let secondary_stream = UnboundedReceiverStream::new(secondary_rx);

    let combined_stream = primary_stream.with_latest_from(secondary_stream, FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act: Primary publishes but secondary never does
    primary_tx.send(Sequenced::new(animal_dog())).unwrap();

    // Assert: No emission yet (waiting for secondary)
    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Act: Close both streams by dropping senders
    drop(primary_tx);
    drop(secondary_tx);

    // Assert: Stream completes with no emissions
    let next_item = combined_stream.next().await;
    assert!(
        next_item.is_none(),
        "Expected stream to close with no emissions when both streams close before secondary publishes"
    );
}

#[tokio::test]
async fn test_with_latest_from_boundary_empty_string_zero_values() {
    // Arrange: Test with boundary values - empty strings and zero values
    // This test verifies the system handles edge cases like:
    // - Empty string names
    // - Zero numeric values (age=0, legs=0)
    // - Transitions from boundary to normal values
    let (primary_tx, primary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (secondary_tx, secondary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let primary_stream = UnboundedReceiverStream::new(primary_rx);
    let secondary_stream = UnboundedReceiverStream::new(secondary_rx);

    let combined_stream = primary_stream.with_latest_from(secondary_stream, FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act: Send empty string person to secondary (boundary: empty string, zero age)
    secondary_tx.send(Sequenced::new(person(String::new(), 0))).unwrap();

    // Act: Send zero-leg animal to primary (boundary: empty string, zero legs)
    // This triggers first emission with both boundary values
    primary_tx.send(Sequenced::new(animal(String::new(), 0))).unwrap();

    // Assert: System handles empty/zero values correctly
    let (sec, prim) = combined_stream.next().await.unwrap();
    assert_eq!(
        sec.value,
        person(String::new(), 0),
        "Secondary should have empty name and zero age"
    );
    assert_eq!(
        prim.value,
        animal(String::new(), 0),
        "Primary should have empty name and zero legs"
    );

    // Act: Send normal non-boundary values
    secondary_tx.send(Sequenced::new(person(String::from("Valid"), 1))).unwrap();

    // Assert: Updating secondary causes emission (with_latest_from uses combine_latest)
    let (sec2, prim2) = combined_stream.next().await.unwrap();
    assert_eq!(
        sec2.value,
        person(String::from("Valid"), 1),
        "Secondary should update to valid person"
    );
    assert_eq!(
        prim2.value,
        animal(String::new(), 0),
        "Primary should still be boundary animal"
    );

    // Act: Now update primary to normal value
    primary_tx.send(Sequenced::new(animal(String::from("ValidAnimal"), 1))).unwrap();

    // Assert: Both are now non-boundary values
    let (sec3, prim3) = combined_stream.next().await.unwrap();
    assert_eq!(
        sec3.value,
        person(String::from("Valid"), 1),
        "Secondary should remain valid person"
    );
    assert_eq!(
        prim3.value,
        animal(String::from("ValidAnimal"), 1),
        "Primary should update to valid animal"
    );
}

#[tokio::test]
async fn test_with_latest_from_boundary_maximum_concurrent_streams() {
    // Arrange: Test boundary of maximum concurrent stream operations
    // Create many parallel with_latest_from streams to test concurrent handling
    let num_stream_pairs = 50;
    let mut handles = Vec::new();

    for _ in 0..num_stream_pairs {
        let handle = tokio::spawn(async {
            let (primary_tx, primary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
            let (secondary_tx, secondary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
            let primary_stream = UnboundedReceiverStream::new(primary_rx);
            let secondary_stream = UnboundedReceiverStream::new(secondary_rx);
            let combined = primary_stream.with_latest_from(secondary_stream, FILTER);
            let mut stream = Box::pin(combined);

            // Act: Send initial values
            secondary_tx.send(Sequenced::new(person_alice())).unwrap();
            primary_tx.send(Sequenced::new(animal_dog())).unwrap();

            // Assert: Verify first emission
            let (sec, prim) = stream.next().await.unwrap();
            assert_eq!(sec.value, person_alice());
            assert_eq!(prim.value, animal_dog());

            // Act: Update both values
            secondary_tx.send(Sequenced::new(person_bob())).unwrap();

            // Assert: Secondary update causes emission
            let (sec2, prim2) = stream.next().await.unwrap();
            assert_eq!(sec2.value, person_bob());
            assert_eq!(prim2.value, animal_dog()); // Primary unchanged

            // Act: Update primary
            primary_tx.send(Sequenced::new(animal_cat())).unwrap();

            // Assert: Primary update causes emission
            let (sec3, prim3) = stream.next().await.unwrap();
            assert_eq!(sec3.value, person_bob());
            assert_eq!(prim3.value, animal_cat());
        });

        handles.push(handle);
    }

    // Wait for all concurrent streams to complete successfully
    for handle in handles {
        handle
            .await
            .expect("Stream task should complete successfully");
    }
}

#[tokio::test]
async fn test_with_latest_from_filter_rejects_initial_state() {
    // Arrange
    let (primary_tx, primary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (secondary_tx, secondary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    // Filter that rejects when primary is Dog
    let filter = |state: &CombinedState<TestData>| {
        let values = state.get_state();
        values[1] != animal_dog() // Reject when primary is Dog
    };

    let primary_stream = UnboundedReceiverStream::new(primary_rx);
    let secondary_stream = UnboundedReceiverStream::new(secondary_rx);

    let combined_stream = primary_stream.with_latest_from(secondary_stream, filter);
    let mut stream = Box::pin(combined_stream);

    // Act: Send Dog to primary, Alice to secondary (filter should reject)
    primary_tx.send(Sequenced::new(animal_dog())).unwrap();
    secondary_tx.send(Sequenced::new(person_alice())).unwrap();

    // Assert: No emission because filter rejects
    assert_no_element_emitted(&mut stream, 100).await;

    // Act: Send Cat to primary (filter should now accept)
    primary_tx.send(Sequenced::new(animal_cat())).unwrap();

    // Assert: Now it should emit
    let (sec, prim) = stream.next().await.unwrap();
    assert_eq!(sec.get(), &person_alice());
    assert_eq!(prim.get(), &animal_cat());
}

#[tokio::test]
async fn test_with_latest_from_filter_alternates() {
    // Arrange
    let (primary_tx, primary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (secondary_tx, secondary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    // Counter to alternate filter behavior
    let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter_clone = counter.clone();

    // Filter alternates: true, false, true, false, ...
    let filter = move |_: &CombinedState<TestData>| {
        let count = counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        count.is_multiple_of(2) // true for even counts, false for odd
    };

    let primary_stream = UnboundedReceiverStream::new(primary_rx);
    let secondary_stream = UnboundedReceiverStream::new(secondary_rx);

    let combined_stream = primary_stream.with_latest_from(secondary_stream, filter);
    let mut stream = Box::pin(combined_stream);

    // Act: Initial values (count=0, filter=true)
    primary_tx.send(Sequenced::new(animal_dog())).unwrap();
    secondary_tx.send(Sequenced::new(person_alice())).unwrap();

    // Assert: Should emit (filter returns true)
    let (sec1, prim1) = stream.next().await.unwrap();
    assert_eq!(sec1.value, person_alice());
    assert_eq!(prim1.value, animal_dog());

    // Act: Update primary (count=1, filter=false)
    primary_tx.send(Sequenced::new(animal_cat())).unwrap();

    // Assert: Should not emit (filter returns false)
    assert_no_element_emitted(&mut stream, 100).await;

    // Act: Update secondary (count=2, filter=true)
    secondary_tx.send(Sequenced::new(person_bob())).unwrap();

    // Assert: Should emit (filter returns true)
    let (sec2, prim2) = stream.next().await.unwrap();
    assert_eq!(sec2.value, person_bob());
    assert_eq!(prim2.value, animal_cat());

    // Act: Update primary (count=3, filter=false)
    primary_tx.send(Sequenced::new(animal_dog())).unwrap();

    // Assert: Should not emit (filter returns false)
    assert_no_element_emitted(&mut stream, 100).await;
}

#[tokio::test]
#[should_panic(expected = "Filter panicked")]
async fn test_with_latest_from_filter_panics() {
    // Arrange
    let (primary_tx, primary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (secondary_tx, secondary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    // Filter that panics
    let filter = |_: &CombinedState<TestData>| -> bool {
        panic!("Filter panicked");
    };

    let primary_stream = UnboundedReceiverStream::new(primary_rx);
    let secondary_stream = UnboundedReceiverStream::new(secondary_rx);

    let combined_stream = primary_stream.with_latest_from(secondary_stream, filter);
    let mut stream = Box::pin(combined_stream);

    // Act: Send values to trigger filter
    primary_tx.send(Sequenced::new(animal_dog())).unwrap();
    secondary_tx.send(Sequenced::new(person_alice())).unwrap();

    // This should panic when filter is called
    let _ = stream.next().await;
}

#[tokio::test]
async fn test_with_latest_from_timestamp_ordering() {
    // Arrange
    let (primary_tx, primary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (secondary_tx, secondary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let primary_stream = UnboundedReceiverStream::new(primary_rx);
    let secondary_stream = UnboundedReceiverStream::new(secondary_rx);

    let combined_stream = primary_stream.with_latest_from(secondary_stream, FILTER);
    let mut stream = Box::pin(combined_stream);

    // Act: Send initial values
    primary_tx.send(Sequenced::new(animal_dog())).unwrap();
    secondary_tx.send(Sequenced::new(person_alice())).unwrap();

    // Assert: First emission
    let (_sec1, prim1) = stream.next().await.unwrap();
    let ts1 = prim1.sequence();

    // Act: Update primary
    primary_tx.send(Sequenced::new(animal_cat())).unwrap();

    // Assert: Second emission should have later timestamp
    let (_sec2, prim2) = stream.next().await.unwrap();
    let ts2 = prim2.sequence();
    assert!(
        ts2 > ts1,
        "Timestamps should be monotonically increasing: {ts2:?} should be > {ts1:?}"
    );

    // Act: Update secondary
    secondary_tx.send(Sequenced::new(person_bob())).unwrap();

    // Assert: Third emission should have later timestamp
    let (sec3, _prim3) = stream.next().await.unwrap();
    let ts3 = sec3.sequence();
    assert!(
        ts3 > ts2,
        "Timestamps should be monotonically increasing: {ts3:?} should be > {ts2:?}"
    );
}

#[tokio::test]
async fn test_with_latest_from_multiple_secondary_values_before_primary() {
    // Arrange
    let (primary_tx, primary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (secondary_tx, secondary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let primary_stream = UnboundedReceiverStream::new(primary_rx);
    let secondary_stream = UnboundedReceiverStream::new(secondary_rx);

    let combined_stream = primary_stream.with_latest_from(secondary_stream, FILTER);
    let mut stream = Box::pin(combined_stream);

    // Act: Send multiple secondary values before any primary value
    secondary_tx.send(Sequenced::new(person_alice())).unwrap();
    secondary_tx.send(Sequenced::new(person_bob())).unwrap();
    secondary_tx.send(Sequenced::new(person_charlie())).unwrap();

    // Assert: No emission yet (waiting for primary)
    assert_no_element_emitted(&mut stream, 100).await;

    // Act: Send first primary value
    primary_tx.send(Sequenced::new(animal_dog())).unwrap();

    // Assert: Should emit with latest secondary value (Charlie)
    let (sec, prim) = stream.next().await.unwrap();
    assert_eq!(sec.value, person_charlie()); // Latest secondary
    assert_eq!(prim.value, animal_dog());

    // Act: Send another secondary value
    secondary_tx.send(Sequenced::new(person_alice())).unwrap();

    // Assert: Should emit immediately (both streams have values)
    let (sec2, prim2) = stream.next().await.unwrap();
    assert_eq!(sec2.value, person_alice());
    assert_eq!(prim2.value, animal_dog()); // Primary unchanged
}
