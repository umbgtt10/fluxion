// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::HasTimestamp;
use fluxion_stream::WindowByCountExt;
use fluxion_test_utils::helpers::{
    assert_no_element_emitted, assert_stream_ended, test_channel, unwrap_stream, unwrap_value,
};
use fluxion_test_utils::sequenced::Sequenced;
use fluxion_test_utils::test_data::{
    animal_cat, animal_dog, person_alice, person_bob, person_charlie, plant_rose, TestData,
};

#[tokio::test]
async fn test_window_emits_complete_windows() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream.window_by_count::<Sequenced<Vec<TestData>>>(2);

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        vec![person_alice(), person_bob()]
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        vec![person_charlie(), animal_dog()]
    );

    Ok(())
}

#[tokio::test]
async fn test_window_emits_partial_on_completion() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream.window_by_count::<Sequenced<Vec<TestData>>>(3);

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    drop(tx);

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        vec![person_alice(), person_bob()]
    );

    assert_stream_ended(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_window_size_one() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = Box::pin(stream.window_by_count::<Sequenced<Vec<TestData>>>(1));

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        vec![person_alice()]
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        vec![person_bob()]
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        vec![person_charlie()]
    );

    Ok(())
}

#[tokio::test]
async fn test_window_large_size() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream.window_by_count::<Sequenced<Vec<TestData>>>(5);

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(animal_cat()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        vec![
            person_alice(),
            person_bob(),
            person_charlie(),
            animal_dog(),
            animal_cat()
        ]
    );

    Ok(())
}

#[tokio::test]
async fn test_window_no_emission_until_complete() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream.window_by_count::<Sequenced<Vec<TestData>>>(3);

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        vec![person_alice(), person_bob(), person_charlie()]
    );

    Ok(())
}

#[tokio::test]
async fn test_window_timestamp_from_last_item() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream.window_by_count::<Sequenced<Vec<TestData>>>(3);

    // Act
    tx.unbounded_send(Sequenced::with_timestamp(person_alice(), 100))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    tx.unbounded_send(Sequenced::with_timestamp(person_bob(), 200))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    tx.unbounded_send(Sequenced::with_timestamp(person_charlie(), 300))?; // last in window
    let window = unwrap_stream(&mut result, 500).await.unwrap();

    // Assert
    assert_eq!(window.timestamp(), 300);
    assert_eq!(
        window.into_inner(),
        vec![person_alice(), person_bob(), person_charlie()]
    );

    Ok(())
}

#[tokio::test]
async fn test_window_partial_timestamp_from_last() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream.window_by_count::<Sequenced<Vec<TestData>>>(5);

    // Act
    tx.unbounded_send(Sequenced::with_timestamp(person_alice(), 100))?;
    tx.unbounded_send(Sequenced::with_timestamp(person_bob(), 200))?;
    drop(tx);

    // Assert
    let window = unwrap_stream(&mut result, 500).await.unwrap();
    assert_eq!(window.timestamp(), 200);
    assert_eq!(window.into_inner(), vec![person_alice(), person_bob()]);

    Ok(())
}

#[tokio::test]
async fn test_window_empty_stream_no_output() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream.window_by_count::<Sequenced<Vec<TestData>>>(3);

    // Act
    drop(tx);

    // Assert
    assert_stream_ended(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_window_multiple_complete_windows() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = Box::pin(stream.window_by_count::<Sequenced<Vec<TestData>>>(2));

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    tx.unbounded_send(Sequenced::new(animal_cat()))?;
    tx.unbounded_send(Sequenced::new(plant_rose()))?;
    tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        vec![person_alice(), person_bob()]
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        vec![animal_dog(), animal_cat()]
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        vec![plant_rose(), person_charlie()]
    );

    assert_no_element_emitted(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_window_mixed_types() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream.window_by_count::<Sequenced<Vec<TestData>>>(3);

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    tx.unbounded_send(Sequenced::new(plant_rose()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        vec![person_alice(), animal_dog(), plant_rose()]
    );

    Ok(())
}

#[tokio::test]
#[should_panic(expected = "window size must be at least 1")]
async fn test_window_zero_panics() {
    // Arrange
    let (_, stream) = test_channel::<Sequenced<TestData>>();

    // Act & Assert
    let _ = stream.window_by_count::<Sequenced<Vec<TestData>>>(0);
}
