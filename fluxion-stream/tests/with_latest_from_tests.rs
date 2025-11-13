use fluxion_stream::combine_latest::CombinedState;
use fluxion_stream::with_latest_from::WithLatestFromExt;
use fluxion_test_utils::FluxionChannel;
use fluxion_test_utils::push;
use fluxion_test_utils::test_data::{
    TestData, animal, animal_cat, animal_dog, person, person_alice, person_bob, person_charlie,
};
use fluxion_test_utils::{
    TestChannels,
    helpers::{assert_no_element_emitted, expect_next_pair_unchecked},
};
use futures::StreamExt;

static FILTER: fn(&CombinedState<TestData>) -> bool = |_: &CombinedState<TestData>| true;

#[tokio::test]
async fn test_with_latest_from_complete() {
    // Arrange
    let animal = FluxionChannel::<TestData>::new();
    let person = FluxionChannel::<TestData>::new();

    let combined_stream = animal.stream.with_latest_from(person.stream, FILTER);

    // Act
    push(animal_cat(), &animal.sender);
    push(person_alice(), &person.sender);

    // Assert
    let mut combined_stream = Box::pin(combined_stream);

    expect_next_pair_unchecked(&mut combined_stream, person_alice(), animal_cat()).await;

    // Act
    push(animal_dog(), &animal.sender);

    // Assert
    expect_next_pair_unchecked(&mut combined_stream, person_alice(), animal_dog()).await;

    // Act
    push(person_bob(), &person.sender);

    // Assert
    expect_next_pair_unchecked(&mut combined_stream, person_bob(), animal_dog()).await;
}

#[tokio::test]
async fn test_with_latest_from_second_stream_does_not_emit_no_output() {
    // Arrange
    let animal = FluxionChannel::new();
    let person = FluxionChannel::new();

    let combined_stream = animal.stream.with_latest_from(person.stream, FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act
    push(animal_cat(), &animal.sender);

    // Assert
    assert_no_element_emitted(&mut combined_stream, 100).await;
}

#[tokio::test]
async fn test_with_latest_from_primary_waits_for_secondary() {
    // Arrange: Secondary must publish first for any emission
    let (primary, secondary) = TestChannels::two();

    let combined_stream = primary.stream.with_latest_from(secondary.stream, FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act: Primary publishes before secondary
    push(animal_cat(), &primary.sender);

    // Assert: No emission yet
    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Act: Secondary publishes
    push(person_alice(), &secondary.sender);

    // Assert: Now emission occurs pairing latest secondary with primary
    expect_next_pair_unchecked(&mut combined_stream, person_alice(), animal_cat()).await;
}

#[tokio::test]
async fn test_with_latest_from_secondary_completes_early() {
    // Arrange
    let (animal, person) = TestChannels::two::<TestData>();

    let combined_stream = animal.stream.with_latest_from(person.stream, FILTER);

    // Act
    push(person_alice(), &person.sender);
    drop(person.sender);

    // Assert
    let mut combined_stream = Box::pin(combined_stream);

    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Act
    push(animal_cat(), &animal.sender);

    // Assert
    expect_next_pair_unchecked(&mut combined_stream, person_alice(), animal_cat()).await;

    // Act
    push(animal_dog(), &animal.sender);

    // Assert
    expect_next_pair_unchecked(&mut combined_stream, person_alice(), animal_dog()).await;
}

#[tokio::test]
async fn test_with_latest_from_primary_completes_early() {
    // Arrange
    let (animal, person) = TestChannels::two();

    let combined_stream = animal.stream.with_latest_from(person.stream, FILTER);

    // Act
    push(animal_cat(), &animal.sender);
    push(person_alice(), &person.sender);

    // Assert
    let mut combined_stream = Box::pin(combined_stream);

    expect_next_pair_unchecked(&mut combined_stream, person_alice(), animal_cat()).await;

    // Act
    drop(animal.sender);
    push(person_bob(), &person.sender);

    // Assert
    expect_next_pair_unchecked(&mut combined_stream, person_bob(), animal_cat()).await;
}

#[tokio::test]
async fn test_large_number_of_emissions() {
    // Arrange
    let (animal_channel, person) = TestChannels::two();

    let combined_stream = animal_channel
        .stream
        .with_latest_from(person.stream, FILTER);

    // Act
    push(person_alice(), &person.sender);

    for i in 0..1000 {
        animal_channel
            .sender
            .send(animal(format!("Animal{}", i), 4))
            .unwrap();
    }

    // Assert
    let mut combined_stream = Box::pin(combined_stream);

    for i in 0..1000 {
        let (p, a) = combined_stream.next().await.unwrap();
        assert_eq!(
            (p.value, a.value),
            (person_alice(), animal(format!("Animal{}", i), 4))
        );
    }
}

#[tokio::test]
async fn test_with_latest_from_rapid_primary_only_updates() {
    // Arrange: Test that rapid primary emissions pair with the latest secondary value
    let (primary, secondary) = TestChannels::two();

    let combined_stream = primary.stream.with_latest_from(secondary.stream, FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act: Secondary publishes once
    push(person_alice(), &secondary.sender);

    // Act: Primary publishes rapidly multiple times (secondary stays Alice)
    push(animal_dog(), &primary.sender);
    push(animal_cat(), &primary.sender);

    // Assert: Each primary emission pairs with the same latest secondary value (Alice)
    // Note: Order is (secondary, primary) = (person, animal)
    expect_next_pair_unchecked(&mut combined_stream, person_alice(), animal_dog()).await;
    expect_next_pair_unchecked(&mut combined_stream, person_alice(), animal_cat()).await;

    // Act: Another rapid sequence of primary emissions (secondary still Alice)
    for i in 0..10 {
        primary
            .sender
            .send(animal(format!("Animal{}", i), 4))
            .unwrap();
    }

    // Assert: All primary emissions pair with Alice
    for i in 0..10 {
        let (p, a) = combined_stream.next().await.unwrap();
        assert_eq!(
            (p.value, a.value),
            (person_alice(), animal(format!("Animal{}", i), 4))
        );
    }
}

#[tokio::test]
async fn test_with_latest_from_both_streams_close_before_any_emission() {
    // Arrange: Both streams close without the secondary ever publishing
    let (primary, secondary) = TestChannels::two();

    let combined_stream = primary.stream.with_latest_from(secondary.stream, FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act: Primary publishes but secondary never does
    push(animal_dog(), &primary.sender);

    // Assert: No emission yet (waiting for secondary)
    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Act: Close both streams by dropping senders
    drop(primary.sender);
    drop(secondary.sender);

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
    let (primary, secondary) = TestChannels::two();

    let combined_stream = primary.stream.with_latest_from(secondary.stream, FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act: Send empty string person to secondary (boundary: empty string, zero age)
    push(person("".to_string(), 0), &secondary.sender);

    // Act: Send zero-leg animal to primary (boundary: empty string, zero legs)
    // This triggers first emission with both boundary values
    push(animal("".to_string(), 0), &primary.sender);

    // Assert: System handles empty/zero values correctly
    let (sec, prim) = combined_stream.next().await.unwrap();
    assert_eq!(
        sec.value,
        person("".to_string(), 0),
        "Secondary should have empty name and zero age"
    );
    assert_eq!(
        prim.value,
        animal("".to_string(), 0),
        "Primary should have empty name and zero legs"
    );

    // Act: Send normal non-boundary values
    push(person("Valid".to_string(), 1), &secondary.sender);

    // Assert: Updating secondary causes emission (with_latest_from uses combine_latest)
    let (sec2, prim2) = combined_stream.next().await.unwrap();
    assert_eq!(
        sec2.value,
        person("Valid".to_string(), 1),
        "Secondary should update to valid person"
    );
    assert_eq!(
        prim2.value,
        animal("".to_string(), 0),
        "Primary should still be boundary animal"
    );

    // Act: Now update primary to normal value
    push(animal("ValidAnimal".to_string(), 1), &primary.sender);

    // Assert: Both are now non-boundary values
    let (sec3, prim3) = combined_stream.next().await.unwrap();
    assert_eq!(
        sec3.value,
        person("Valid".to_string(), 1),
        "Secondary should remain valid person"
    );
    assert_eq!(
        prim3.value,
        animal("ValidAnimal".to_string(), 1),
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
            let (primary, secondary) = TestChannels::two();
            let combined = primary.stream.with_latest_from(secondary.stream, FILTER);
            let mut stream = Box::pin(combined);

            // Act: Send initial values
            push(person_alice(), &secondary.sender);
            push(animal_dog(), &primary.sender);

            // Assert: Verify first emission
            let (sec, prim) = stream.next().await.unwrap();
            assert_eq!(sec.value, person_alice());
            assert_eq!(prim.value, animal_dog());

            // Act: Update both values
            push(person_bob(), &secondary.sender);

            // Assert: Secondary update causes emission
            let (sec2, prim2) = stream.next().await.unwrap();
            assert_eq!(sec2.value, person_bob());
            assert_eq!(prim2.value, animal_dog()); // Primary unchanged

            // Act: Update primary
            push(animal_cat(), &primary.sender);

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
    let (primary, secondary) = TestChannels::two::<TestData>();

    // Filter that rejects when primary is Dog
    let filter = |state: &CombinedState<TestData>| {
        let values = state.get_state();
        values[1] != animal_dog() // Reject when primary is Dog
    };

    let combined_stream = primary.stream.with_latest_from(secondary.stream, filter);
    let mut stream = Box::pin(combined_stream);

    // Act: Send Dog to primary, Alice to secondary (filter should reject)
    push(animal_dog(), &primary.sender);
    push(person_alice(), &secondary.sender);

    // Assert: No emission because filter rejects
    assert_no_element_emitted(&mut stream, 100).await;

    // Act: Send Cat to primary (filter should now accept)
    push(animal_cat(), &primary.sender);

    // Assert: Now it should emit
    let (sec, prim) = stream.next().await.unwrap();
    assert_eq!(sec.get(), &person_alice());
    assert_eq!(prim.get(), &animal_cat());
}

#[tokio::test]
async fn test_with_latest_from_filter_alternates() {
    // Arrange
    let (primary, secondary) = TestChannels::two::<TestData>();

    // Counter to alternate filter behavior
    let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter_clone = counter.clone();

    // Filter alternates: true, false, true, false, ...
    let filter = move |_: &CombinedState<TestData>| {
        let count = counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        count.is_multiple_of(2) // true for even counts, false for odd
    };

    let combined_stream = primary.stream.with_latest_from(secondary.stream, filter);
    let mut stream = Box::pin(combined_stream);

    // Act: Initial values (count=0, filter=true)
    push(animal_dog(), &primary.sender);
    push(person_alice(), &secondary.sender);

    // Assert: Should emit (filter returns true)
    let (sec1, prim1) = stream.next().await.unwrap();
    assert_eq!(sec1.value, person_alice());
    assert_eq!(prim1.value, animal_dog());

    // Act: Update primary (count=1, filter=false)
    push(animal_cat(), &primary.sender);

    // Assert: Should not emit (filter returns false)
    assert_no_element_emitted(&mut stream, 100).await;

    // Act: Update secondary (count=2, filter=true)
    push(person_bob(), &secondary.sender);

    // Assert: Should emit (filter returns true)
    let (sec2, prim2) = stream.next().await.unwrap();
    assert_eq!(sec2.value, person_bob());
    assert_eq!(prim2.value, animal_cat());

    // Act: Update primary (count=3, filter=false)
    push(animal_dog(), &primary.sender);

    // Assert: Should not emit (filter returns false)
    assert_no_element_emitted(&mut stream, 100).await;
}

#[tokio::test]
#[should_panic(expected = "Filter panicked")]
async fn test_with_latest_from_filter_panics() {
    // Arrange
    let (primary, secondary) = TestChannels::two::<TestData>();

    // Filter that panics
    let filter = |_: &CombinedState<TestData>| -> bool {
        panic!("Filter panicked");
    };

    let combined_stream = primary.stream.with_latest_from(secondary.stream, filter);
    let mut stream = Box::pin(combined_stream);

    // Act: Send values to trigger filter
    push(animal_dog(), &primary.sender);
    push(person_alice(), &secondary.sender);

    // This should panic when filter is called
    let _ = stream.next().await;
}

#[tokio::test]
async fn test_with_latest_from_timestamp_ordering() {
    // Arrange
    let (primary, secondary) = TestChannels::two();

    let combined_stream = primary.stream.with_latest_from(secondary.stream, FILTER);
    let mut stream = Box::pin(combined_stream);

    // Act: Send initial values
    push(animal_dog(), &primary.sender);
    push(person_alice(), &secondary.sender);

    // Assert: First emission
    let (_sec1, prim1) = stream.next().await.unwrap();
    let ts1 = prim1.sequence();

    // Act: Update primary
    push(animal_cat(), &primary.sender);

    // Assert: Second emission should have later timestamp
    let (_sec2, prim2) = stream.next().await.unwrap();
    let ts2 = prim2.sequence();
    assert!(
        ts2 > ts1,
        "Timestamps should be monotonically increasing: {:?} should be > {:?}",
        ts2,
        ts1
    );

    // Act: Update secondary
    push(person_bob(), &secondary.sender);

    // Assert: Third emission should have later timestamp
    let (sec3, _prim3) = stream.next().await.unwrap();
    let ts3 = sec3.sequence();
    assert!(
        ts3 > ts2,
        "Timestamps should be monotonically increasing: {:?} should be > {:?}",
        ts3,
        ts2
    );
}

#[tokio::test]
async fn test_with_latest_from_multiple_secondary_values_before_primary() {
    // Arrange
    let (primary, secondary) = TestChannels::two::<TestData>();

    let combined_stream = primary.stream.with_latest_from(secondary.stream, FILTER);
    let mut stream = Box::pin(combined_stream);

    // Act: Send multiple secondary values before any primary value
    push(person_alice(), &secondary.sender);
    push(person_bob(), &secondary.sender);
    push(person_charlie(), &secondary.sender);

    // Assert: No emission yet (waiting for primary)
    assert_no_element_emitted(&mut stream, 100).await;

    // Act: Send first primary value
    push(animal_dog(), &primary.sender);

    // Assert: Should emit with latest secondary value (Charlie)
    let (sec, prim) = stream.next().await.unwrap();
    assert_eq!(sec.value, person_charlie()); // Latest secondary
    assert_eq!(prim.value, animal_dog());

    // Act: Send another secondary value
    push(person_alice(), &secondary.sender);

    // Assert: Should emit immediately (both streams have values)
    let (sec2, prim2) = stream.next().await.unwrap();
    assert_eq!(sec2.value, person_alice());
    assert_eq!(prim2.value, animal_dog()); // Primary unchanged
}
