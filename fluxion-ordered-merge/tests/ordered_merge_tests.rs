// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_ordered_merge::OrderedMergeExt;
use fluxion_test_utils::test_data::{
    animal_dog, animal_spider, person_alice, person_bob, person_charlie, plant_rose,
    plant_sunflower, TestData,
};
use fluxion_test_utils::ChronoTimestamped;
use futures::StreamExt;
use tokio::sync::mpsc::unbounded_channel;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[tokio::test]
async fn test_ordered_merge_empty_streams() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = unbounded_channel::<ChronoTimestamped<TestData>>();
    let (animal_tx, animal_rx) = unbounded_channel::<ChronoTimestamped<TestData>>();
    let (plant_tx, plant_rx) = unbounded_channel::<ChronoTimestamped<TestData>>();

    let person_stream = UnboundedReceiverStream::new(person_rx);
    let animal_stream = UnboundedReceiverStream::new(animal_rx);
    let plant_stream = UnboundedReceiverStream::new(plant_rx);

    let streams_list = vec![person_stream, animal_stream, plant_stream];
    let mut results = streams_list.ordered_merge();

    // Act
    drop(person_tx);
    drop(animal_tx);
    drop(plant_tx);

    // Assert
    let next_item = results.next().await;
    assert!(next_item.is_none(), "Expected no items from empty streams");

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_single_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = unbounded_channel::<ChronoTimestamped<TestData>>();
    let stream = UnboundedReceiverStream::new(rx);

    let mut ordered_stream = vec![stream].ordered_merge();

    // Act
    tx.send(ChronoTimestamped::new(person_alice()))?;
    tx.send(ChronoTimestamped::new(person_bob()))?;
    tx.send(ChronoTimestamped::new(person_charlie()))?;

    // Assert
    assert_eq!(ordered_stream.next().await.unwrap().value, person_alice());
    assert_eq!(ordered_stream.next().await.unwrap().value, person_bob());
    assert_eq!(ordered_stream.next().await.unwrap().value, person_charlie());

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_one_empty_stream() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = unbounded_channel::<ChronoTimestamped<TestData>>();
    let (animal_tx, animal_rx) = unbounded_channel::<ChronoTimestamped<TestData>>();
    let (plant_tx, plant_rx) = unbounded_channel::<ChronoTimestamped<TestData>>();

    let person_stream = UnboundedReceiverStream::new(person_rx);
    let animal_stream = UnboundedReceiverStream::new(animal_rx);
    let plant_stream = UnboundedReceiverStream::new(plant_rx);

    let streams_list = vec![person_stream, animal_stream, plant_stream];
    let mut results = streams_list.ordered_merge();

    // Act
    drop(animal_tx);

    person_tx.send(ChronoTimestamped::new(person_alice()))?;
    plant_tx.send(ChronoTimestamped::new(plant_rose()))?;
    person_tx.send(ChronoTimestamped::new(person_bob()))?;

    // Assert
    assert_eq!(results.next().await.unwrap().value, person_alice());
    assert_eq!(results.next().await.unwrap().value, plant_rose());
    assert_eq!(results.next().await.unwrap().value, person_bob());

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_interleaved_emissions() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = unbounded_channel::<ChronoTimestamped<TestData>>();
    let (animal_tx, animal_rx) = unbounded_channel::<ChronoTimestamped<TestData>>();
    let (plant_tx, plant_rx) = unbounded_channel::<ChronoTimestamped<TestData>>();

    let person_stream = UnboundedReceiverStream::new(person_rx);
    let animal_stream = UnboundedReceiverStream::new(animal_rx);
    let plant_stream = UnboundedReceiverStream::new(plant_rx);

    let mut results = vec![person_stream, animal_stream, plant_stream].ordered_merge();

    // Act & Assert
    person_tx.send(ChronoTimestamped::new(person_alice()))?;
    assert_eq!(results.next().await.unwrap().value, person_alice());

    animal_tx.send(ChronoTimestamped::new(animal_dog()))?;
    assert_eq!(results.next().await.unwrap().value, animal_dog());

    person_tx.send(ChronoTimestamped::new(person_bob()))?;
    assert_eq!(results.next().await.unwrap().value, person_bob());

    plant_tx.send(ChronoTimestamped::new(plant_rose()))?;
    assert_eq!(results.next().await.unwrap().value, plant_rose());

    animal_tx.send(ChronoTimestamped::new(animal_spider()))?;
    assert_eq!(results.next().await.unwrap().value, animal_spider());

    plant_tx.send(ChronoTimestamped::new(plant_sunflower()))?;
    assert_eq!(results.next().await.unwrap().value, plant_sunflower());

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_stream_completes_early() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = unbounded_channel::<ChronoTimestamped<TestData>>();
    let (animal_tx, animal_rx) = unbounded_channel::<ChronoTimestamped<TestData>>();

    let person_stream = UnboundedReceiverStream::new(person_rx);
    let animal_stream = UnboundedReceiverStream::new(animal_rx);

    let mut results = vec![person_stream, animal_stream].ordered_merge();

    // Act
    person_tx.send(ChronoTimestamped::new(person_alice()))?;
    animal_tx.send(ChronoTimestamped::new(animal_dog()))?;
    drop(person_tx);
    animal_tx.send(ChronoTimestamped::new(animal_spider()))?;

    // Assert
    assert_eq!(results.next().await.unwrap().value, person_alice());
    assert_eq!(results.next().await.unwrap().value, animal_dog());
    assert_eq!(results.next().await.unwrap().value, animal_spider());

    drop(animal_tx);

    let next_item = results.next().await;
    assert!(next_item.is_none(), "Expected stream to end");
    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_all_streams_close_simultaneously() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = unbounded_channel::<ChronoTimestamped<TestData>>();
    let (animal_tx, animal_rx) = unbounded_channel::<ChronoTimestamped<TestData>>();
    let (plant_tx, plant_rx) = unbounded_channel::<ChronoTimestamped<TestData>>();

    let person_stream = UnboundedReceiverStream::new(person_rx);
    let animal_stream = UnboundedReceiverStream::new(animal_rx);
    let plant_stream = UnboundedReceiverStream::new(plant_rx);

    let mut results = vec![person_stream, animal_stream, plant_stream].ordered_merge();

    // Act
    person_tx.send(ChronoTimestamped::new(person_alice()))?;
    animal_tx.send(ChronoTimestamped::new(animal_dog()))?;
    plant_tx.send(ChronoTimestamped::new(plant_rose()))?;

    drop(person_tx);
    drop(animal_tx);
    drop(plant_tx);

    // Assert
    assert_eq!(results.next().await.unwrap().value, person_alice());
    assert_eq!(results.next().await.unwrap().value, animal_dog());
    assert_eq!(results.next().await.unwrap().value, plant_rose());

    let next_item = results.next().await;
    assert!(next_item.is_none(), "Expected stream to end");

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_one_stream_closes_midway_three_streams() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = unbounded_channel::<ChronoTimestamped<TestData>>();
    let (animal_tx, animal_rx) = unbounded_channel::<ChronoTimestamped<TestData>>();
    let (plant_tx, plant_rx) = unbounded_channel::<ChronoTimestamped<TestData>>();

    let person_stream = UnboundedReceiverStream::new(person_rx);
    let animal_stream = UnboundedReceiverStream::new(animal_rx);
    let plant_stream = UnboundedReceiverStream::new(plant_rx);

    let mut results = vec![person_stream, animal_stream, plant_stream].ordered_merge();

    // Act & Assert stepwise
    person_tx.send(ChronoTimestamped::new(person_alice()))?;
    assert_eq!(results.next().await.unwrap().value, person_alice());

    animal_tx.send(ChronoTimestamped::new(animal_dog()))?;
    assert_eq!(results.next().await.unwrap().value, animal_dog());

    plant_tx.send(ChronoTimestamped::new(plant_rose()))?;
    assert_eq!(results.next().await.unwrap().value, plant_rose());

    drop(plant_tx);

    person_tx.send(ChronoTimestamped::new(person_bob()))?;
    assert_eq!(results.next().await.unwrap().value, person_bob());

    animal_tx.send(ChronoTimestamped::new(animal_spider()))?;
    assert_eq!(results.next().await.unwrap().value, animal_spider());

    drop(person_tx);
    drop(animal_tx);
    let next = results.next().await;
    assert!(next.is_none(), "Expected stream to end after all closed");

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_large_volume() -> anyhow::Result<()> {
    // Arrange
    let (tx1, rx1) = unbounded_channel::<ChronoTimestamped<TestData>>();
    let (tx2, rx2) = unbounded_channel::<ChronoTimestamped<TestData>>();

    let stream1 = UnboundedReceiverStream::new(rx1);
    let stream2 = UnboundedReceiverStream::new(rx2);

    let mut results = vec![stream1, stream2].ordered_merge();

    // Act
    for _ in 0..500 {
        tx1.send(ChronoTimestamped::new(person_alice()))?;
        tx2.send(ChronoTimestamped::new(animal_dog()))?;
    }

    // Assert
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

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_maximum_concurrent_streams() -> anyhow::Result<()> {
    let num_concurrent = 50;
    let mut handles = Vec::new();

    for _i in 0..num_concurrent {
        let handle = tokio::spawn(async move {
            // Arrange
            let (tx1, rx1) = unbounded_channel::<ChronoTimestamped<TestData>>();
            let (tx2, rx2) = unbounded_channel::<ChronoTimestamped<TestData>>();
            let (tx3, rx3) = unbounded_channel::<ChronoTimestamped<TestData>>();

            let stream1 = UnboundedReceiverStream::new(rx1);
            let stream2 = UnboundedReceiverStream::new(rx2);
            let stream3 = UnboundedReceiverStream::new(rx3);

            let mut results = vec![stream1, stream2, stream3].ordered_merge();

            // Act
            tx1.send(ChronoTimestamped::new(person_alice())).unwrap();
            tx2.send(ChronoTimestamped::new(animal_dog())).unwrap();
            tx3.send(ChronoTimestamped::new(plant_rose())).unwrap();

            // Assert
            assert_eq!(results.next().await.unwrap().value, person_alice());
            assert_eq!(results.next().await.unwrap().value, animal_dog());
            assert_eq!(results.next().await.unwrap().value, plant_rose());

            let (tx4, rx4) = unbounded_channel::<ChronoTimestamped<TestData>>();
            let (tx5, rx5) = unbounded_channel::<ChronoTimestamped<TestData>>();

            let stream4 = UnboundedReceiverStream::new(rx4);
            let stream5 = UnboundedReceiverStream::new(rx5);

            // Act
            tx4.send(ChronoTimestamped::new(person_bob())).unwrap();
            tx5.send(ChronoTimestamped::new(person_charlie())).unwrap();

            let mut results2 = vec![stream4, stream5].ordered_merge();

            // Assert
            assert_eq!(results2.next().await.unwrap().value, person_bob());
            assert_eq!(results2.next().await.unwrap().value, person_charlie());
        });

        handles.push(handle);
    }

    for handle in handles {
        handle
            .await
            .expect("Concurrent stream task should complete successfully");
    }

    Ok(())
}
