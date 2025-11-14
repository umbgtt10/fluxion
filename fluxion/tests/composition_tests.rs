// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion::{CombinedState, FluxionStream, Ordered, OrderedWrapper};
use fluxion_test_utils::helpers::assert_no_element_emitted;
use fluxion_test_utils::test_data::{
    animal_dog, person_alice, person_bob, person_charlie, person_dave, plant_rose, TestData,
};
use fluxion_test_utils::{FluxionChannel, TestChannels};
use futures::StreamExt;

static FILTER: fn(&TestData) -> bool = |_| true;
static COMBINE_FILTER: fn(&CombinedState<TestData>) -> bool = |_| true;
static WITH_LATEST_FILTER: fn(&CombinedState<TestData>) -> bool = |_| true;
static LATEST_FILTER: fn(&TestData) -> bool = |_| true;
static LATEST_FILTER_COMBINED: fn(&CombinedState<TestData>) -> bool = |_| true;

#[tokio::test]
async fn test_fluxion_stream_composition() {
    // Arrange
    let (source, filter, _secondary) = TestChannels::three::<TestData>();

    // Compose multiple operations using FluxionStream
    let composed = FluxionStream::new(source.stream)
        .take_latest_when(filter.stream, FILTER)
        .combine_with_previous();

    // Act & Assert
    filter.sender.send(person_alice()).unwrap();
    source.sender.send(person_alice()).unwrap();

    let mut composed = Box::pin(composed);
    let (prev, curr) = composed.next().await.unwrap();
    assert!(prev.is_none(), "First emission should have no previous");
    assert_eq!(curr.get(), &person_alice());

    source.sender.send(person_bob()).unwrap();
    let (prev, curr) = composed.next().await.unwrap();
    assert_eq!(prev.unwrap().get(), &person_alice());
    assert_eq!(curr.get(), &person_bob());

    source.sender.send(person_charlie()).unwrap();
    let (prev, curr) = composed.next().await.unwrap();
    assert_eq!(prev.unwrap().get(), &person_bob());
    assert_eq!(curr.get(), &person_charlie());

    drop(filter.sender);
    source.sender.send(person_dave()).unwrap();
    let (prev, curr) = composed.next().await.unwrap();
    assert_eq!(prev.unwrap().get(), &person_charlie());
    assert_eq!(curr.get(), &person_dave());
}

#[tokio::test]
async fn test_fluxion_stream_combine_latest_composition() {
    // Arrange
    let person: FluxionChannel<TestData> = FluxionChannel::new();
    let animal = FluxionChannel::new();
    let plant = FluxionChannel::new();

    let combined = FluxionStream::new(person.stream)
        .combine_latest(vec![animal.stream, plant.stream], COMBINE_FILTER);

    // Act
    person.sender.send(person_alice()).unwrap();
    animal.sender.send(animal_dog()).unwrap();
    plant.sender.send(plant_rose()).unwrap();

    // Assert
    let mut combined = Box::pin(combined);
    let result = combined.next().await.unwrap();
    let state = result.get().get_state();
    assert_eq!(state.len(), 3);
    assert_eq!(state[0], person_alice());
    assert_eq!(state[1], animal_dog());
    assert_eq!(state[2], plant_rose());
}

#[tokio::test]
async fn test_fluxion_stream_with_latest_from() {
    // Arrange
    let (primary, secondary) = TestChannels::two::<TestData>();

    let combined =
        FluxionStream::new(primary.stream).with_latest_from(secondary.stream, WITH_LATEST_FILTER);

    // Act
    secondary.sender.send(person_alice()).unwrap();
    primary.sender.send(person_bob()).unwrap();

    // Assert
    let mut combined = Box::pin(combined);
    let (sec, prim) = combined.next().await.unwrap();
    assert_eq!(prim.get(), &person_bob());
    assert_eq!(sec.get(), &person_alice());

    primary.sender.send(person_charlie()).unwrap();
    let (sec, prim) = combined.next().await.unwrap();
    assert_eq!(prim.get(), &person_charlie());
    assert_eq!(sec.get(), &person_alice());
}

#[tokio::test]
async fn test_fluxion_stream_combine_with_previous() {
    // Arrange
    let channel: FluxionChannel<TestData> = FluxionChannel::new();

    let stream = FluxionStream::new(channel.stream).combine_with_previous();

    // Act & Assert
    channel.sender.send(person_alice()).unwrap();

    let mut stream = Box::pin(stream);
    let (prev, curr) = stream.next().await.unwrap();
    assert!(prev.is_none());
    assert_eq!(curr.get(), &person_alice());

    channel.sender.send(person_bob()).unwrap();
    let (prev, curr) = stream.next().await.unwrap();
    assert_eq!(prev.unwrap().get(), &person_alice());
    assert_eq!(curr.get(), &person_bob());
}

#[tokio::test]
async fn test_fluxion_stream_take_while_with() {
    // Arrange
    let source: FluxionChannel<TestData> = FluxionChannel::new();
    let filter: FluxionChannel<bool> = FluxionChannel::new();

    let composed = FluxionStream::new(source.stream).take_while_with(filter.stream, |f| *f);

    // Act & Assert
    filter.sender.send(true).unwrap();
    source.sender.send(person_alice()).unwrap();

    let mut composed = Box::pin(composed);
    assert_eq!(composed.next().await.unwrap(), person_alice());

    source.sender.send(person_bob()).unwrap();
    assert_eq!(composed.next().await.unwrap(), person_bob());

    filter.sender.send(false).unwrap();
    source.sender.send(person_charlie()).unwrap();

    assert_no_element_emitted(&mut composed, 100).await;
}

#[tokio::test]
async fn test_fluxion_stream_take_latest_when_take_while() {
    // Arrange
    let source: FluxionChannel<TestData> = FluxionChannel::new();
    let latest_filter = FluxionChannel::new();
    let while_filter: FluxionChannel<bool> = FluxionChannel::new();

    let composed = FluxionStream::new(source.stream)
        .take_latest_when(latest_filter.stream, LATEST_FILTER)
        .take_while_with(while_filter.stream, |f| *f);

    // Act & Assert
    while_filter.sender.send(true).unwrap();
    latest_filter.sender.send(person_alice()).unwrap();
    source.sender.send(person_alice()).unwrap();

    let mut composed = Box::pin(composed);
    let result = composed.next().await.unwrap();
    assert_eq!(result, person_alice());

    source.sender.send(person_bob()).unwrap();
    let result = composed.next().await.unwrap();
    assert_eq!(result, person_bob());

    while_filter.sender.send(false).unwrap();
    source.sender.send(person_charlie()).unwrap();
    assert_no_element_emitted(&mut composed, 100).await;
}

#[tokio::test]
async fn test_fluxion_stream_combine_latest_and_take_while() {
    // Arrange
    let person: FluxionChannel<TestData> = FluxionChannel::new();
    let animal: FluxionChannel<TestData> = FluxionChannel::new();
    let plant: FluxionChannel<TestData> = FluxionChannel::new();
    let filter: FluxionChannel<bool> = FluxionChannel::new();

    let composed = FluxionStream::new(person.stream)
        .combine_latest(vec![animal.stream, plant.stream], COMBINE_FILTER)
        .take_while_with(filter.stream, |f| *f);

    // Act & Assert
    filter.sender.send(true).unwrap();
    person.sender.send(person_alice()).unwrap();
    animal.sender.send(animal_dog()).unwrap();
    plant.sender.send(plant_rose()).unwrap();

    let mut composed = Box::pin(composed);
    let result = composed.next().await.unwrap();
    let state = result.get_state();
    assert_eq!(state.len(), 3);
    assert_eq!(state[0], person_alice());
    assert_eq!(state[1], animal_dog());
    assert_eq!(state[2], plant_rose());

    person.sender.send(person_bob()).unwrap();
    let result = composed.next().await.unwrap();
    let state = result.get_state();
    assert_eq!(state[0], person_bob());
    assert_eq!(state[1], animal_dog());
    assert_eq!(state[2], plant_rose());

    filter.sender.send(false).unwrap();
    person.sender.send(person_charlie()).unwrap();
    assert_no_element_emitted(&mut composed, 100).await;
}

#[tokio::test]
async fn test_fluxion_stream_ordered_merge() {
    // Arrange
    let person = FluxionChannel::new();
    let animal = FluxionChannel::new();
    let plant = FluxionChannel::new();

    let merged = FluxionStream::new(person.stream).ordered_merge(vec![animal.stream, plant.stream]);

    let mut merged = Box::pin(merged);

    // Act & Assert
    person.sender.send(person_alice()).unwrap();
    animal.sender.send(animal_dog()).unwrap();
    plant.sender.send(plant_rose()).unwrap();

    let result1 = merged.next().await.unwrap();
    assert_eq!(result1.get(), &person_alice());

    let result2 = merged.next().await.unwrap();
    assert_eq!(result2.get(), &animal_dog());

    let result3 = merged.next().await.unwrap();
    assert_eq!(result3.get(), &plant_rose());
}

#[tokio::test]
async fn test_ordered_merge_then_combine_with_previous() {
    // Arrange
    let person = FluxionChannel::new();
    let animal = FluxionChannel::new();

    let composed = FluxionStream::new(person.stream)
        .ordered_merge(vec![animal.stream])
        .combine_with_previous();

    let mut composed = Box::pin(composed);

    // Act & Assert
    person.sender.send(person_alice()).unwrap();
    let (prev, curr) = composed.next().await.unwrap();
    assert!(prev.is_none());
    assert_eq!(curr.get(), &person_alice());

    animal.sender.send(animal_dog()).unwrap();
    let (prev, curr) = composed.next().await.unwrap();
    assert_eq!(prev.unwrap().get(), &person_alice());
    assert_eq!(curr.get(), &animal_dog());

    person.sender.send(person_bob()).unwrap();
    let (prev, curr) = composed.next().await.unwrap();
    assert_eq!(prev.unwrap().get(), &animal_dog());
    assert_eq!(curr.get(), &person_bob());
}

#[tokio::test]
async fn test_combine_latest_then_combine_with_previous() {
    // Arrange
    let person: FluxionChannel<TestData> = FluxionChannel::new();
    let animal: FluxionChannel<TestData> = FluxionChannel::new();

    let composed = FluxionStream::new(person.stream)
        .combine_latest(vec![animal.stream], COMBINE_FILTER)
        .combine_with_previous();

    // Act & Assert
    person.sender.send(person_alice()).unwrap();
    animal.sender.send(animal_dog()).unwrap();

    let mut composed = Box::pin(composed);
    let (prev, curr) = composed.next().await.unwrap();
    assert!(prev.is_none());
    let curr_state = curr.get().get_state();
    assert_eq!(curr_state[0], person_alice());
    assert_eq!(curr_state[1], animal_dog());

    person.sender.send(person_bob()).unwrap();
    let (prev, curr) = composed.next().await.unwrap();
    let prev_seq = prev.unwrap();
    let prev_state = prev_seq.get().get_state();
    assert_eq!(prev_state[0], person_alice());
    assert_eq!(prev_state[1], animal_dog());
    let curr_state = curr.get().get_state();
    assert_eq!(curr_state[0], person_bob());
    assert_eq!(curr_state[1], animal_dog());
}

#[tokio::test]
async fn test_combine_latest_then_take_latest_when() {
    // Arrange
    let person: FluxionChannel<TestData> = FluxionChannel::new();
    let animal: FluxionChannel<TestData> = FluxionChannel::new();
    let filter: FluxionChannel<CombinedState<TestData>> = FluxionChannel::new();

    let filter_mapped = filter.stream.map(|seq| {
        let order = seq.order();
        OrderedWrapper::with_order(seq.into_inner(), order)
    });

    let composed = FluxionStream::new(person.stream)
        .combine_latest(vec![animal.stream], COMBINE_FILTER)
        .take_latest_when(filter_mapped, LATEST_FILTER_COMBINED);

    // Act & Assert
    let filter_state = CombinedState::new(vec![person_alice()]);
    filter.sender.send(filter_state).unwrap();
    person.sender.send(person_alice()).unwrap();
    animal.sender.send(animal_dog()).unwrap();

    let mut composed = Box::pin(composed);
    let result = composed.next().await.unwrap();
    let state = result.get().get_state();
    assert_eq!(state.len(), 2);
    assert_eq!(state[0], person_alice());
    assert_eq!(state[1], animal_dog());

    person.sender.send(person_bob()).unwrap();
    let result = composed.next().await.unwrap();
    let state = result.get().get_state();
    assert_eq!(state[0], person_bob());
    assert_eq!(state[1], animal_dog());
}

#[tokio::test]
async fn test_ordered_merge_then_take_while_with() {
    // Arrange
    let person = FluxionChannel::new();
    let animal = FluxionChannel::new();
    let filter: FluxionChannel<bool> = FluxionChannel::new();

    let composed = FluxionStream::new(person.stream)
        .ordered_merge(vec![animal.stream])
        .take_while_with(filter.stream, |f| *f);

    // Act & Assert
    filter.sender.send(true).unwrap();
    person.sender.send(person_alice()).unwrap();

    let mut composed = Box::pin(composed);
    assert_eq!(composed.next().await.unwrap(), person_alice());

    animal.sender.send(animal_dog()).unwrap();
    assert_eq!(composed.next().await.unwrap(), animal_dog());

    person.sender.send(person_bob()).unwrap();
    assert_eq!(composed.next().await.unwrap(), person_bob());

    filter.sender.send(false).unwrap();
    person.sender.send(person_charlie()).unwrap();
    assert_no_element_emitted(&mut composed, 100).await;
}

#[tokio::test]
async fn test_triple_composition_combine_latest_take_while_ordered_merge() {
    // Arrange
    let person: FluxionChannel<TestData> = FluxionChannel::new();
    let animal: FluxionChannel<TestData> = FluxionChannel::new();
    let filter: FluxionChannel<bool> = FluxionChannel::new();

    let composed = FluxionStream::new(person.stream)
        .combine_latest(vec![animal.stream], COMBINE_FILTER)
        .take_while_with(filter.stream, |f| *f);

    // Act & Assert
    filter.sender.send(true).unwrap();
    person.sender.send(person_alice()).unwrap();
    animal.sender.send(animal_dog()).unwrap();

    let mut composed = Box::pin(composed);
    let result = composed.next().await.unwrap();
    assert_eq!(result.get_state().len(), 2);
    assert_eq!(result.get_state()[0], person_alice());
    assert_eq!(result.get_state()[1], animal_dog());

    person.sender.send(person_bob()).unwrap();
    let result = composed.next().await.unwrap();
    assert_eq!(result.get_state()[0], person_bob());
    assert_eq!(result.get_state()[1], animal_dog());

    filter.sender.send(false).unwrap();
    person.sender.send(person_charlie()).unwrap();
    assert_no_element_emitted(&mut composed, 100).await;
}

#[tokio::test]
async fn test_ordered_merge_then_take_latest_when() {
    // Arrange
    let person = FluxionChannel::new();
    let animal = FluxionChannel::new();
    let filter: FluxionChannel<TestData> = FluxionChannel::new();

    let composed = FluxionStream::new(person.stream)
        .ordered_merge(vec![animal.stream])
        .take_latest_when(filter.stream, LATEST_FILTER);

    // Act & Assert
    filter.sender.send(person_alice()).unwrap();
    person.sender.send(person_alice()).unwrap();

    let mut composed = Box::pin(composed);
    assert_eq!(composed.next().await.unwrap().get(), &person_alice());

    animal.sender.send(animal_dog()).unwrap();
    assert_eq!(composed.next().await.unwrap().get(), &animal_dog());

    person.sender.send(person_bob()).unwrap();
    assert_eq!(composed.next().await.unwrap().get(), &person_bob());

    drop(filter.sender);
    person.sender.send(person_charlie()).unwrap();
    assert_eq!(composed.next().await.unwrap().get(), &person_charlie());
}

#[tokio::test]
async fn test_take_latest_when_then_ordered_merge() {
    // Arrange
    static LATEST_FILTER_LOCAL: fn(&TestData) -> bool = |_| true;

    let source: FluxionChannel<TestData> = FluxionChannel::new();
    let filter: FluxionChannel<TestData> = FluxionChannel::new();
    let animal: FluxionChannel<TestData> = FluxionChannel::new();

    let composed = FluxionStream::new(source.stream)
        .take_latest_when(filter.stream, LATEST_FILTER_LOCAL)
        .ordered_merge(vec![animal.stream]);

    // Act & Assert
    filter.sender.send(person_alice()).unwrap();
    source.sender.send(person_alice()).unwrap();
    animal.sender.send(animal_dog()).unwrap();

    let mut composed = Box::pin(composed);
    let result1 = composed.next().await.unwrap();
    let result2 = composed.next().await.unwrap();

    let values: Vec<_> = vec![result1.get(), result2.get()];
    assert!(values.contains(&&person_alice()));
    assert!(values.contains(&&animal_dog()));

    source.sender.send(person_bob()).unwrap();
    let result = composed.next().await.unwrap();
    assert_eq!(result.get(), &person_bob());

    drop(filter.sender);
    source.sender.send(person_charlie()).unwrap();
    let result = composed.next().await.unwrap();
    assert_eq!(result.get(), &person_charlie());
}
