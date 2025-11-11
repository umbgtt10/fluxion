use fluxion_stream::{combine_latest::CombinedState, take_latest_when::TakeLatestWhenExt};
use fluxion_test_utils::{
    TestChannels,
    helpers::assert_no_element_emitted,
    push,
    test_data::{
        TestData, animal, animal_ant, animal_cat, animal_dog, person, person_alice, person_bob,
        person_charlie, person_dave,
    },
};
use futures::StreamExt;

#[tokio::test]
async fn test_take_latest_when_empty_streams() {
    static FILTER: fn(&CombinedState<TestData>) -> bool = |_: &CombinedState<TestData>| true;

    // Arrange
    let (source, filter) = TestChannels::two();
    drop(source.sender);
    drop(filter.sender);

    let output_stream = source.stream.take_latest_when(filter.stream, FILTER);
    let mut output_stream = Box::pin(output_stream);

    // Act & Assert
    let next_item = output_stream.next().await;
    assert!(
        next_item.is_none(),
        "Expected no items from an empty stream with `take_latest_when`"
    );
}

#[tokio::test]
async fn test_take_latest_when_filter_not_satisfied_does_not_emit() {
    // Arrange
    let (source, filter) = TestChannels::two();

    static FILTER: fn(&CombinedState<TestData>) -> bool = |state| {
        let state = state.get_state().first().unwrap().clone();
        match state {
            TestData::Animal(animal) => animal.legs > 5,
            _ => false,
        }
    };

    let output_stream = source.stream.take_latest_when(filter.stream, FILTER);
    let mut output_stream = Box::pin(output_stream);

    // Act
    push(person_alice(), &source.sender);
    push(animal_dog(), &filter.sender);

    // Assert
    assert_no_element_emitted(&mut output_stream, 100).await;
}

#[tokio::test]
async fn test_take_latest_when_filter_satisfied_emits() {
    // Arrange
    let (source, filter) = TestChannels::two();

    static FILTER: fn(&CombinedState<TestData>) -> bool = |state| {
        let filter_value = state.get_state()[1].clone();

        match filter_value {
            TestData::Animal(animal) => animal.legs > 5,
            _ => {
                panic!(
                    "Expected the filter stream to emit an Animal value. But it emitted: {:?} instead!",
                    filter_value
                );
            }
        }
    };

    let output_stream = source.stream.take_latest_when(filter.stream, FILTER);

    // Act
    push(person_alice(), &source.sender);
    push(animal_ant(), &filter.sender);

    // Assert
    let mut output_stream = Box::pin(output_stream);
    let emitted_item = output_stream.next().await.unwrap();
    assert_eq!(
        emitted_item,
        person_alice(),
        "Expected the source item to be emitted when the filter is satisfied"
    );
}

#[tokio::test]
async fn test_take_latest_when_multiple_emissions_filter_satisfied() {
    // Arrange
    let (source, filter) = TestChannels::two();

    static FILTER: fn(&CombinedState<TestData>) -> bool = |state| {
        let filter_value = state.get_state()[1].clone();

        match filter_value {
            TestData::Animal(animal) => animal.legs > 5,
            _ => {
                panic!(
                    "Expected the filter stream to emit an Animal value. But it emitted: {:?} instead!",
                    filter_value
                );
            }
        }
    };

    let output_stream = source.stream.take_latest_when(filter.stream, FILTER);

    // Act
    push(person_alice(), &source.sender);
    push(animal_ant(), &filter.sender);

    // Assert
    let mut output_stream = Box::pin(output_stream);

    let first_item = output_stream.next().await.unwrap();
    assert_eq!(
        first_item,
        person_alice(),
        "First emitted item did not match expected"
    );

    // Act
    push(person_bob(), &source.sender);

    // Assert
    let second_item = output_stream.next().await.unwrap();
    assert_eq!(
        second_item,
        person_bob(),
        "Second emitted item did not match expected"
    );
}

#[tokio::test]
async fn test_take_latest_when_multiple_emissions_filter_not_satisfied() {
    // Arrange
    let (source, filter) = TestChannels::two();

    static FILTER: fn(&CombinedState<TestData>) -> bool = |state| {
        let filter_value = state.get_state()[1].clone();

        match filter_value {
            TestData::Animal(animal) => animal.legs > 5,
            _ => {
                panic!(
                    "Expected the filter stream to emit an Animal value. But it emitted: {:?} instead!",
                    filter_value
                );
            }
        }
    };

    let output_stream = source.stream.take_latest_when(filter.stream, FILTER);

    // Act
    push(animal_ant(), &filter.sender);
    push(person_charlie(), &source.sender);

    // Assert
    let mut output_stream = Box::pin(output_stream);

    let first_item = output_stream.next().await.unwrap();
    assert_eq!(
        first_item,
        person_charlie(),
        "First emitted item did not match expected"
    );

    // Act
    push(animal_cat(), &filter.sender);
    push(person_dave(), &source.sender);

    // Assert
    assert_no_element_emitted(&mut output_stream, 100).await;
}

#[tokio::test]
async fn test_take_latest_when_filter_toggle_emissions() {
    // Arrange
    let (source, filter) = TestChannels::two();

    static FILTER: fn(&CombinedState<TestData>) -> bool = |state| {
        let filter_value = state.get_state()[1].clone();

        match filter_value {
            TestData::Animal(animal) => animal.legs > 5,
            _ => panic!(
                "Expected the filter stream to emit an Animal value. But it emitted: {:?} instead!",
                filter_value
            ),
        }
    };

    let output_stream = source.stream.take_latest_when(filter.stream, FILTER);
    let mut output_stream = Box::pin(output_stream);

    // Act: filter true, then source -> emit
    push(animal_ant(), &filter.sender); // legs 6 -> true
    push(person_alice(), &source.sender);
    let first = output_stream.next().await.unwrap();
    assert_eq!(
        first,
        person_alice(),
        "Should emit Alice when filter is true"
    );

    // Act: filter false, then source -> no emit
    push(animal_cat(), &filter.sender); // legs 4 -> false
    push(person_bob(), &source.sender);
    fluxion_test_utils::helpers::assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: filter true again -> should emit the latest buffered source (Bob)
    push(animal_ant(), &filter.sender); // true
    let third = output_stream.next().await.unwrap();
    assert_eq!(
        third,
        person_bob(),
        "Should emit buffered Bob when filter becomes true"
    );

    // Act: then source emits another -> emit immediately with true filter
    push(person_charlie(), &source.sender);
    let fourth = output_stream.next().await.unwrap();
    assert_eq!(
        fourth,
        person_charlie(),
        "Should emit Charlie immediately with filter still true"
    );
}

#[tokio::test]
async fn test_take_latest_when_filter_stream_closes_no_further_emits() {
    // Arrange
    let (source, filter) = TestChannels::two();

    static FILTER: fn(&CombinedState<TestData>) -> bool = |state| {
        let filter_value = state.get_state()[1].clone();
        match filter_value {
            TestData::Animal(animal) => animal.legs > 5,
            _ => false,
        }
    };

    let output_stream = source.stream.take_latest_when(filter.stream, FILTER);
    let mut output_stream = Box::pin(output_stream);

    // Prime both streams so a first emission can happen
    push(animal_ant(), &filter.sender); // true
    push(person_alice(), &source.sender);
    let first = output_stream.next().await.unwrap();
    assert_eq!(
        first,
        person_alice(),
        "First emission should be Alice with filter true"
    );

    // Close the filter stream
    drop(filter.sender);

    // With the last filter value still true, further source events should continue to emit
    push(person_bob(), &source.sender);
    let second = output_stream.next().await.unwrap();
    assert_eq!(
        second,
        person_bob(),
        "Should emit Bob with persisted filter value after filter stream closes"
    );

    push(person_charlie(), &source.sender);
    let third = output_stream.next().await.unwrap();
    assert_eq!(
        third,
        person_charlie(),
        "Should emit Charlie with persisted filter value"
    );
}

#[tokio::test]
async fn test_take_latest_when_source_publishes_before_filter() {
    // Arrange
    let (source, filter) = TestChannels::two();

    static FILTER: fn(&CombinedState<TestData>) -> bool = |state| {
        let filter_value = state.get_state()[1].clone();
        match filter_value {
            TestData::Animal(animal) => animal.legs > 5,
            _ => false,
        }
    };

    let output_stream = source.stream.take_latest_when(filter.stream, FILTER);
    let mut output_stream = Box::pin(output_stream);

    // Act: Source publishes first (before filter has any value)
    push(person_alice(), &source.sender);

    // Assert: No emission yet (waiting for filter)
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Filter publishes with true condition
    push(animal_ant(), &filter.sender); // legs 6 -> true

    // Assert: Now we get the buffered source value
    let first = output_stream.next().await.unwrap();
    assert_eq!(
        first,
        person_alice(),
        "Should emit buffered Alice when filter becomes true"
    );

    // Act: Filter changes to false first, THEN source updates
    push(animal_cat(), &filter.sender); // legs 4 -> false
    push(person_bob(), &source.sender);

    // Assert: No emission (filter is false when Bob arrives and after)
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Filter becomes true again
    push(animal_ant(), &filter.sender); // legs 6 -> true

    // Assert: Emits the buffered Bob
    let second = output_stream.next().await.unwrap();
    assert_eq!(
        second,
        person_bob(),
        "Should emit buffered Bob when filter becomes true again"
    );
}

#[tokio::test]
async fn test_take_latest_when_multiple_source_updates_while_filter_false() {
    // Arrange
    let (source, filter) = TestChannels::two();

    static FILTER: fn(&CombinedState<TestData>) -> bool = |state| {
        let filter_value = state.get_state()[1].clone();
        match filter_value {
            TestData::Animal(animal) => animal.legs > 5,
            _ => false,
        }
    };

    let output_stream = source.stream.take_latest_when(filter.stream, FILTER);
    let mut output_stream = Box::pin(output_stream);

    // Act: Start with filter false
    push(animal_cat(), &filter.sender); // legs 4 -> false

    // Act: Source publishes multiple times while filter remains false
    push(person_alice(), &source.sender);
    push(person_bob(), &source.sender);
    push(person_charlie(), &source.sender);
    push(person_dave(), &source.sender);

    // Assert: No emissions yet
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Filter becomes true
    push(animal_ant(), &filter.sender); // legs 6 -> true

    // Assert: Only the LATEST source value (Dave) is emitted, not all previous ones
    let first = output_stream.next().await.unwrap();
    assert_eq!(
        first,
        person_dave(),
        "Should only emit latest value (Dave), not earlier buffered values"
    );

    // Assert: No additional emissions (Alice, Bob, Charlie were discarded)
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Source publishes again with filter still true
    push(person_alice(), &source.sender);

    // Assert: Emits immediately
    let second = output_stream.next().await.unwrap();
    assert_eq!(
        second,
        person_alice(),
        "Should emit Alice immediately when filter is true"
    );
}

#[tokio::test]
async fn test_take_latest_when_buffer_does_not_grow_unbounded() {
    // Arrange
    let (source, filter) = TestChannels::two();

    static FILTER: fn(&CombinedState<TestData>) -> bool = |state| {
        let filter_value = state.get_state()[1].clone();
        match filter_value {
            TestData::Animal(animal) => animal.legs > 5,
            _ => false,
        }
    };

    let output_stream = source.stream.take_latest_when(filter.stream, FILTER);
    let mut output_stream = Box::pin(output_stream);

    // Act: Set filter to false
    push(animal_cat(), &filter.sender); // legs 4 -> false

    // Act: Publish a large number of source events while filter is false
    for i in 0..10000 {
        source
            .sender
            .send(person(format!("Person{}", i), i as u32))
            .unwrap();
    }

    // Assert: No emissions yet
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Filter becomes true
    push(animal_ant(), &filter.sender); // legs 6 -> true

    // Assert: Only the LATEST value is emitted (Person9999)
    let first = output_stream.next().await.unwrap();
    assert_eq!(first, person("Person9999".to_string(), 9999));

    // Assert: No additional emissions (buffer only held the latest, not all 10000)
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Toggle filter false and publish more
    push(animal_dog(), &filter.sender); // legs 4 -> false
    for i in 10000..20000 {
        source
            .sender
            .send(person(format!("Person{}", i), i as u32))
            .unwrap();
    }

    // Assert: Still no emissions
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Filter true again
    push(animal_ant(), &filter.sender); // legs 6 -> true

    // Assert: Only the latest from the second batch (Person19999)
    let second = output_stream.next().await.unwrap();
    assert_eq!(second, person("Person19999".to_string(), 19999));

    // This test validates that the buffer doesn't grow unbounded - it only keeps
    // the latest source value, not all historical values while the filter is false
}

#[tokio::test]
async fn test_take_latest_when_boundary_empty_string_zero_values() {
    static FILTER: fn(&CombinedState<TestData>) -> bool = |_: &CombinedState<TestData>| true;

    // Arrange: Test boundary values (empty strings, zero numeric values)
    let (source, filter) = TestChannels::two();

    let output_stream = source.stream.take_latest_when(filter.stream, FILTER);
    let mut output_stream = Box::pin(output_stream);

    // Act: Send empty string with zero value to source
    push(person("".to_string(), 0), &source.sender);

    // Act: Send filter trigger with empty/zero
    push(animal("".to_string(), 0), &filter.sender);

    // Assert: Should emit the boundary value
    let result = output_stream.next().await.unwrap();
    assert_eq!(
        result,
        person("".to_string(), 0),
        "Should handle empty string and zero age"
    );

    // Act: Update to normal values
    push(person_alice(), &source.sender);
    push(animal_dog(), &filter.sender);

    // Assert: Should emit normal value
    let result2 = output_stream.next().await.unwrap();
    assert_eq!(result2, person_alice());
}

#[tokio::test]
async fn test_take_latest_when_boundary_maximum_concurrent_streams() {
    static FILTER: fn(&CombinedState<TestData>) -> bool = |_: &CombinedState<TestData>| true;

    // Arrange: Test concurrent handling with many parallel streams
    let num_concurrent = 50;
    let mut handles = Vec::new();

    for i in 0..num_concurrent {
        let handle = tokio::spawn(async move {
            let (source, filter) = TestChannels::two();
            let output_stream = source.stream.take_latest_when(filter.stream, FILTER);
            let mut output_stream = Box::pin(output_stream);

            // Act: Send values
            push(person(format!("Person{}", i), i), &source.sender);
            push(animal(format!("Animal{}", i), i), &filter.sender);

            // Assert: Should emit
            let result = output_stream.next().await.unwrap();
            assert_eq!(result, person(format!("Person{}", i), i));

            // Act: Update source and trigger again
            push(person_bob(), &source.sender);
            push(animal_cat(), &filter.sender);

            // Assert: Should emit updated value
            let result2 = output_stream.next().await.unwrap();
            assert_eq!(result2, person_bob());
        });

        handles.push(handle);
    }

    // Wait for all concurrent streams to complete
    for handle in handles {
        handle
            .await
            .expect("Concurrent stream task should complete successfully");
    }
}

#[tokio::test]
#[should_panic(expected = "Filter panicked")]
async fn test_take_latest_when_filter_panics() {
    // Arrange:
    let filter_fn = |_: &CombinedState<TestData>| -> bool {
        panic!("Filter panicked");
    };

    let (source, filter) = TestChannels::two();
    let output_stream = source.stream.take_latest_when(filter.stream, filter_fn);

    // Act
    push(person_alice(), &source.sender);
    push(animal_dog(), &filter.sender);

    // Assert
    let mut output_stream = Box::pin(output_stream);
    let _ = output_stream.next().await;
}
