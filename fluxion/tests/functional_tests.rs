// Copyright 2025 Umberto Gotti <umberto.gotti@umberto.gotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::StreamItem;
use fluxion_core::Timestamped;
use fluxion_rx::{CombinedState, FluxionStream};
use fluxion_test_utils::helpers::unwrap_stream;
use fluxion_test_utils::test_channel;
use fluxion_test_utils::test_data::{
    animal_dog, person_alice, person_bob, person_charlie, plant_rose, TestData,
};
use fluxion_test_utils::unwrap_value;
use fluxion_test_utils::ChronoTimestamped;
use futures::StreamExt;
use tokio::sync::mpsc::unbounded_channel;

static ALWAYS_TRUE: fn(&TestData) -> bool = |_| true;
static ALWAYS_TRUE_COMBINED: fn(&CombinedState<TestData>) -> bool = |_| true;
static RESULT_SELECTOR: fn(&CombinedState<TestData>) -> CombinedState<TestData> =
    |state: &CombinedState<TestData>| state.clone();

#[tokio::test]
async fn test_functional_combine_latest() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_stream) = test_channel::<ChronoTimestamped<TestData>>();
    let (animal_tx, animal_stream) = test_channel::<ChronoTimestamped<TestData>>();

    let person_stream = FluxionStream::new(person_stream);
    let animal_stream = FluxionStream::new(animal_stream);

    let mut combined = person_stream.combine_latest(vec![animal_stream], ALWAYS_TRUE_COMBINED);

    // Act
    person_tx.send(ChronoTimestamped::new(person_alice()))?;
    animal_tx.send(ChronoTimestamped::new(animal_dog()))?;

    // Assert
    let state = unwrap_stream(&mut combined, 100).await;
    match state {
        StreamItem::Value(ref ts) => {
            let combined_state = ts.clone();
            assert_eq!(combined_state.values()[0], person_alice());
            assert_eq!(combined_state.values()[1], animal_dog());
        }
        _ => panic!("Expected Value"),
    }

    Ok(())
}

#[tokio::test]
async fn test_functional_combine_with_previous() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<ChronoTimestamped<TestData>>();
    let mut with_previous = FluxionStream::new(stream).combine_with_previous();

    // Act
    tx.send(ChronoTimestamped::new(person_alice()))?;
    tx.send(ChronoTimestamped::new(person_bob()))?;
    tx.send(ChronoTimestamped::new(person_charlie()))?;

    // Assert
    let item = unwrap_value(Some(unwrap_stream(&mut with_previous, 500).await));
    assert!(item.previous.is_none());
    assert_eq!(&*item.current, &person_alice());

    let item = unwrap_value(Some(unwrap_stream(&mut with_previous, 500).await));
    assert_eq!(&*item.previous.unwrap(), &person_alice());
    assert_eq!(&*item.current, &person_bob());

    let item = unwrap_value(Some(unwrap_stream(&mut with_previous, 500).await));
    assert_eq!(&*item.previous.unwrap(), &person_bob());
    assert_eq!(&*item.current, &person_charlie());

    Ok(())
}

#[tokio::test]
async fn test_functional_ordered_merge() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_stream) = test_channel::<ChronoTimestamped<TestData>>();
    let (animal_tx, animal_stream) = test_channel::<ChronoTimestamped<TestData>>();
    let (plant_tx, plant_stream) = test_channel::<ChronoTimestamped<TestData>>();

    let person_stream = FluxionStream::new(person_stream);
    let animal_stream = FluxionStream::new(animal_stream);
    let plant_stream = FluxionStream::new(plant_stream);

    let mut merged = person_stream.ordered_merge(vec![animal_stream, plant_stream]);

    // Act
    person_tx.send(ChronoTimestamped::new(person_alice()))?;
    animal_tx.send(ChronoTimestamped::new(animal_dog()))?;
    plant_tx.send(ChronoTimestamped::new(plant_rose()))?;
    person_tx.send(ChronoTimestamped::new(person_bob()))?;

    // Assert - items emitted in order they were pushed
    match unwrap_stream(&mut merged, 100).await {
        StreamItem::Value(ref ts) => assert_eq!(&**ts, &person_alice()),
        _ => panic!("Expected Value"),
    }
    match unwrap_stream(&mut merged, 100).await {
        StreamItem::Value(ref ts) => assert_eq!(&**ts, &animal_dog()),
        _ => panic!("Expected Value"),
    }
    match unwrap_stream(&mut merged, 100).await {
        StreamItem::Value(ref ts) => assert_eq!(&**ts, &plant_rose()),
        _ => panic!("Expected Value"),
    }
    match unwrap_stream(&mut merged, 100).await {
        StreamItem::Value(ref ts) => assert_eq!(&**ts, &person_bob()),
        _ => panic!("Expected Value"),
    }

    Ok(())
}

#[tokio::test]
async fn test_functional_take_latest_when() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<ChronoTimestamped<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<ChronoTimestamped<TestData>>();

    let source_stream = FluxionStream::new(source_stream);
    let filter_stream = FluxionStream::new(filter_stream);

    let mut filtered = source_stream.take_latest_when(filter_stream, ALWAYS_TRUE);

    // Act
    source_tx.send(ChronoTimestamped::new(person_bob()))?;
    source_tx.send(ChronoTimestamped::new(person_charlie()))?;
    filter_tx.send(ChronoTimestamped::new(person_alice()))?;

    // Assert - latest buffered value emitted when filter updates
    match unwrap_stream(&mut filtered, 100).await {
        StreamItem::Value(ref ts) => assert_eq!(&**ts, &person_charlie()),
        _ => panic!("Expected Value"),
    }

    Ok(())
}

#[tokio::test]
async fn test_functional_take_while_with() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<ChronoTimestamped<TestData>>();
    let (predicate_tx, predicate_stream) = test_channel::<ChronoTimestamped<TestData>>();

    let source_stream = FluxionStream::new(source_stream);
    let predicate_stream = FluxionStream::new(predicate_stream);

    let mut taken = source_stream.take_while_with(predicate_stream, |_| true);

    // Act
    predicate_tx.send(ChronoTimestamped::new(person_alice()))?;
    let bob = ChronoTimestamped::new(person_bob());
    let charlie = ChronoTimestamped::new(person_charlie());
    source_tx.send(bob.clone())?;
    source_tx.send(charlie.clone())?;

    // Assert
    assert_eq!(taken.next().await.unwrap(), StreamItem::Value(bob));
    assert_eq!(taken.next().await.unwrap(), StreamItem::Value(charlie));

    Ok(())
}

#[tokio::test]
async fn test_functional_with_latest_from() -> anyhow::Result<()> {
    // Arrange
    let (primary_tx, primary_stream) = test_channel::<ChronoTimestamped<TestData>>();
    let (secondary_tx, secondary_stream) = test_channel::<ChronoTimestamped<TestData>>();

    let mut combined =
        FluxionStream::new(primary_stream).with_latest_from(secondary_stream, RESULT_SELECTOR);

    // Act
    secondary_tx.send(ChronoTimestamped::new(animal_dog()))?;
    primary_tx.send(ChronoTimestamped::new(person_alice()))?;
    primary_tx.send(ChronoTimestamped::new(person_bob()))?;

    // Assert - primary drives emissions, secondary provides latest value
    let result = unwrap_value(Some(unwrap_stream(&mut combined, 100).await));
    assert_eq!(result.clone().into_inner().values()[0], person_alice());
    assert_eq!(result.clone().into_inner().values()[1], animal_dog());

    let result = unwrap_value(Some(unwrap_stream(&mut combined, 100).await));
    assert_eq!(result.clone().into_inner().values()[0], person_bob());
    assert_eq!(result.clone().into_inner().values()[1], animal_dog());

    Ok(())
}

#[tokio::test]
async fn test_functional_chained_operations() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<ChronoTimestamped<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<ChronoTimestamped<TestData>>();

    let source_stream = FluxionStream::new(source_stream);
    let filter_stream = FluxionStream::new(filter_stream);

    let mut composed = source_stream
        .take_latest_when(filter_stream, ALWAYS_TRUE)
        .combine_with_previous();

    // Act
    source_tx.send(ChronoTimestamped::new(person_bob()))?;
    source_tx.send(ChronoTimestamped::new(person_charlie()))?;
    filter_tx.send(ChronoTimestamped::new(person_alice()))?;

    // Assert
    let item = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert!(item.previous.is_none());
    assert_eq!(&*item.current, &person_charlie());

    Ok(())
}

#[tokio::test]
async fn test_functional_from_unbounded_receiver() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = unbounded_channel();
    let mut stream = FluxionStream::from_unbounded_receiver(rx);

    // Act
    tx.send(person_alice())?;
    tx.send(person_bob())?;
    drop(tx);

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        person_alice()
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        person_bob()
    );
    assert!(stream.next().await.is_none());

    Ok(())
}
