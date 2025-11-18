// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::take_latest_when::TakeLatestWhenExt;
use fluxion_test_utils::sequenced::Sequenced;
use fluxion_test_utils::{
    helpers::assert_no_element_emitted,
    helpers::unwrap_stream,
    test_channel,
    test_data::{
        animal, animal_ant, animal_cat, animal_dog, person, person_alice, person_bob,
        person_charlie, person_dave, TestData,
    },
};
use futures::StreamExt;

#[tokio::test]
async fn test_take_latest_when_empty_streams() -> anyhow::Result<()> {
    let filter_fn = |_: &TestData| -> bool { true };

    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();
    drop(source_tx);
    drop(filter_tx);

    let mut output_stream = source_stream.take_latest_when(filter_stream, filter_fn);

    // Act & Assert
    let next_item = output_stream.next().await;
    assert!(
        next_item.is_none(),
        "Expected no items from an empty stream with `take_latest_when`"
    );
    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_filter_not_satisfied_does_not_emit() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let filter_fn = |filter_val: &TestData| -> bool {
        match filter_val {
            TestData::Animal(animal) => animal.legs > 5,
            _ => false,
        }
    };

    let mut output_stream = source_stream.take_latest_when(filter_stream, filter_fn);

    // Act
    source_tx.send(Sequenced::new(person_alice()))?;
    filter_tx.send(Sequenced::new(animal_dog()))?;

    // Assert
    assert_no_element_emitted(&mut output_stream, 100).await;
    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_filter_satisfied_emits() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let filter_fn = |filter_val: &TestData| -> bool {
        match filter_val {
            TestData::Animal(animal) => animal.legs > 5,
            _ => {
                panic!(
                    "Expected the filter stream to emit an Animal value. But it emitted: {filter_val:?} instead!"
                );
            }
        }
    };

    let mut output_stream = source_stream.take_latest_when(filter_stream, filter_fn);

    // Act
    source_tx.send(Sequenced::new(person_alice()))?;
    filter_tx.send(Sequenced::new(animal_ant()))?;

    // Assert
    let emitted_item = unwrap_stream(&mut output_stream, 500).await.unwrap();
    assert_eq!(
        emitted_item.get(),
        &person_alice(),
        "Expected the source item to be emitted when the filter is satisfied"
    );
    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_multiple_emissions_filter_satisfied() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let filter_fn = |filter_val: &TestData| -> bool {
        match filter_val {
            TestData::Animal(animal) => animal.legs > 5,
            _ => panic!(
                "Expected the filter stream to emit an Animal value. But it emitted: {filter_val:?} instead!",
            ),
        }
    };

    let mut output_stream = source_stream.take_latest_when(filter_stream, filter_fn);

    // Act
    source_tx.send(Sequenced::new(person_alice()))?;
    filter_tx.send(Sequenced::new(animal_ant()))?;

    // Assert

    let first_item = unwrap_stream(&mut output_stream, 500).await.unwrap();
    assert_eq!(
        first_item.get(),
        &person_alice(),
        "First emitted item did not match expected"
    );

    // Act
    source_tx.send(Sequenced::new(person_bob()))?;
    filter_tx.send(Sequenced::new(animal_ant()))?; // Trigger filter again to sample Bob

    // Assert
    let second_item = unwrap_stream(&mut output_stream, 500).await.unwrap();
    assert_eq!(
        second_item.get(),
        &person_bob(),
        "Second emitted item did not match expected"
    );
    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_multiple_emissions_filter_not_satisfied() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let filter_fn = |filter_val: &TestData| -> bool {
        match filter_val {
            TestData::Animal(animal) => animal.legs > 5,
            _ => panic!(
                "Expected the filter stream to emit an Animal value. But it emitted: {filter_val:?} instead!",
            ),
        }
    };

    let mut output_stream = source_stream.take_latest_when(filter_stream, filter_fn);

    // Act
    source_tx.send(Sequenced::new(person_charlie()))?;
    filter_tx.send(Sequenced::new(animal_ant()))?; // Trigger filter to sample Charlie

    // Assert

    let first_item = unwrap_stream(&mut output_stream, 500).await.unwrap();
    assert_eq!(
        first_item.get(),
        &person_charlie(),
        "First emitted item did not match expected"
    );

    // Act
    filter_tx.send(Sequenced::new(animal_cat()))?; // legs 4 -> predicate false
    source_tx.send(Sequenced::new(person_dave()))?;

    // Assert: No emission since filter predicate is false (cat has 4 legs)
    assert_no_element_emitted(&mut output_stream, 100).await;
    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_filter_toggle_emissions() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let filter_fn = |filter_val: &TestData| -> bool {
        match filter_val {
            TestData::Animal(animal) => animal.legs > 5,
            _ => panic!(
                "Expected the filter stream to emit an Animal value. But it emitted: {filter_val:?} instead!",
            ),
        }
    };

    let mut output_stream = source_stream.take_latest_when(filter_stream, filter_fn);

    // Act: source first, then filter triggers -> emit
    source_tx.send(Sequenced::new(person_alice()))?;
    filter_tx.send(Sequenced::new(animal_ant()))?; // legs 6 -> true, triggers emission
    let first = unwrap_stream(&mut output_stream, 500).await.unwrap();
    assert_eq!(
        first.get(),
        &person_alice(),
        "Should emit Alice when filter triggers with true predicate"
    );

    // Act: filter false, then source -> no emit
    filter_tx.send(Sequenced::new(animal_cat()))?; // legs 4 -> false
    source_tx.send(Sequenced::new(person_bob()))?;
    fluxion_test_utils::helpers::assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: filter true again -> should emit the latest buffered source (Bob)
    filter_tx.send(Sequenced::new(animal_ant()))?; // true
    let third = unwrap_stream(&mut output_stream, 500).await.unwrap();
    assert_eq!(
        third.get(),
        &person_bob(),
        "Should emit buffered Bob when filter becomes true"
    );

    // Act: source emits another value (Charlie)
    source_tx.send(Sequenced::new(person_charlie()))?;

    // Act: filter triggers again -> should sample Charlie
    filter_tx.send(Sequenced::new(animal_ant()))?; // true
    let fourth = unwrap_stream(&mut output_stream, 500).await.unwrap();
    assert_eq!(
        fourth.get(),
        &person_charlie(),
        "Should emit Charlie when filter triggers again"
    );
    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_filter_stream_closes_no_further_emits() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let filter_fn = |filter_val: &TestData| -> bool {
        match filter_val {
            TestData::Animal(animal) => animal.legs > 5,
            _ => false,
        }
    };

    let mut output_stream = source_stream.take_latest_when(filter_stream, filter_fn);

    // Prime both streams so a first emission can happen
    source_tx.send(Sequenced::new(person_alice()))?;
    filter_tx.send(Sequenced::new(animal_ant()))?; // true - triggers emission of Alice
    let first = unwrap_stream(&mut output_stream, 500).await.unwrap();
    assert_eq!(
        first.get(),
        &person_alice(),
        "First emission should be Alice with filter true"
    );

    // Close the filter stream
    drop(filter_tx);

    // After filter stream closes, source updates should NOT trigger emissions
    // because take_latest_when only emits when the filter stream updates
    source_tx.send(Sequenced::new(person_bob()))?;

    // Assert: No emission because filter stream is closed
    fluxion_test_utils::helpers::assert_no_element_emitted(&mut output_stream, 100).await;

    source_tx.send(Sequenced::new(person_charlie()))?;

    // Assert: Still no emission
    fluxion_test_utils::helpers::assert_no_element_emitted(&mut output_stream, 100).await;
    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_source_publishes_before_filter() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let filter_fn = |filter_val: &TestData| -> bool {
        match filter_val {
            TestData::Animal(animal) => animal.legs > 5,
            _ => false,
        }
    };

    let mut output_stream = source_stream.take_latest_when(filter_stream, filter_fn);

    // Act: Source publishes first (before filter has any value)
    source_tx.send(Sequenced::new(person_alice()))?;

    // Assert: No emission yet (waiting for filter)
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Filter publishes with true condition
    filter_tx.send(Sequenced::new(animal_ant()))?; // legs 6 -> true

    // Assert: Now we get the buffered source value
    let first = unwrap_stream(&mut output_stream, 500).await.unwrap();
    assert_eq!(
        first.get(),
        &person_alice(),
        "Should emit buffered Alice when filter becomes true"
    );

    // Act: Filter changes to false first, THEN source updates
    filter_tx.send(Sequenced::new(animal_cat()))?; // legs 4 -> false
    source_tx.send(Sequenced::new(person_bob()))?;

    // Assert: No emission (filter is false when Bob arrives and after)
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Filter becomes true again
    filter_tx.send(Sequenced::new(animal_ant()))?; // legs 6 -> true

    // Assert: Emits the buffered Bob
    let second = unwrap_stream(&mut output_stream, 500).await.unwrap();
    assert_eq!(
        second.get(),
        &person_bob(),
        "Should emit buffered Bob when filter becomes true again"
    );
    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_multiple_source_updates_while_filter_false() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let filter_fn = |filter_val: &TestData| -> bool {
        match filter_val {
            TestData::Animal(animal) => animal.legs > 5,
            _ => false,
        }
    };

    let mut output_stream = source_stream.take_latest_when(filter_stream, filter_fn);

    // Act: Start with filter false
    filter_tx.send(Sequenced::new(animal_cat()))?; // legs 4 -> false

    // Act: Source publishes multiple times while filter remains false
    source_tx.send(Sequenced::new(person_alice()))?;
    source_tx.send(Sequenced::new(person_bob()))?;
    source_tx.send(Sequenced::new(person_charlie()))?;
    source_tx.send(Sequenced::new(person_dave()))?;

    // Assert: No emissions yet
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Filter becomes true
    filter_tx.send(Sequenced::new(animal_ant()))?; // legs 6 -> true

    // Assert: Only the LATEST source value (Dave) is emitted, not all previous ones
    let first = unwrap_stream(&mut output_stream, 500).await.unwrap();
    assert_eq!(
        first.get(),
        &person_dave(),
        "Should only emit latest value (Dave), not earlier buffered values"
    );

    // Assert: No additional emissions (Alice, Bob, Charlie were discarded)
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Source publishes again with filter still true
    source_tx.send(Sequenced::new(person_alice()))?;
    filter_tx.send(Sequenced::new(animal_ant()))?; // Trigger filter to sample Alice

    // Assert: Emits when filter triggers
    let second = unwrap_stream(&mut output_stream, 500).await.unwrap();
    assert_eq!(
        second.get(),
        &person_alice(),
        "Should emit Alice immediately when filter is true"
    );
    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_buffer_does_not_grow_unbounded() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let filter_fn = |filter_val: &TestData| -> bool {
        match filter_val {
            TestData::Animal(animal) => animal.legs > 5,
            _ => false,
        }
    };

    let mut output_stream = source_stream.take_latest_when(filter_stream, filter_fn);

    // Act: Set filter to false
    filter_tx.send(Sequenced::new(animal_cat()))?; // legs 4 -> false

    // Act: Publish a large number of source events while filter is false
    for i in 0u32..10000u32 {
        source_tx
            .send(Sequenced::new(person(format!("Person{i}"), i)))
            .unwrap();
    }

    // Assert: No emissions yet
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Filter becomes true
    filter_tx.send(Sequenced::new(animal_ant()))?; // legs 6 -> true

    // Assert: Only the LATEST value is emitted (Person9999)
    let first = unwrap_stream(&mut output_stream, 500).await.unwrap();
    assert_eq!(first.get(), &person(String::from("Person9999"), 9999u32));

    // Assert: No additional emissions (buffer only held the latest, not all 10000)
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Toggle filter false and publish more
    filter_tx.send(Sequenced::new(animal_dog()))?; // legs 4 -> false
    for i in 10000u32..20000u32 {
        source_tx
            .send(Sequenced::new(person(format!("Person{i}"), i)))
            .unwrap();
    }

    // Assert: Still no emissions
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Filter true again
    filter_tx.send(Sequenced::new(animal_ant()))?; // legs 6 -> true

    // Assert: Only the latest from the second batch (Person19999)
    let second = unwrap_stream(&mut output_stream, 500).await.unwrap();
    assert_eq!(second.get(), &person(String::from("Person19999"), 19999u32));

    // This test validates that the buffer doesn't grow unbounded - it only keeps
    // the latest source value, not all historical values while the filter is false
    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_boundary_empty_string_zero_values() -> anyhow::Result<()> {
    let filter_fn: fn(&TestData) -> bool = |_: &TestData| true;

    // Arrange: Test boundary values (empty strings, zero numeric values)
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut output_stream = source_stream.take_latest_when(filter_stream, filter_fn);

    // Act: Send empty string with zero value to source
    source_tx
        .send(Sequenced::new(person(String::new(), 0)))
        .unwrap();

    // Act: Send filter trigger with empty/zero
    filter_tx
        .send(Sequenced::new(animal(String::new(), 0)))
        .unwrap();

    // Assert: Should emit the boundary value
    let result = unwrap_stream(&mut output_stream, 500).await.unwrap();
    assert_eq!(
        result.get(),
        &person(String::new(), 0),
        "Should handle empty string and zero age"
    );

    // Act: Update to normal values
    source_tx.send(Sequenced::new(person_alice()))?;
    filter_tx.send(Sequenced::new(animal_dog()))?;

    // Assert: Should emit normal value
    let result2 = unwrap_stream(&mut output_stream, 500).await.unwrap();
    assert_eq!(result2.get(), &person_alice());

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_boundary_maximum_concurrent_streams() -> anyhow::Result<()> {
    let filter_fn: fn(&TestData) -> bool = |_: &TestData| true;

    // Arrange: Test concurrent handling with many parallel streams
    let num_concurrent: u32 = 50;
    let mut handles = Vec::new();

    for i in 0..num_concurrent {
        let handle = tokio::spawn(async move {
            let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
            let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();
            let mut output_stream = source_stream.take_latest_when(filter_stream, filter_fn);

            // Act: Send test values
            source_tx
                .send(Sequenced::new(person(format!("Person{i}"), i)))
                .unwrap();
            filter_tx
                .send(Sequenced::new(animal(format!("Animal{i}"), i)))
                .unwrap();

            // Assert: Should emit
            let result = unwrap_stream(&mut output_stream, 500).await.unwrap();
            assert_eq!(result.get(), &person(format!("Person{i}"), i));

            // Act: Update source and trigger again
            source_tx.send(Sequenced::new(person_bob())).unwrap();
            filter_tx.send(Sequenced::new(animal_cat())).unwrap();

            // Assert: Should emit updated value
            let result2 = unwrap_stream(&mut output_stream, 500).await.unwrap();
            assert_eq!(result2.get(), &person_bob());
        });

        handles.push(handle);
    }

    // Wait for all concurrent streams to complete
    for handle in handles {
        handle
            .await
            .expect("Concurrent stream task should complete successfully");
    }

    Ok(())
}

#[tokio::test]
#[should_panic(expected = "Filter panicked")]
async fn test_take_latest_when_filter_panics() {
    // Arrange:
    let filter_fn = |_: &TestData| -> bool {
        panic!("Filter panicked");
    };

    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();
    let mut output_stream = source_stream.take_latest_when(filter_stream, filter_fn);

    // Act
    source_tx.send(Sequenced::new(person_alice())).unwrap();
    filter_tx.send(Sequenced::new(animal_dog())).unwrap();

    // Assert
    let _ = output_stream.next().await;
}
