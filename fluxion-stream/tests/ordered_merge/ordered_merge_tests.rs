// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{HasTimestamp, Timestamped};
use fluxion_stream::OrderedStreamExt;
use fluxion_test_utils::helpers::{assert_stream_ended, test_channel, unwrap_stream};
use fluxion_test_utils::sequenced::Sequenced;
use fluxion_test_utils::test_data::{
    animal_bird, animal_dog, animal_spider, person_alice, person_bob, person_charlie, person_dave,
    plant_oak, plant_rose, plant_sunflower, TestData,
};
use tokio::spawn;

#[tokio::test]
async fn test_ordered_merge_empty_streams() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    drop(tx1);
    let (tx2, s2) = test_channel::<Sequenced<TestData>>();
    drop(tx2);

    // Act
    let mut merged = s1.ordered_merge(vec![s2]);

    // Assert
    assert_stream_ended(&mut merged, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_timestamp_ordering() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel::<Sequenced<TestData>>();

    let mut merged = s1.ordered_merge(vec![s2]);

    // Act
    tx1.unbounded_send(Sequenced::with_timestamp(person_charlie(), 3))?;
    tx2.unbounded_send(Sequenced::with_timestamp(person_alice(), 1))?;
    tx1.unbounded_send(Sequenced::with_timestamp(person_dave(), 4))?;
    tx2.unbounded_send(Sequenced::with_timestamp(person_bob(), 2))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut merged, 200).await.into_inner(),
        person_alice()
    );
    assert_eq!(
        unwrap_stream(&mut merged, 200).await.into_inner(),
        person_bob()
    );
    assert_eq!(
        unwrap_stream(&mut merged, 200).await.into_inner(),
        person_charlie()
    );
    assert_eq!(
        unwrap_stream(&mut merged, 200).await.into_inner(),
        person_dave()
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_multiple_streams() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_stream) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_stream) = test_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_stream) = test_channel::<Sequenced<TestData>>();

    let mut merged = person_stream.ordered_merge(vec![animal_stream, plant_stream]);

    // Act
    person_tx.unbounded_send(Sequenced::with_timestamp(person_alice(), 5))?;
    animal_tx.unbounded_send(Sequenced::with_timestamp(animal_dog(), 1))?;
    plant_tx.unbounded_send(Sequenced::with_timestamp(plant_rose(), 3))?;
    animal_tx.unbounded_send(Sequenced::with_timestamp(animal_spider(), 2))?;
    plant_tx.unbounded_send(Sequenced::with_timestamp(plant_oak(), 4))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut merged, 200).await.into_inner(),
        animal_dog()
    );
    assert_eq!(
        unwrap_stream(&mut merged, 200).await.into_inner(),
        animal_spider()
    );
    assert_eq!(
        unwrap_stream(&mut merged, 200).await.into_inner(),
        plant_rose()
    );
    assert_eq!(
        unwrap_stream(&mut merged, 200).await.into_inner(),
        plant_oak()
    );
    assert_eq!(
        unwrap_stream(&mut merged, 200).await.into_inner(),
        person_alice()
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_ends_when_all_closed() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel::<Sequenced<TestData>>();

    let mut merged = s1.ordered_merge(vec![s2]);

    // Act
    tx1.unbounded_send(Sequenced::with_timestamp(person_alice(), 1))?;
    tx2.unbounded_send(Sequenced::with_timestamp(person_bob(), 2))?;

    let _ = unwrap_stream(&mut merged, 200).await;
    let _ = unwrap_stream(&mut merged, 200).await;

    drop(tx1);
    drop(tx2);

    // Assert
    assert_stream_ended(&mut merged, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_one_stream_closes_early() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel::<Sequenced<TestData>>();

    let mut merged = s1.ordered_merge(vec![s2]);

    // Act
    tx1.unbounded_send(Sequenced::with_timestamp(person_alice(), 1))?;
    tx1.unbounded_send(Sequenced::with_timestamp(person_bob(), 2))?;
    drop(tx1);

    tx2.unbounded_send(Sequenced::with_timestamp(animal_dog(), 3))?;
    tx2.unbounded_send(Sequenced::with_timestamp(animal_spider(), 4))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut merged, 200).await.into_inner(),
        person_alice()
    );
    assert_eq!(
        unwrap_stream(&mut merged, 200).await.into_inner(),
        person_bob()
    );
    assert_eq!(
        unwrap_stream(&mut merged, 200).await.into_inner(),
        animal_dog()
    );
    assert_eq!(
        unwrap_stream(&mut merged, 200).await.into_inner(),
        animal_spider()
    );

    drop(tx2);
    assert_stream_ended(&mut merged, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_duplicate_timestamps() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel::<Sequenced<TestData>>();

    let mut merged = s1.ordered_merge(vec![s2]);

    // Act
    tx1.unbounded_send(Sequenced::with_timestamp(person_alice(), 1))?;
    tx2.unbounded_send(Sequenced::with_timestamp(animal_dog(), 1))?;
    tx1.unbounded_send(Sequenced::with_timestamp(person_bob(), 2))?;
    tx2.unbounded_send(Sequenced::with_timestamp(animal_spider(), 2))?;

    // Assert
    let first = unwrap_stream(&mut merged, 200).await.into_inner();
    let second = unwrap_stream(&mut merged, 200).await.into_inner();
    let third = unwrap_stream(&mut merged, 200).await.into_inner();
    let fourth = unwrap_stream(&mut merged, 200).await.into_inner();

    assert!(first == person_alice() || first == animal_dog());
    assert!(second == person_alice() || second == animal_dog());
    assert_ne!(first, second);

    assert!(third == person_bob() || third == animal_spider());
    assert!(fourth == person_bob() || fourth == animal_spider());
    assert_ne!(third, fourth);

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_large_volume() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel::<Sequenced<i32>>();
    let (tx2, s2) = test_channel::<Sequenced<i32>>();

    let mut merged = s1.ordered_merge(vec![s2]);

    // Act
    for i in 0..1000 {
        tx1.unbounded_send(Sequenced::with_timestamp(i * 2, (i * 2) as u64))?;
        tx2.unbounded_send(Sequenced::with_timestamp(i * 2 + 1, (i * 2 + 1) as u64))?;
    }

    // Assert
    for expected in 0..2000 {
        let item = unwrap_stream(&mut merged, 200).await;
        assert_eq!(item.into_inner(), expected);
    }

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_parallel_sends() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel::<Sequenced<TestData>>();

    let mut merged = s1.ordered_merge(vec![s2]);

    // Act
    spawn(async move {
        for i in 0..100 {
            tx1.unbounded_send(Sequenced::with_timestamp(person_alice(), (i * 2) as u64))
                .unwrap();
        }
    });

    spawn(async move {
        for i in 0..100 {
            tx2.unbounded_send(Sequenced::with_timestamp(animal_dog(), (i * 2 + 1) as u64))
                .unwrap();
        }
    });

    // Assert: All 200 items should be emitted in timestamp order
    let mut last_ts = 0;
    for _ in 0..200 {
        let item = unwrap_stream(&mut merged, 500).await;
        let ts = item.timestamp();
        assert!(ts >= last_ts, "Timestamps must be non-decreasing");
        last_ts = ts;
    }

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_preserves_timestamp_metadata() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel::<Sequenced<TestData>>();

    let mut merged = s1.ordered_merge(vec![s2]);

    // Act
    tx1.unbounded_send(Sequenced::with_timestamp(person_alice(), 10))?;
    tx2.unbounded_send(Sequenced::with_timestamp(animal_dog(), 20))?;

    // Assert: Timestamps should be preserved
    let first = unwrap_stream(&mut merged, 200).await;
    assert_eq!(first.timestamp(), 10);
    assert_eq!(first.into_inner(), person_alice());

    let second = unwrap_stream(&mut merged, 200).await;
    assert_eq!(second.timestamp(), 20);
    assert_eq!(second.into_inner(), animal_dog());

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_does_not_wait_for_all_streams() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    let (_tx2, s2) = test_channel::<Sequenced<TestData>>();

    let mut merged = s1.ordered_merge(vec![s2]);

    // Act
    tx1.unbounded_send(Sequenced::with_timestamp(person_alice(), 1))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut merged, 200).await.into_inner(),
        person_alice()
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_mixed_types_in_enum() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_stream) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_stream) = test_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_stream) = test_channel::<Sequenced<TestData>>();

    let mut merged = person_stream.ordered_merge(vec![animal_stream, plant_stream]);

    // Act
    person_tx.unbounded_send(Sequenced::with_timestamp(person_alice(), 1))?;
    animal_tx.unbounded_send(Sequenced::with_timestamp(animal_dog(), 2))?;
    plant_tx.unbounded_send(Sequenced::with_timestamp(plant_rose(), 3))?;
    person_tx.unbounded_send(Sequenced::with_timestamp(person_bob(), 4))?;
    animal_tx.unbounded_send(Sequenced::with_timestamp(animal_bird(), 5))?;
    plant_tx.unbounded_send(Sequenced::with_timestamp(plant_sunflower(), 6))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut merged, 200).await.into_inner(),
        person_alice()
    );
    assert_eq!(
        unwrap_stream(&mut merged, 200).await.into_inner(),
        animal_dog()
    );
    assert_eq!(
        unwrap_stream(&mut merged, 200).await.into_inner(),
        plant_rose()
    );
    assert_eq!(
        unwrap_stream(&mut merged, 200).await.into_inner(),
        person_bob()
    );
    assert_eq!(
        unwrap_stream(&mut merged, 200).await.into_inner(),
        animal_bird()
    );
    assert_eq!(
        unwrap_stream(&mut merged, 200).await.into_inner(),
        plant_sunflower()
    );

    Ok(())
}
