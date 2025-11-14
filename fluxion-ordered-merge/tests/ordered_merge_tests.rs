// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_ordered_merge::OrderedMergeExt;
use fluxion_test_utils::FluxionChannel;
use fluxion_test_utils::test_data::{
    DataVariant, TestData, animal_dog, animal_spider, expect_variant, person_alice, person_bob,
    person_charlie, plant_rose, plant_sunflower, push, send_variant,
};
use fluxion_test_utils::{TestChannels, helpers::expect_next_timestamped_unchecked};
use futures::StreamExt;

#[tokio::test]
async fn test_ordered_merge_all_permutations() {
    ordered_merge_template_test(DataVariant::Person, DataVariant::Animal, DataVariant::Plant).await;
    ordered_merge_template_test(DataVariant::Person, DataVariant::Plant, DataVariant::Animal).await;
    ordered_merge_template_test(DataVariant::Plant, DataVariant::Animal, DataVariant::Person).await;
    ordered_merge_template_test(DataVariant::Plant, DataVariant::Person, DataVariant::Animal).await;
    ordered_merge_template_test(DataVariant::Animal, DataVariant::Person, DataVariant::Plant).await;
    ordered_merge_template_test(DataVariant::Animal, DataVariant::Plant, DataVariant::Person).await;
}

async fn ordered_merge_template_test(
    variant1: DataVariant,
    variant2: DataVariant,
    variant3: DataVariant,
) {
    // Arrange
    let (person, animal, plant) = TestChannels::three();

    let senders = vec![person.sender, animal.sender, plant.sender];
    let streams_list = vec![person.stream, animal.stream, plant.stream];

    let results = streams_list.ordered_merge();

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
async fn test_ordered_merge_empty_streams() {
    // Arrange
    let (person, animal, plant) = TestChannels::three::<TestData>();

    let streams_list = vec![person.stream, animal.stream, plant.stream];
    let results = streams_list.ordered_merge();

    // Act
    drop(person.sender);
    drop(animal.sender);
    drop(plant.sender);

    // Assert
    let mut results = Box::pin(results);
    let next_item = results.next().await;
    assert!(next_item.is_none(), "Expected no items from empty streams");
}

#[tokio::test]
async fn test_ordered_merge_single_stream() {
    // Arrange
    let channel = FluxionChannel::new();

    let streams_list = vec![channel.stream];
    let results = streams_list.ordered_merge();

    // Act
    push(person_alice(), &channel.sender);
    push(person_bob(), &channel.sender);
    push(person_charlie(), &channel.sender);

    // Assert
    let mut results = Box::pin(results);

    expect_next_timestamped_unchecked(&mut results, person_alice()).await;

    let item = results.next().await.unwrap();
    assert_eq!(item.value, person_bob());

    let item = results.next().await.unwrap();
    assert_eq!(item.value, person_charlie());
}

#[tokio::test]
async fn test_ordered_merge_one_empty_stream() {
    // Arrange
    let person = FluxionChannel::new();
    let animal = FluxionChannel::new();
    let plant = FluxionChannel::new();

    let streams_list = vec![person.stream, animal.stream, plant.stream];
    let results = streams_list.ordered_merge();

    // Act
    drop(animal.sender);

    push(person_alice(), &person.sender);
    push(plant_rose(), &plant.sender);
    push(person_bob(), &person.sender);

    // Assert
    let mut results = Box::pin(results);

    let item = results.next().await.unwrap();
    assert_eq!(item.value, person_alice());

    expect_next_timestamped_unchecked(&mut results, plant_rose()).await;

    expect_next_timestamped_unchecked(&mut results, person_bob()).await;
}

#[tokio::test]
async fn test_ordered_merge_interleaved_emissions() {
    // Arrange
    let (person, animal, plant) = TestChannels::three();

    let streams_list = vec![person.stream, animal.stream, plant.stream];
    let mut results = Box::pin(streams_list.ordered_merge());

    // Act & Assert
    push(person_alice(), &person.sender);
    expect_next_timestamped_unchecked(&mut results, person_alice()).await;

    push(animal_dog(), &animal.sender);
    expect_next_timestamped_unchecked(&mut results, animal_dog()).await;

    push(person_bob(), &person.sender);
    expect_next_timestamped_unchecked(&mut results, person_bob()).await;

    push(plant_rose(), &plant.sender);
    expect_next_timestamped_unchecked(&mut results, plant_rose()).await;

    push(animal_spider(), &animal.sender);
    expect_next_timestamped_unchecked(&mut results, animal_spider()).await;

    push(plant_sunflower(), &plant.sender);
    expect_next_timestamped_unchecked(&mut results, plant_sunflower()).await;
}

#[tokio::test]
async fn test_ordered_merge_stream_completes_early() {
    // Arrange
    let (person, animal) = TestChannels::two();

    let streams_list = vec![person.stream, animal.stream];
    let results = streams_list.ordered_merge();

    // Act
    push(person_alice(), &person.sender);
    push(animal_dog(), &animal.sender);
    drop(person.sender);
    push(animal_spider(), &animal.sender);

    // Assert
    let mut results = Box::pin(results);

    expect_next_timestamped_unchecked(&mut results, person_alice()).await;

    expect_next_timestamped_unchecked(&mut results, animal_dog()).await;

    expect_next_timestamped_unchecked(&mut results, animal_spider()).await;

    drop(animal.sender);

    let next_item = results.next().await;
    assert!(next_item.is_none(), "Expected stream to end");
}

#[tokio::test]
async fn test_ordered_merge_all_streams_close_simultaneously() {
    // Arrange
    let (person, animal, plant) = TestChannels::three();

    let streams_list = vec![person.stream, animal.stream, plant.stream];
    let results = streams_list.ordered_merge();

    // Act
    push(person_alice(), &person.sender);
    push(animal_dog(), &animal.sender);
    push(plant_rose(), &plant.sender);

    drop(person.sender);
    drop(animal.sender);
    drop(plant.sender);

    // Assert
    let mut results = Box::pin(results);

    expect_next_timestamped_unchecked(&mut results, person_alice()).await;

    expect_next_timestamped_unchecked(&mut results, animal_dog()).await;

    expect_next_timestamped_unchecked(&mut results, plant_rose()).await;

    let next_item = results.next().await;
    assert!(next_item.is_none(), "Expected stream to end");
}

#[tokio::test]
async fn test_ordered_merge_one_stream_closes_midway_three_streams() {
    // Arrange
    let (person, animal, plant) = TestChannels::three();

    let streams = vec![person.stream, animal.stream, plant.stream];
    let mut results = Box::pin(streams.ordered_merge());

    // Act & Assert stepwise
    push(person_alice(), &person.sender);
    expect_next_timestamped_unchecked(&mut results, person_alice()).await;

    push(animal_dog(), &animal.sender);
    expect_next_timestamped_unchecked(&mut results, animal_dog()).await;

    push(plant_rose(), &plant.sender);
    expect_next_timestamped_unchecked(&mut results, plant_rose()).await;

    drop(plant.sender);

    push(person_bob(), &person.sender);
    let item = results.next().await.unwrap();
    assert_eq!(item.value, person_bob());

    push(animal_spider(), &animal.sender);
    let item = results.next().await.unwrap();
    assert_eq!(item.value, animal_spider());

    drop(person.sender);
    drop(animal.sender);
    let next = results.next().await;
    assert!(next.is_none(), "Expected stream to end after all closed");
}

#[tokio::test]
async fn test_ordered_merge_large_volume() {
    // Arrange
    let (stream1, stream2) = TestChannels::two();

    let streams_list = vec![stream1.stream, stream2.stream];
    let results = streams_list.ordered_merge();

    // Act
    for _ in 0..500 {
        push(person_alice(), &stream1.sender);
        push(animal_dog(), &stream2.sender);
    }

    // Assert
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
async fn test_ordered_merge_maximum_concurrent_streams() {
    let num_concurrent = 50;
    let mut handles = Vec::new();

    for _i in 0..num_concurrent {
        let handle = tokio::spawn(async move {
            // Arrange
            let (stream1, stream2, stream3) = TestChannels::three();

            // Act
            push(person_alice(), &stream1.sender);
            push(animal_dog(), &stream2.sender);
            push(plant_rose(), &stream3.sender);

            let streams_list = vec![stream1.stream, stream2.stream, stream3.stream];
            let results = streams_list.ordered_merge();
            let mut results = Box::pin(results);

            // Assert
            let first = results.next().await.unwrap();
            assert_eq!(first.value, person_alice());

            let second = results.next().await.unwrap();
            assert_eq!(second.value, animal_dog());

            let third = results.next().await.unwrap();
            assert_eq!(third.value, plant_rose());

            // Act (more)
            let (stream4, stream5) = TestChannels::two();
            push(person_bob(), &stream4.sender);
            push(person_charlie(), &stream5.sender);

            let other_streams_list = vec![stream4.stream, stream5.stream];
            let mut results2 = Box::pin(other_streams_list.ordered_merge());

            // Assert (more)
            let fourth = results2.next().await.unwrap();
            assert_eq!(fourth.value, person_bob());

            let fifth = results2.next().await.unwrap();
            assert_eq!(fifth.value, person_charlie());
        });

        handles.push(handle);
    }

    for handle in handles {
        handle
            .await
            .expect("Concurrent stream task should complete successfully");
    }
}
