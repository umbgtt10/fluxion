use fluxion_stream::select_all_ordered::SelectAllExt;
use fluxion_test_utils::FluxionChannel;
use fluxion_test_utils::test_data::{
    DataVariant, TestData, animal_dog, animal_spider, expect_variant, person_alice, person_bob,
    person_charlie, plant_rose, plant_sunflower, push, send_variant,
};
use fluxion_test_utils::{TestChannels, helpers::expect_next_timestamped};
use futures::StreamExt;

#[tokio::test]
async fn test_select_all_ordered_all_permutations() {
    select_all_ordered_template_test(DataVariant::Person, DataVariant::Animal, DataVariant::Plant)
        .await;
    select_all_ordered_template_test(DataVariant::Person, DataVariant::Plant, DataVariant::Animal)
        .await;
    select_all_ordered_template_test(DataVariant::Plant, DataVariant::Animal, DataVariant::Person)
        .await;
    select_all_ordered_template_test(DataVariant::Plant, DataVariant::Person, DataVariant::Animal)
        .await;
    select_all_ordered_template_test(DataVariant::Animal, DataVariant::Person, DataVariant::Plant)
        .await;
    select_all_ordered_template_test(DataVariant::Animal, DataVariant::Plant, DataVariant::Person)
        .await;
}

/// Test template for `select_all_ordered` that verifies temporal Variant preservation in stream merging.
/// Sets up channels for Person/Animal/Plant, merges streams, sends in specified Variant, and asserts correct reception.
/// Crucial for deterministic async processing; ensures `select_all_ordered` avoids race conditions of `select_all`.
/// Called with different permutations to validate robustness across send sequences.
/// Validates sequence-based Varianting over poll-time randomness for reliable event handling.
async fn select_all_ordered_template_test(
    variant1: DataVariant,
    variant2: DataVariant,
    variant3: DataVariant,
) {
    // Arrange
    let (person, animal, plant) = TestChannels::three();

    let senders = vec![person.sender, animal.sender, plant.sender];
    let streams = vec![person.stream, animal.stream, plant.stream];

    let results = streams.select_all_ordered();

    // Act
    send_variant(&variant1, &senders);
    send_variant(&variant2, &senders);
    send_variant(&variant3, &senders);

    // Assert
    let mut results = Box::pin(results);

    expect_variant(&variant1, &mut results).await;
    expect_variant(&variant2, &mut results).await;
    expect_variant(&variant3, &mut results).await;
}

#[tokio::test]
async fn test_select_all_ordered_empty_streams() {
    // Arrange
    let (person, animal, plant) = TestChannels::three::<TestData>();

    let streams = vec![person.stream, animal.stream, plant.stream];
    let results = streams.select_all_ordered();

    // Drop senders to close streams
    drop(person.sender);
    drop(animal.sender);
    drop(plant.sender);

    // Assert
    let mut results = Box::pin(results);
    let next_item = results.next().await;
    assert!(next_item.is_none(), "Expected no items from empty streams");
}

#[tokio::test]
async fn test_select_all_ordered_single_stream() {
    // Arrange
    let channel = FluxionChannel::new();

    let streams = vec![channel.stream];
    let results = streams.select_all_ordered();

    // Act
    push(person_alice(), &channel.sender);
    push(person_bob(), &channel.sender);
    push(person_charlie(), &channel.sender);

    // Assert
    let mut results = Box::pin(results);

    expect_next_timestamped(&mut results, person_alice()).await;

    let item = results.next().await.unwrap();
    assert_eq!(item.value, person_bob());

    let item = results.next().await.unwrap();
    assert_eq!(item.value, person_charlie());
}

#[tokio::test]
async fn test_select_all_ordered_one_empty_stream() {
    // Arrange
    let person = FluxionChannel::new();
    let animal = FluxionChannel::new();
    let plant = FluxionChannel::new();

    let streams = vec![person.stream, animal.stream, plant.stream];
    let results = streams.select_all_ordered();

    // Close animal stream
    drop(animal.sender);

    // Act
    push(person_alice(), &person.sender);
    push(plant_rose(), &plant.sender);
    push(person_bob(), &person.sender);

    // Assert
    let mut results = Box::pin(results);

    let item = results.next().await.unwrap();
    assert_eq!(item.value, person_alice());

    expect_next_timestamped(&mut results, plant_rose()).await;

    expect_next_timestamped(&mut results, person_bob()).await;
}

#[tokio::test]
async fn test_select_all_ordered_interleaved_emissions() {
    // Arrange
    let (person, animal, plant) = TestChannels::three();

    let streams = vec![person.stream, animal.stream, plant.stream];
    let mut results = Box::pin(streams.select_all_ordered());

    // Act & Assert - Send and verify one at a time to avoid race conditions
    push(person_alice(), &person.sender);
    expect_next_timestamped(&mut results, person_alice()).await;

    push(animal_dog(), &animal.sender);
    expect_next_timestamped(&mut results, animal_dog()).await;

    push(person_bob(), &person.sender);
    expect_next_timestamped(&mut results, person_bob()).await;

    push(plant_rose(), &plant.sender);
    expect_next_timestamped(&mut results, plant_rose()).await;

    push(animal_spider(), &animal.sender);
    expect_next_timestamped(&mut results, animal_spider()).await;

    push(plant_sunflower(), &plant.sender);
    expect_next_timestamped(&mut results, plant_sunflower()).await;
}

#[tokio::test]
async fn test_select_all_ordered_stream_completes_early() {
    // Arrange
    let (person, animal) = TestChannels::two();

    let streams = vec![person.stream, animal.stream];
    let results = streams.select_all_ordered();

    // Act
    push(person_alice(), &person.sender);
    push(animal_dog(), &animal.sender);
    drop(person.sender); // Close person stream
    push(animal_spider(), &animal.sender);

    // Assert
    let mut results = Box::pin(results);

    expect_next_timestamped(&mut results, person_alice()).await;

    expect_next_timestamped(&mut results, animal_dog()).await;

    expect_next_timestamped(&mut results, animal_spider()).await;

    // Close remaining stream
    drop(animal.sender);

    // Assert stream ends
    let next_item = results.next().await;
    assert!(next_item.is_none(), "Expected stream to end");
}

#[tokio::test]
async fn test_select_all_ordered_all_streams_close_simultaneously() {
    // Arrange
    let (person, animal, plant) = TestChannels::three();

    let streams = vec![person.stream, animal.stream, plant.stream];
    let results = streams.select_all_ordered();

    // Act
    push(person_alice(), &person.sender);
    push(animal_dog(), &animal.sender);
    push(plant_rose(), &plant.sender);

    // Close all senders
    drop(person.sender);
    drop(animal.sender);
    drop(plant.sender);

    // Assert
    let mut results = Box::pin(results);

    expect_next_timestamped(&mut results, person_alice()).await;

    expect_next_timestamped(&mut results, animal_dog()).await;

    expect_next_timestamped(&mut results, plant_rose()).await;

    // Assert stream ends after all items consumed
    let next_item = results.next().await;
    assert!(next_item.is_none(), "Expected stream to end");
}

#[tokio::test]
async fn test_select_all_ordered_one_stream_closes_midway_three_streams() {
    // Arrange
    let (person, animal, plant) = TestChannels::three();

    let streams = vec![person.stream, animal.stream, plant.stream];
    let mut results = Box::pin(streams.select_all_ordered());

    // Act & Assert stepwise to avoid race conditions
    push(person_alice(), &person.sender);
    expect_next_timestamped(&mut results, person_alice()).await;

    push(animal_dog(), &animal.sender);
    expect_next_timestamped(&mut results, animal_dog()).await;

    push(plant_rose(), &plant.sender);
    expect_next_timestamped(&mut results, plant_rose()).await;

    // Close plant stream mid-flight
    drop(plant.sender);

    // Remaining streams continue to emit
    push(person_bob(), &person.sender);
    let item = results.next().await.unwrap();
    assert_eq!(item.value, person_bob());

    push(animal_spider(), &animal.sender);
    let item = results.next().await.unwrap();
    assert_eq!(item.value, animal_spider());

    // Close remaining streams and ensure end
    drop(person.sender);
    drop(animal.sender);
    let next = results.next().await;
    assert!(next.is_none(), "Expected stream to end after all closed");
}

#[tokio::test]
async fn test_select_all_ordered_large_volume() {
    // Arrange
    let (stream1, stream2) = TestChannels::two();

    let streams = vec![stream1.stream, stream2.stream];
    let results = streams.select_all_ordered();

    // Act - Send 1000 items interleaved
    for _ in 0..500 {
        push(person_alice(), &stream1.sender);
        push(animal_dog(), &stream2.sender);
    }

    // Assert - Should receive all 1000 items in order
    let mut results = Box::pin(results);
    let mut count = 0;

    for _ in 0..500 {
        let item = results.next().await.unwrap();
        assert_eq!(item.value, person_alice());
        count += 1;

        let item = results.next().await.unwrap();
        assert_eq!(item.value, animal_dog());
        count += 1;
    }

    assert_eq!(count, 1000, "Expected 1000 items");
}

#[tokio::test]
async fn test_select_all_ordered_maximum_concurrent_streams() {
    // Arrange: Test concurrent handling with many parallel select_all_ordered operations
    let num_concurrent = 50;
    let mut handles = Vec::new();

    for _i in 0..num_concurrent {
        let handle = tokio::spawn(async move {
            let (stream1, stream2, stream3) = TestChannels::three();

            // Act: Send values to all three streams
            push(person_alice(), &stream1.sender);
            push(animal_dog(), &stream2.sender);
            push(plant_rose(), &stream3.sender);

            // Create select_all_ordered
            let streams = vec![stream1.stream, stream2.stream, stream3.stream];
            let results = streams.select_all_ordered();
            let mut results = Box::pin(results);

            // Assert: Should receive all three items in timestamp order
            let first = results.next().await.unwrap();
            assert_eq!(first.value, person_alice());

            let second = results.next().await.unwrap();
            assert_eq!(second.value, animal_dog());

            let third = results.next().await.unwrap();
            assert_eq!(third.value, plant_rose());

            // Act: Send more values
            let (stream4, stream5) = TestChannels::two();
            push(person_bob(), &stream4.sender);
            push(person_charlie(), &stream5.sender);

            let streams2 = vec![stream4.stream, stream5.stream];
            let mut results2 = Box::pin(streams2.select_all_ordered());

            // Assert: Should continue to work
            let fourth = results2.next().await.unwrap();
            assert_eq!(fourth.value, person_bob());

            let fifth = results2.next().await.unwrap();
            assert_eq!(fifth.value, person_charlie());
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
