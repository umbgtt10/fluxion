// Copyright 2025 Umberto Gotti <umberto.gotti@umberto.gotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_rx::{CombinedState, FluxionStream, Ordered};
use fluxion_test_utils::test_data::{
    animal_dog, person_alice, person_bob, person_charlie, plant_rose, TestData,
};
use fluxion_test_utils::Sequenced;
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

static ALWAYS_TRUE: fn(&TestData) -> bool = |_| true;
static ALWAYS_TRUE_COMBINED: fn(&CombinedState<TestData>) -> bool = |_| true;
static RESULT_SELECTOR: fn(&CombinedState<TestData>) -> CombinedState<TestData> =
    |state: &CombinedState<TestData>| state.clone();

#[tokio::test]
async fn test_functional_combine_latest() {
    // Arrange
    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let person_stream = UnboundedReceiverStream::new(person_rx);

    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let animal_stream = UnboundedReceiverStream::new(animal_rx);

    let person_stream = FluxionStream::new(person_stream);
    let animal_stream = FluxionStream::new(animal_stream);

    let mut combined = person_stream.combine_latest(vec![animal_stream], ALWAYS_TRUE_COMBINED);

    // Act
    person_tx.send(Sequenced::new(person_alice())).unwrap();
    animal_tx.send(Sequenced::new(animal_dog())).unwrap();

    // Assert
    let state = combined.next().await.unwrap();
    let combined_state = state.get();
    assert_eq!(combined_state.get_state()[0], person_alice());
    assert_eq!(combined_state.get_state()[1], animal_dog());
}

#[tokio::test]
async fn test_functional_combine_with_previous() {
    // Arrange
    let (tx, rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let stream = UnboundedReceiverStream::new(rx);

    let stream = FluxionStream::new(stream);
    let mut with_previous = stream.combine_with_previous();

    // Act
    tx.send(Sequenced::new(person_alice())).unwrap();
    tx.send(Sequenced::new(person_bob())).unwrap();
    tx.send(Sequenced::new(person_charlie())).unwrap();

    // Assert
    let item = with_previous.next().await.unwrap();
    assert!(item.previous.is_none());
    assert_eq!(item.current.get(), &person_alice());

    let item = with_previous.next().await.unwrap();
    assert_eq!(item.previous.unwrap().get(), &person_alice());
    assert_eq!(item.current.get(), &person_bob());

    let item = with_previous.next().await.unwrap();
    assert_eq!(item.previous.unwrap().get(), &person_bob());
    assert_eq!(item.current.get(), &person_charlie());
}

#[tokio::test]
async fn test_functional_ordered_merge() {
    // Arrange
    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let person_stream = UnboundedReceiverStream::new(person_rx);

    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let animal_stream = UnboundedReceiverStream::new(animal_rx);

    let (plant_tx, plant_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let plant_stream = UnboundedReceiverStream::new(plant_rx);

    let person_stream = FluxionStream::new(person_stream);
    let animal_stream = FluxionStream::new(animal_stream);
    let plant_stream = FluxionStream::new(plant_stream);

    let mut merged = person_stream.ordered_merge(vec![animal_stream, plant_stream]);

    // Act
    person_tx.send(Sequenced::new(person_alice())).unwrap();
    animal_tx.send(Sequenced::new(animal_dog())).unwrap();
    plant_tx.send(Sequenced::new(plant_rose())).unwrap();
    person_tx.send(Sequenced::new(person_bob())).unwrap();

    // Assert - items emitted in order they were pushed
    assert_eq!(merged.next().await.unwrap().get(), &person_alice());
    assert_eq!(merged.next().await.unwrap().get(), &animal_dog());
    assert_eq!(merged.next().await.unwrap().get(), &plant_rose());
    assert_eq!(merged.next().await.unwrap().get(), &person_bob());
}

#[tokio::test]
async fn test_functional_take_latest_when() {
    // Arrange
    let (source_tx, source_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let source_stream = UnboundedReceiverStream::new(source_rx);

    let (filter_tx, filter_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let filter_stream = UnboundedReceiverStream::new(filter_rx);

    let source_stream = FluxionStream::new(source_stream);
    let filter_stream = FluxionStream::new(filter_stream);

    let mut filtered = source_stream.take_latest_when(filter_stream, ALWAYS_TRUE);

    // Act
    source_tx.send(Sequenced::new(person_bob())).unwrap();
    source_tx.send(Sequenced::new(person_charlie())).unwrap();
    filter_tx.send(Sequenced::new(person_alice())).unwrap();

    // Assert - latest buffered value emitted when filter updates
    assert_eq!(filtered.next().await.unwrap().get(), &person_charlie());
}

#[tokio::test]
async fn test_functional_take_while_with() {
    // Arrange
    let (source_tx, source_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let source_stream = UnboundedReceiverStream::new(source_rx);

    let (predicate_tx, predicate_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let predicate_stream = UnboundedReceiverStream::new(predicate_rx);

    let source_stream = FluxionStream::new(source_stream);
    let predicate_stream = FluxionStream::new(predicate_stream);

    let taken = source_stream.take_while_with(predicate_stream, |_| true);
    let mut taken = Box::pin(taken);

    // Act
    predicate_tx.send(Sequenced::new(person_alice())).unwrap();
    source_tx.send(Sequenced::new(person_bob())).unwrap();
    source_tx.send(Sequenced::new(person_charlie())).unwrap();

    // Assert
    assert_eq!(taken.next().await.unwrap(), person_bob());
    assert_eq!(taken.next().await.unwrap(), person_charlie());
}

#[tokio::test]
async fn test_functional_with_latest_from() {
    // Arrange
    let (primary_tx, primary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let primary_stream = UnboundedReceiverStream::new(primary_rx);

    let (secondary_tx, secondary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let secondary_stream = UnboundedReceiverStream::new(secondary_rx);

    let primary_stream = FluxionStream::new(primary_stream);

    let mut combined = primary_stream.with_latest_from(secondary_stream, RESULT_SELECTOR);

    // Act
    secondary_tx.send(Sequenced::new(animal_dog())).unwrap();
    primary_tx.send(Sequenced::new(person_alice())).unwrap();
    primary_tx.send(Sequenced::new(person_bob())).unwrap();

    // Assert - primary drives emissions, secondary provides latest value
    let result = combined.next().await.unwrap();
    assert_eq!(result.get().get_state()[0], person_alice());
    assert_eq!(result.get().get_state()[1], animal_dog());

    let result = combined.next().await.unwrap();
    assert_eq!(result.get().get_state()[0], person_bob());
    assert_eq!(result.get().get_state()[1], animal_dog());
}

#[tokio::test]
async fn test_functional_chained_operations() {
    // Arrange
    let (source_tx, source_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let source_stream = UnboundedReceiverStream::new(source_rx);

    let (filter_tx, filter_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let filter_stream = UnboundedReceiverStream::new(filter_rx);

    let source_stream = FluxionStream::new(source_stream);
    let filter_stream = FluxionStream::new(filter_stream);

    let mut composed = source_stream
        .take_latest_when(filter_stream, ALWAYS_TRUE)
        .combine_with_previous();

    // Act
    source_tx.send(Sequenced::new(person_bob())).unwrap();
    source_tx.send(Sequenced::new(person_charlie())).unwrap();
    filter_tx.send(Sequenced::new(person_alice())).unwrap();

    // Assert
    let item = composed.next().await.unwrap();
    assert!(item.previous.is_none());
    assert_eq!(item.current.get(), &person_charlie());
}

#[tokio::test]
async fn test_functional_from_unbounded_receiver() {
    // Arrange
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let mut stream = FluxionStream::from_unbounded_receiver(rx);

    // Act
    tx.send(person_alice()).unwrap();
    tx.send(person_bob()).unwrap();
    drop(tx);

    // Assert
    assert_eq!(stream.next().await.unwrap(), person_alice());
    assert_eq!(stream.next().await.unwrap(), person_bob());
    assert!(stream.next().await.is_none());
}
