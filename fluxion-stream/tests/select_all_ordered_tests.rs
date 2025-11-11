use fluxion_stream::select_all_ordered::SelectAllExt;
use fluxion_stream::timestamped_channel::unbounded_channel;
use fluxion_test_utils::helpers::expect_next_timestamped;
use fluxion_test_utils::test_data::{
    DataVariant, TestData, animal_dog, animal_spider, expect_variant, person_alice, person_bob,
    person_charlie, plant_rose, plant_sunflower, push, send_variant,
};
use futures::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;

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
    let (person_sender, person_receiver) = unbounded_channel();
    let (animal_sender, animal_receiver) = unbounded_channel();
    let (plant_sender, plant_receiver) = unbounded_channel();

    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let senders = vec![person_sender, animal_sender, plant_sender];
    let streams = vec![person_stream, animal_stream, plant_stream];

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
    let (_, person_receiver) = unbounded_channel::<TestData>();
    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());

    let (_, animal_receiver) = unbounded_channel::<TestData>();
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());

    let (_, plant_receiver) = unbounded_channel::<TestData>();
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let streams = vec![person_stream, animal_stream, plant_stream];
    let results = streams.select_all_ordered();

    // Assert
    let mut results = Box::pin(results);
    let next_item = results.next().await;
    assert!(next_item.is_none(), "Expected no items from empty streams");
}

#[tokio::test]
async fn test_select_all_ordered_single_stream() {
    // Arrange
    let (sender, receiver) = unbounded_channel();
    let stream = UnboundedReceiverStream::new(receiver.into_inner());

    let streams = vec![stream];
    let results = streams.select_all_ordered();

    // Act
    push(person_alice(), &sender);
    push(person_bob(), &sender);
    push(person_charlie(), &sender);

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
    let (person_sender, person_receiver) = unbounded_channel();
    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());

    let (_, animal_receiver) = unbounded_channel::<TestData>();
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());

    let (plant_sender, plant_receiver) = unbounded_channel();
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let streams = vec![person_stream, animal_stream, plant_stream];
    let results = streams.select_all_ordered();

    // Act
    push(person_alice(), &person_sender);
    push(plant_rose(), &plant_sender);
    push(person_bob(), &person_sender);

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
    let (person_sender, person_receiver) = unbounded_channel();
    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());

    let (animal_sender, animal_receiver) = unbounded_channel();
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());

    let (plant_sender, plant_receiver) = unbounded_channel();
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let streams = vec![person_stream, animal_stream, plant_stream];
    let mut results = Box::pin(streams.select_all_ordered());

    // Act & Assert - Send and verify one at a time to avoid race conditions
    push(person_alice(), &person_sender);
    expect_next_timestamped(&mut results, person_alice()).await;

    push(animal_dog(), &animal_sender);
    expect_next_timestamped(&mut results, animal_dog()).await;

    push(person_bob(), &person_sender);
    expect_next_timestamped(&mut results, person_bob()).await;

    push(plant_rose(), &plant_sender);
    expect_next_timestamped(&mut results, plant_rose()).await;

    push(animal_spider(), &animal_sender);
    expect_next_timestamped(&mut results, animal_spider()).await;

    push(plant_sunflower(), &plant_sender);
    expect_next_timestamped(&mut results, plant_sunflower()).await;
}

#[tokio::test]
async fn test_select_all_ordered_stream_completes_early() {
    // Arrange
    let (person_sender, person_receiver) = unbounded_channel();
    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());

    let (animal_sender, animal_receiver) = unbounded_channel();
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());

    let streams = vec![person_stream, animal_stream];
    let results = streams.select_all_ordered();

    // Act
    push(person_alice(), &person_sender);
    push(animal_dog(), &animal_sender);
    drop(person_sender); // Close person stream
    push(animal_spider(), &animal_sender);

    // Assert
    let mut results = Box::pin(results);

    expect_next_timestamped(&mut results, person_alice()).await;

    expect_next_timestamped(&mut results, animal_dog()).await;

    expect_next_timestamped(&mut results, animal_spider()).await;

    // Close remaining stream
    drop(animal_sender);

    // Assert stream ends
    let next_item = results.next().await;
    assert!(next_item.is_none(), "Expected stream to end");
}

#[tokio::test]
async fn test_select_all_ordered_all_streams_close_simultaneously() {
    // Arrange
    let (person_sender, person_receiver) = unbounded_channel();
    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());

    let (animal_sender, animal_receiver) = unbounded_channel();
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());

    let (plant_sender, plant_receiver) = unbounded_channel();
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let streams = vec![person_stream, animal_stream, plant_stream];
    let results = streams.select_all_ordered();

    // Act
    push(person_alice(), &person_sender);
    push(animal_dog(), &animal_sender);
    push(plant_rose(), &plant_sender);

    // Close all senders
    drop(person_sender);
    drop(animal_sender);
    drop(plant_sender);

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
    let (person_sender, person_receiver) = unbounded_channel();
    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());

    let (animal_sender, animal_receiver) = unbounded_channel();
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());

    let (plant_sender, plant_receiver) = unbounded_channel();
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let streams = vec![person_stream, animal_stream, plant_stream];
    let mut results = Box::pin(streams.select_all_ordered());

    // Act & Assert stepwise to avoid race conditions
    push(person_alice(), &person_sender);
    expect_next_timestamped(&mut results, person_alice()).await;

    push(animal_dog(), &animal_sender);
    expect_next_timestamped(&mut results, animal_dog()).await;

    push(plant_rose(), &plant_sender);
    expect_next_timestamped(&mut results, plant_rose()).await;

    // Close plant stream mid-flight
    drop(plant_sender);

    // Remaining streams continue to emit
    push(person_bob(), &person_sender);
    let item = results.next().await.unwrap();
    assert_eq!(item.value, person_bob());

    push(animal_spider(), &animal_sender);
    let item = results.next().await.unwrap();
    assert_eq!(item.value, animal_spider());

    // Close remaining streams and ensure end
    drop(person_sender);
    drop(animal_sender);
    let next = results.next().await;
    assert!(next.is_none(), "Expected stream to end after all closed");
}

#[tokio::test]
async fn test_select_all_ordered_large_volume() {
    // Arrange
    let (sender1, receiver1) = unbounded_channel();
    let stream1 = UnboundedReceiverStream::new(receiver1.into_inner());

    let (sender2, receiver2) = unbounded_channel();
    let stream2 = UnboundedReceiverStream::new(receiver2.into_inner());

    let streams = vec![stream1, stream2];
    let results = streams.select_all_ordered();

    // Act - Send 1000 items interleaved
    for _ in 0..500 {
        push(person_alice(), &sender1);
        push(animal_dog(), &sender2);
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
