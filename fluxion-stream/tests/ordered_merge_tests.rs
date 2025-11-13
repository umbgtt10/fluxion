use fluxion_stream::ordered_merge::OrderedMergeExt;
use fluxion_test_utils::FluxionChannel;
use fluxion_test_utils::test_data::{
    DataVariant, TestData, animal_dog, animal_spider, expect_variant, person_alice, person_bob,
    person_charlie, plant_rose, plant_sunflower, push, send_variant,
};
use fluxion_test_utils::{TestChannels, helpers::expect_next_timestamped};
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
    let streams = vec![person.stream, animal.stream, plant.stream];

    let results = streams.ordered_merge();

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
    let (person, animal, plant) = TestChannels::three::<TestData>();

    let streams = vec![person.stream, animal.stream, plant.stream];
    let results = streams.ordered_merge();

    drop(person.sender);
    drop(animal.sender);
    drop(plant.sender);

    let mut results = Box::pin(results);
    let next_item = results.next().await;
    assert!(next_item.is_none(), "Expected no items from empty streams");
}

#[tokio::test]
async fn test_ordered_merge_single_stream() {
    let channel = FluxionChannel::new();

    let streams = vec![channel.stream];
    let results = streams.ordered_merge();

    push(person_alice(), &channel.sender);
    push(person_bob(), &channel.sender);
    push(person_charlie(), &channel.sender);

    let mut results = Box::pin(results);

    expect_next_timestamped(&mut results, person_alice()).await;

    let item = results.next().await.unwrap();
    assert_eq!(item.value, person_bob());

    let item = results.next().await.unwrap();
    assert_eq!(item.value, person_charlie());
}

#[tokio::test]
async fn test_ordered_merge_one_empty_stream() {
    let person = FluxionChannel::new();
    let animal = FluxionChannel::new();
    let plant = FluxionChannel::new();

    let streams = vec![person.stream, animal.stream, plant.stream];
    let results = streams.ordered_merge();

    drop(animal.sender);

    push(person_alice(), &person.sender);
    push(plant_rose(), &plant.sender);
    push(person_bob(), &person.sender);

    let mut results = Box::pin(results);

    let item = results.next().await.unwrap();
    assert_eq!(item.value, person_alice());

    expect_next_timestamped(&mut results, plant_rose()).await;

    expect_next_timestamped(&mut results, person_bob()).await;
}

#[tokio::test]
async fn test_ordered_merge_interleaved_emissions() {
    let (person, animal, plant) = TestChannels::three();

    let streams = vec![person.stream, animal.stream, plant.stream];
    let mut results = Box::pin(streams.ordered_merge());

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
async fn test_ordered_merge_stream_completes_early() {
    let (person, animal) = TestChannels::two();

    let streams = vec![person.stream, animal.stream];
    let results = streams.ordered_merge();

    push(person_alice(), &person.sender);
    push(animal_dog(), &animal.sender);
    drop(person.sender);
    push(animal_spider(), &animal.sender);

    let mut results = Box::pin(results);

    expect_next_timestamped(&mut results, person_alice()).await;

    expect_next_timestamped(&mut results, animal_dog()).await;

    expect_next_timestamped(&mut results, animal_spider()).await;

    drop(animal.sender);

    let next_item = results.next().await;
    assert!(next_item.is_none(), "Expected stream to end");
}

#[tokio::test]
async fn test_ordered_merge_all_streams_close_simultaneously() {
    let (person, animal, plant) = TestChannels::three();

    let streams = vec![person.stream, animal.stream, plant.stream];
    let results = streams.ordered_merge();

    push(person_alice(), &person.sender);
    push(animal_dog(), &animal.sender);
    push(plant_rose(), &plant.sender);

    drop(person.sender);
    drop(animal.sender);
    drop(plant.sender);

    let mut results = Box::pin(results);

    expect_next_timestamped(&mut results, person_alice()).await;

    expect_next_timestamped(&mut results, animal_dog()).await;

    expect_next_timestamped(&mut results, plant_rose()).await;

    let next_item = results.next().await;
    assert!(next_item.is_none(), "Expected stream to end");
}

#[tokio::test]
async fn test_ordered_merge_one_stream_closes_midway_three_streams() {
    let (person, animal, plant) = TestChannels::three();

    let streams = vec![person.stream, animal.stream, plant.stream];
    let mut results = Box::pin(streams.ordered_merge());

    push(person_alice(), &person.sender);
    expect_next_timestamped(&mut results, person_alice()).await;

    push(animal_dog(), &animal.sender);
    expect_next_timestamped(&mut results, animal_dog()).await;

    push(plant_rose(), &plant.sender);
    expect_next_timestamped(&mut results, plant_rose()).await;

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
    let (stream1, stream2) = TestChannels::two();

    let streams = vec![stream1.stream, stream2.stream];
    let results = streams.ordered_merge();

    for _ in 0..500 {
        push(person_alice(), &stream1.sender);
        push(animal_dog(), &stream2.sender);
    }

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
            let (stream1, stream2, stream3) = TestChannels::three();

            push(person_alice(), &stream1.sender);
            push(animal_dog(), &stream2.sender);
            push(plant_rose(), &stream3.sender);

            let streams = vec![stream1.stream, stream2.stream, stream3.stream];
            let results = streams.ordered_merge();
            let mut results = Box::pin(results);

            let first = results.next().await.unwrap();
            assert_eq!(first.value, person_alice());

            let second = results.next().await.unwrap();
            assert_eq!(second.value, animal_dog());

            let third = results.next().await.unwrap();
            assert_eq!(third.value, plant_rose());

            let (stream4, stream5) = TestChannels::two();
            push(person_bob(), &stream4.sender);
            push(person_charlie(), &stream5.sender);

            let streams2 = vec![stream4.stream, stream5.stream];
            let mut results2 = Box::pin(streams2.ordered_merge());

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
