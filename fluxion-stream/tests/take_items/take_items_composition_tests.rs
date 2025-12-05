// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::StreamItem;
use fluxion_stream::{CombinedState, FluxionStream};
use fluxion_test_utils::test_data::TestData;
use fluxion_test_utils::{
    helpers::{assert_stream_ended, unwrap_stream},
    test_channel,
    test_data::{animal_dog, person_alice, person_bob, person_charlie, person_dave, person_diane},
    unwrap_value, Sequenced,
};

#[tokio::test]
async fn test_start_with_then_take_items() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<_>>();

    let initial = vec![
        StreamItem::Value(Sequenced::new(person_alice())),
        StreamItem::Value(Sequenced::new(person_bob())),
    ];

    let mut result = FluxionStream::new(stream).start_with(initial).take_items(3); // Take 2 initial + 1 from stream

    // Act
    tx.send(Sequenced::new(person_charlie()))?;
    tx.send(Sequenced::new(person_dave()))?; // Should not be emitted
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_alice()
    );
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_bob()
    );
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_charlie()
    );
    assert_stream_ended(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_skip_items_then_take_items() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<_>>();

    let mut result = FluxionStream::new(stream)
        .skip_items(2) // Skip first 2
        .take_items(2); // Then take next 2

    // Act
    tx.send(Sequenced::new(person_alice()))?; // Skipped
    tx.send(Sequenced::new(person_bob()))?; // Skipped
    tx.send(Sequenced::new(person_charlie()))?; // Taken
    tx.send(Sequenced::new(person_dave()))?; // Taken
    tx.send(Sequenced::new(person_alice()))?; // Not emitted (take limit reached)

    // Assert - Only charlie and dave
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_charlie()
    );
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_dave()
    );
    assert_stream_ended(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_start_with_skip_items_take_items() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<_>>();

    let initial = vec![
        StreamItem::Value(Sequenced::new(person_alice())), // age=25
        StreamItem::Value(Sequenced::new(person_dave())),  // age=28
    ];

    // Start with 2 initial values, skip 1, take 4
    let mut result = FluxionStream::new(stream)
        .start_with(initial)
        .skip_items(1) // Skip alice
        .take_items(4); // Take next 4: [dave, bob, charlie, diane]

    // Act
    tx.send(Sequenced::new(person_bob()))?; // age=30
    tx.send(Sequenced::new(person_charlie()))?; // age=35
    tx.send(Sequenced::new(person_diane()))?; // age=40
    tx.send(Sequenced::new(person_alice()))?; // Should not be emitted (take limit)

    // Assert - [dave, bob, charlie, diane]
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_dave()
    );
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_bob()
    );
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_charlie()
    );
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        person_diane()
    );

    assert_stream_ended(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_map_ordered_then_take_items() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut result = FluxionStream::new(stream)
        .map_ordered(|item| {
            let name = match item.value {
                TestData::Person(p) => p.name,
                TestData::Animal(a) => a.species,
                TestData::Plant(p) => p.species,
            };
            Sequenced::new(name)
        })
        .take_items(3);

    // Act
    tx.send(Sequenced::new(person_alice()))?;
    tx.send(Sequenced::new(person_bob()))?;
    tx.send(Sequenced::new(person_charlie()))?;
    tx.send(Sequenced::new(person_dave()))?; // Should not be emitted

    // Assert
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        "Alice"
    );
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        "Bob"
    );
    assert_eq!(
        unwrap_stream(&mut result, 100).await.unwrap().into_inner(),
        "Charlie"
    );
    assert_stream_ended(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_then_take_items() -> anyhow::Result<()> {
    // Arrange
    let (primary_tx, primary_rx) = test_channel::<Sequenced<TestData>>();
    let (secondary_tx, secondary_rx) = test_channel::<Sequenced<TestData>>();

    // Combine then take 2 items
    let mut stream = FluxionStream::new(primary_rx)
        .with_latest_from(
            FluxionStream::new(secondary_rx),
            |state: &CombinedState<TestData, u64>| -> Sequenced<String> {
                let values = state.values();
                let p_name = match &values[0] {
                    TestData::Person(p) => p.name.clone(),
                    _ => "Unknown".to_string(),
                };
                let s_name = match &values[1] {
                    TestData::Animal(a) => a.species.clone(),
                    _ => "Unknown".to_string(),
                };
                Sequenced::new(format!("{} with {}", p_name, s_name))
            },
        )
        .take_items(2);

    // Act & Assert
    secondary_tx.send(Sequenced::new(animal_dog()))?;

    // 1. First emission
    primary_tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Alice with Dog"
    );

    // 2. Second emission
    primary_tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Bob with Dog"
    );

    // 3. Third emission (Should be ignored/stream ended)
    primary_tx.send(Sequenced::new(person_charlie()))?;
    assert_stream_ended(&mut stream, 100).await;

    Ok(())
}
