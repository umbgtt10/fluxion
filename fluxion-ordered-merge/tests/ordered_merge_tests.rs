// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_ordered_merge::OrderedMergeExt;
use fluxion_test_utils::helpers::expect_next_timestamped_unchecked;
use fluxion_test_utils::sequenced::Sequenced;
use fluxion_test_utils::test_data::{
    TestData, animal_dog, animal_spider, person_alice, person_bob, person_charlie, plant_rose,
    plant_sunflower,
};
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[tokio::test]
async fn test_ordered_merge_empty_streams() {
    // Arrange
    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let person_stream = UnboundedReceiverStream::new(person_rx);
    let animal_stream = UnboundedReceiverStream::new(animal_rx);
    let plant_stream = UnboundedReceiverStream::new(plant_rx);

    let streams_list = vec![person_stream, animal_stream, plant_stream];
    let results = streams_list.ordered_merge();

    // Act
    drop(person_tx);
    drop(animal_tx);
    drop(plant_tx);

    // Assert
    let mut results = Box::pin(results);
    let next_item = results.next().await;
    assert!(next_item.is_none(), "Expected no items from empty streams");
}

#[tokio::test]
async fn test_ordered_merge_single_stream() {
    // Arrange
    let (tx, rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let stream = UnboundedReceiverStream::new(rx);

    let streams_list = vec![stream];
    let results = streams_list.ordered_merge();

    // Act
    tx.send(Sequenced::new(person_alice())).unwrap();
    tx.send(Sequenced::new(person_bob())).unwrap();
    tx.send(Sequenced::new(person_charlie())).unwrap();

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
    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let person_stream = UnboundedReceiverStream::new(person_rx);
    let animal_stream = UnboundedReceiverStream::new(animal_rx);
    let plant_stream = UnboundedReceiverStream::new(plant_rx);

    let streams_list = vec![person_stream, animal_stream, plant_stream];
    let results = streams_list.ordered_merge();

    // Act
    drop(animal_tx);

    person_tx.send(Sequenced::new(person_alice())).unwrap();
    plant_tx.send(Sequenced::new(plant_rose())).unwrap();
    person_tx.send(Sequenced::new(person_bob())).unwrap();

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
    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let person_stream = UnboundedReceiverStream::new(person_rx);
    let animal_stream = UnboundedReceiverStream::new(animal_rx);
    let plant_stream = UnboundedReceiverStream::new(plant_rx);

    let streams_list = vec![person_stream, animal_stream, plant_stream];
    let mut results = Box::pin(streams_list.ordered_merge());

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice())).unwrap();
    expect_next_timestamped_unchecked(&mut results, person_alice()).await;

    animal_tx.send(Sequenced::new(animal_dog())).unwrap();
    expect_next_timestamped_unchecked(&mut results, animal_dog()).await;

    person_tx.send(Sequenced::new(person_bob())).unwrap();
    expect_next_timestamped_unchecked(&mut results, person_bob()).await;

    plant_tx.send(Sequenced::new(plant_rose())).unwrap();
    expect_next_timestamped_unchecked(&mut results, plant_rose()).await;

    animal_tx.send(Sequenced::new(animal_spider())).unwrap();
    expect_next_timestamped_unchecked(&mut results, animal_spider()).await;

    plant_tx.send(Sequenced::new(plant_sunflower())).unwrap();
    expect_next_timestamped_unchecked(&mut results, plant_sunflower()).await;
}

#[tokio::test]
async fn test_ordered_merge_stream_completes_early() {
    // Arrange
    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let person_stream = UnboundedReceiverStream::new(person_rx);
    let animal_stream = UnboundedReceiverStream::new(animal_rx);

    let streams_list = vec![person_stream, animal_stream];
    let results = streams_list.ordered_merge();

    // Act
    person_tx.send(Sequenced::new(person_alice())).unwrap();
    animal_tx.send(Sequenced::new(animal_dog())).unwrap();
    drop(person_tx);
    animal_tx.send(Sequenced::new(animal_spider())).unwrap();

    // Assert
    let mut results = Box::pin(results);

    expect_next_timestamped_unchecked(&mut results, person_alice()).await;

    expect_next_timestamped_unchecked(&mut results, animal_dog()).await;

    expect_next_timestamped_unchecked(&mut results, animal_spider()).await;

    drop(animal_tx);

    let next_item = results.next().await;
    assert!(next_item.is_none(), "Expected stream to end");
}

#[tokio::test]
async fn test_ordered_merge_all_streams_close_simultaneously() {
    // Arrange
    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let person_stream = UnboundedReceiverStream::new(person_rx);
    let animal_stream = UnboundedReceiverStream::new(animal_rx);
    let plant_stream = UnboundedReceiverStream::new(plant_rx);

    let streams_list = vec![person_stream, animal_stream, plant_stream];
    let results = streams_list.ordered_merge();

    // Act
    person_tx.send(Sequenced::new(person_alice())).unwrap();
    animal_tx.send(Sequenced::new(animal_dog())).unwrap();
    plant_tx.send(Sequenced::new(plant_rose())).unwrap();

    drop(person_tx);
    drop(animal_tx);
    drop(plant_tx);

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
    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let person_stream = UnboundedReceiverStream::new(person_rx);
    let animal_stream = UnboundedReceiverStream::new(animal_rx);
    let plant_stream = UnboundedReceiverStream::new(plant_rx);

    let streams = vec![person_stream, animal_stream, plant_stream];
    let mut results = Box::pin(streams.ordered_merge());

    // Act & Assert stepwise
    person_tx.send(Sequenced::new(person_alice())).unwrap();
    expect_next_timestamped_unchecked(&mut results, person_alice()).await;

    animal_tx.send(Sequenced::new(animal_dog())).unwrap();
    expect_next_timestamped_unchecked(&mut results, animal_dog()).await;

    plant_tx.send(Sequenced::new(plant_rose())).unwrap();
    expect_next_timestamped_unchecked(&mut results, plant_rose()).await;

    drop(plant_tx);

    person_tx.send(Sequenced::new(person_bob())).unwrap();
    let item = results.next().await.unwrap();
    assert_eq!(item.value, person_bob());

    animal_tx.send(Sequenced::new(animal_spider())).unwrap();
    let item = results.next().await.unwrap();
    assert_eq!(item.value, animal_spider());

    drop(person_tx);
    drop(animal_tx);
    let next = results.next().await;
    assert!(next.is_none(), "Expected stream to end after all closed");
}

#[tokio::test]
async fn test_ordered_merge_large_volume() {
    // Arrange
    let (tx1, rx1) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (tx2, rx2) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let stream1 = UnboundedReceiverStream::new(rx1);
    let stream2 = UnboundedReceiverStream::new(rx2);

    let streams_list = vec![stream1, stream2];
    let results = streams_list.ordered_merge();

    // Act
    for _ in 0..500 {
        tx1.send(Sequenced::new(person_alice())).unwrap();
        tx2.send(Sequenced::new(animal_dog())).unwrap();
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
            let (tx1, rx1) = mpsc::unbounded_channel::<Sequenced<TestData>>();
            let (tx2, rx2) = mpsc::unbounded_channel::<Sequenced<TestData>>();
            let (tx3, rx3) = mpsc::unbounded_channel::<Sequenced<TestData>>();

            let stream1 = UnboundedReceiverStream::new(rx1);
            let stream2 = UnboundedReceiverStream::new(rx2);
            let stream3 = UnboundedReceiverStream::new(rx3);

            // Act
            tx1.send(Sequenced::new(person_alice())).unwrap();
            tx2.send(Sequenced::new(animal_dog())).unwrap();
            tx3.send(Sequenced::new(plant_rose())).unwrap();

            let streams_list = vec![stream1, stream2, stream3];
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
            let (tx4, rx4) = mpsc::unbounded_channel::<Sequenced<TestData>>();
            let (tx5, rx5) = mpsc::unbounded_channel::<Sequenced<TestData>>();

            let stream4 = UnboundedReceiverStream::new(rx4);
            let stream5 = UnboundedReceiverStream::new(rx5);

            tx4.send(Sequenced::new(person_bob())).unwrap();
            tx5.send(Sequenced::new(person_charlie())).unwrap();

            let other_streams_list = vec![stream4, stream5];
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
