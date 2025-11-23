// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_ordered_merge::OrderedMergeExt;
use fluxion_test_utils::test_data::{
    animal_dog, animal_spider, person_alice, person_bob, person_charlie, plant_rose,
    plant_sunflower, TestData,
};
use fluxion_test_utils::{
    assert_stream_ended, test_channel, unwrap_stream, unwrap_value, Sequenced,
};

#[tokio::test]
async fn test_ordered_merge_empty_streams() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_rx) = test_channel::<Sequenced<TestData>>();

    let streams_list = vec![person_rx, animal_rx, plant_rx];
    let mut results = streams_list.ordered_merge();

    // Act
    drop(person_tx);
    drop(animal_tx);
    drop(plant_tx);

    // Assert
    assert_stream_ended(&mut results, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_single_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = test_channel::<Sequenced<TestData>>();
    let mut ordered_stream = vec![rx].ordered_merge();

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut ordered_stream, 100).await)).value,
        person_alice()
    );

    tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut ordered_stream, 100).await)).value,
        person_bob()
    );

    tx.send(Sequenced::new(person_charlie()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut ordered_stream, 100).await)).value,
        person_charlie()
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_one_empty_stream() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_rx) = test_channel::<Sequenced<TestData>>();

    let streams_list = vec![person_rx, animal_rx, plant_rx];
    let mut results = streams_list.ordered_merge();

    // Act & Assert
    drop(animal_tx);

    person_tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut results, 100).await)).value,
        person_alice()
    );

    plant_tx.send(Sequenced::new(plant_rose()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut results, 100).await)).value,
        plant_rose()
    );

    person_tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut results, 100).await)).value,
        person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_interleaved_emissions() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_rx) = test_channel::<Sequenced<TestData>>();

    let mut results = vec![person_rx, animal_rx, plant_rx].ordered_merge();

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut results, 100).await)).value,
        person_alice()
    );

    animal_tx.send(Sequenced::new(animal_dog()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut results, 100).await)).value,
        animal_dog()
    );

    person_tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut results, 100).await)).value,
        person_bob()
    );

    plant_tx.send(Sequenced::new(plant_rose()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut results, 100).await)).value,
        plant_rose()
    );

    animal_tx.send(Sequenced::new(animal_spider()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut results, 100).await)).value,
        animal_spider()
    );

    plant_tx.send(Sequenced::new(plant_sunflower()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut results, 100).await)).value,
        plant_sunflower()
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_stream_completes_early() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();

    let mut results = vec![person_rx, animal_rx].ordered_merge();

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;
    drop(person_tx);
    animal_tx.send(Sequenced::new(animal_spider()))?;

    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut results, 100).await)).value,
        person_alice()
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut results, 100).await)).value,
        animal_dog()
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut results, 100).await)).value,
        animal_spider()
    );

    drop(animal_tx);
    assert_stream_ended(&mut results, 500).await;
    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_all_streams_close_simultaneously() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_rx) = test_channel::<Sequenced<TestData>>();

    let mut results = vec![person_rx, animal_rx, plant_rx].ordered_merge();

    // Act
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;
    plant_tx.send(Sequenced::new(plant_rose()))?;

    drop(person_tx);
    drop(animal_tx);
    drop(plant_tx);

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut results, 100).await)).value,
        person_alice()
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut results, 100).await)).value,
        animal_dog()
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut results, 100).await)).value,
        plant_rose()
    );

    assert_stream_ended(&mut results, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_one_stream_closes_midway_three_streams() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_rx) = test_channel::<Sequenced<TestData>>();

    let mut results = vec![person_rx, animal_rx, plant_rx].ordered_merge();

    // Act & Assert stepwise
    person_tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut results, 100).await)).value,
        person_alice()
    );

    animal_tx.send(Sequenced::new(animal_dog()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut results, 100).await)).value,
        animal_dog()
    );

    plant_tx.send(Sequenced::new(plant_rose()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut results, 100).await)).value,
        plant_rose()
    );

    drop(plant_tx);

    person_tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut results, 100).await)).value,
        person_bob()
    );

    animal_tx.send(Sequenced::new(animal_spider()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut results, 100).await)).value,
        animal_spider()
    );

    drop(person_tx);
    drop(animal_tx);
    assert_stream_ended(&mut results, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_large_volume() -> anyhow::Result<()> {
    // Arrange
    let (tx1, rx1) = test_channel::<Sequenced<TestData>>();
    let (tx2, rx2) = test_channel::<Sequenced<TestData>>();

    let mut results = vec![rx1, rx2].ordered_merge();

    // Act
    for _ in 0..500 {
        tx1.send(Sequenced::new(person_alice()))?;
        tx2.send(Sequenced::new(animal_dog()))?;
    }

    // Assert
    let mut count = 0;

    for _ in 0..500 {
        let item = unwrap_value(Some(unwrap_stream(&mut results, 100).await));
        assert_eq!(item.value, person_alice());
        count += 1;

        let item = unwrap_value(Some(unwrap_stream(&mut results, 100).await));
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
            let (tx1, rx1) = test_channel::<Sequenced<TestData>>();
            let (tx2, rx2) = test_channel::<Sequenced<TestData>>();
            let (tx3, rx3) = test_channel::<Sequenced<TestData>>();

            let mut results = vec![rx1, rx2, rx3].ordered_merge();

            // Act
            tx1.send(Sequenced::new(person_alice())).unwrap();
            tx2.send(Sequenced::new(animal_dog())).unwrap();
            tx3.send(Sequenced::new(plant_rose())).unwrap();

            // Assert
            assert_eq!(
                unwrap_value(Some(unwrap_stream(&mut results, 100).await)).value,
                person_alice()
            );
            assert_eq!(
                unwrap_value(Some(unwrap_stream(&mut results, 100).await)).value,
                animal_dog()
            );
            assert_eq!(
                unwrap_value(Some(unwrap_stream(&mut results, 100).await)).value,
                plant_rose()
            );

            let (tx4, rx4) = test_channel::<Sequenced<TestData>>();
            let (tx5, rx5) = test_channel::<Sequenced<TestData>>();

            let mut results2 = vec![rx4, rx5].ordered_merge();

            // Act & Assert
            tx4.send(Sequenced::new(person_bob())).unwrap();
            assert_eq!(
                unwrap_value(Some(unwrap_stream(&mut results2, 100).await)).value,
                person_bob()
            );

            tx5.send(Sequenced::new(person_charlie())).unwrap();
            assert_eq!(
                unwrap_value(Some(unwrap_stream(&mut results2, 100).await)).value,
                person_charlie()
            );
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
