// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion::{CombinedState, FluxionStream, Ordered};
use fluxion_test_utils::test_data::{
    animal_dog, person_alice, person_bob, person_charlie, plant_rose, TestData,
};
use fluxion_test_utils::TestChannel;
use futures::StreamExt;

static ALWAYS_TRUE: fn(&CombinedState<TestData>) -> bool = |_| true;

#[tokio::test]
async fn test_functional_combine_latest() {
    // Arrange
    let person_chan = TestChannel::new();
    let animal_chan = TestChannel::new();

    let person_sender = person_chan.sender.clone();
    let animal_sender = animal_chan.sender.clone();

    let person_stream = FluxionStream::new(person_chan.stream);
    let animal_stream = FluxionStream::new(animal_chan.stream);

    let mut combined = person_stream.combine_latest(vec![animal_stream], ALWAYS_TRUE);

    // Act
    person_sender.send(person_alice()).unwrap();
    animal_sender.send(animal_dog()).unwrap();

    // Assert
    let state = combined.next().await.unwrap();
    let combined_state = state.get();
    assert_eq!(combined_state.get_state()[0], person_alice());
    assert_eq!(combined_state.get_state()[1], animal_dog());
}

#[tokio::test]
async fn test_functional_combine_with_previous() {
    // Arrange
    let channel = TestChannel::new();
    let sender = channel.sender.clone();

    let stream = FluxionStream::new(channel.stream);
    let mut with_previous = stream.combine_with_previous();

    // Act
    sender.send(person_alice()).unwrap();
    sender.send(person_bob()).unwrap();
    sender.send(person_charlie()).unwrap();

    // Assert
    let (prev, curr) = with_previous.next().await.unwrap();
    assert!(prev.is_none());
    assert_eq!(curr.get(), &person_alice());

    let (prev, curr) = with_previous.next().await.unwrap();
    assert_eq!(prev.unwrap().get(), &person_alice());
    assert_eq!(curr.get(), &person_bob());

    let (prev, curr) = with_previous.next().await.unwrap();
    assert_eq!(prev.unwrap().get(), &person_bob());
    assert_eq!(curr.get(), &person_charlie());
}

#[tokio::test]
async fn test_functional_ordered_merge() {
    // Arrange
    let person_chan = TestChannel::new();
    let animal_chan = TestChannel::new();
    let plant_chan = TestChannel::new();

    let person_sender = person_chan.sender.clone();
    let animal_sender = animal_chan.sender.clone();
    let plant_sender = plant_chan.sender.clone();

    let person_stream = FluxionStream::new(person_chan.stream);
    let animal_stream = FluxionStream::new(animal_chan.stream);
    let plant_stream = FluxionStream::new(plant_chan.stream);

    let mut merged = person_stream.ordered_merge(vec![animal_stream, plant_stream]);

    // Act
    person_sender.send(person_alice()).unwrap();
    animal_sender.send(animal_dog()).unwrap();
    plant_sender.send(plant_rose()).unwrap();
    person_sender.send(person_bob()).unwrap();

    // Assert - items emitted in order they were pushed
    assert_eq!(merged.next().await.unwrap().get(), &person_alice());
    assert_eq!(merged.next().await.unwrap().get(), &animal_dog());
    assert_eq!(merged.next().await.unwrap().get(), &plant_rose());
    assert_eq!(merged.next().await.unwrap().get(), &person_bob());
}

#[tokio::test]
async fn test_functional_take_latest_when() {
    // Arrange
    let source_chan = TestChannel::new();
    let filter_chan = TestChannel::new();

    let source_sender = source_chan.sender.clone();
    let filter_sender = filter_chan.sender.clone();

    let source_stream = FluxionStream::new(source_chan.stream);
    let filter_stream = FluxionStream::new(filter_chan.stream);

    let mut filtered = source_stream.take_latest_when(filter_stream, ALWAYS_TRUE);

    // Act
    source_sender.send(person_bob()).unwrap();
    source_sender.send(person_charlie()).unwrap();
    filter_sender.send(person_alice()).unwrap();

    // Assert - latest buffered value emitted when filter updates
    assert_eq!(filtered.next().await.unwrap().get(), &person_charlie());
}

#[tokio::test]
async fn test_functional_take_while_with() {
    // Arrange
    let source_chan = TestChannel::new();
    let predicate_chan = TestChannel::new();

    let source_sender = source_chan.sender.clone();
    let predicate_sender = predicate_chan.sender.clone();

    let source_stream = FluxionStream::new(source_chan.stream);
    let predicate_stream = FluxionStream::new(predicate_chan.stream);

    let taken = source_stream.take_while_with(predicate_stream, |_| true);
    let mut taken = Box::pin(taken);

    // Act
    predicate_sender.send(person_alice()).unwrap();
    source_sender.send(person_bob()).unwrap();
    source_sender.send(person_charlie()).unwrap();

    // Assert
    assert_eq!(taken.next().await.unwrap(), person_bob());
    assert_eq!(taken.next().await.unwrap(), person_charlie());
}

#[tokio::test]
async fn test_functional_with_latest_from() {
    // Arrange
    let primary_chan = TestChannel::new();
    let secondary_chan = TestChannel::new();

    let primary_sender = primary_chan.sender.clone();
    let secondary_sender = secondary_chan.sender.clone();

    let primary_stream = FluxionStream::new(primary_chan.stream);
    let secondary_stream = FluxionStream::new(secondary_chan.into_stream());

    let mut combined = primary_stream.with_latest_from(secondary_stream, ALWAYS_TRUE);

    // Act
    secondary_sender.send(animal_dog()).unwrap();
    primary_sender.send(person_alice()).unwrap();
    primary_sender.send(person_bob()).unwrap();

    // Assert - primary drives emissions, secondary provides latest value
    let (primary_val, secondary_val) = combined.next().await.unwrap();
    assert_eq!(primary_val.get(), &person_alice());
    assert_eq!(secondary_val.get(), &animal_dog());

    let (primary_val, secondary_val) = combined.next().await.unwrap();
    assert_eq!(primary_val.get(), &person_bob());
    assert_eq!(secondary_val.get(), &animal_dog());
}

#[tokio::test]
async fn test_functional_chained_operations() {
    // Arrange
    let source_chan = TestChannel::new();
    let filter_chan = TestChannel::new();

    let source_sender = source_chan.sender.clone();
    let filter_sender = filter_chan.sender.clone();

    let source_stream = FluxionStream::new(source_chan.stream);
    let filter_stream = FluxionStream::new(filter_chan.stream);

    let mut composed = source_stream
        .take_latest_when(filter_stream, ALWAYS_TRUE)
        .combine_with_previous();

    // Act
    source_sender.send(person_bob()).unwrap();
    source_sender.send(person_charlie()).unwrap();
    filter_sender.send(person_alice()).unwrap();

    // Assert
    let (prev, curr) = composed.next().await.unwrap();
    assert!(prev.is_none());
    assert_eq!(curr.get(), &person_charlie());
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
