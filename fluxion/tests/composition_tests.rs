// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion::{CombinedState, FluxionStream, Ordered, OrderedWrapper};
use fluxion_stream::combine_with_previous::WithPrevious;
use fluxion_test_utils::helpers::assert_no_element_emitted;
use fluxion_test_utils::sequenced::Sequenced;
use fluxion_test_utils::test_data::{
    animal_dog, person_alice, person_bob, person_charlie, person_dave, person_diane, plant_rose,
    TestData,
};
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

static FILTER: fn(&TestData) -> bool = |_| true;
static COMBINE_FILTER: fn(&CombinedState<TestData>) -> bool = |_| true;
static WITH_LATEST_FILTER: fn(&CombinedState<TestData>) -> bool = |_| true;
static LATEST_FILTER: fn(&TestData) -> bool = |_| true;
static LATEST_FILTER_COMBINED: fn(&CombinedState<TestData>) -> bool = |_| true;

#[tokio::test]
async fn test_fluxion_stream_composition() {
    // Arrange
    let (source_tx, source_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (_secondary_tx, _secondary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let source_stream = UnboundedReceiverStream::new(source_rx);
    let filter_stream = UnboundedReceiverStream::new(filter_rx);

    let composed = FluxionStream::new(source_stream)
        .take_latest_when(filter_stream, FILTER)
        .combine_with_previous();

    // Act & Assert
    filter_tx.send(Sequenced::new(person_alice())).unwrap();
    source_tx.send(Sequenced::new(person_alice())).unwrap();

    let mut composed = Box::pin(composed);
    let item = composed.next().await.unwrap();
    assert!(
        item.previous.is_none(),
        "First emission should have no previous"
    );
    assert_eq!(item.current.get(), &person_alice());

    source_tx.send(Sequenced::new(person_bob())).unwrap();
    let item = composed.next().await.unwrap();
    assert_eq!(item.previous.unwrap().get(), &person_alice());
    assert_eq!(item.current.get(), &person_bob());

    source_tx.send(Sequenced::new(person_charlie())).unwrap();
    let item = composed.next().await.unwrap();
    assert_eq!(item.previous.unwrap().get(), &person_bob());
    assert_eq!(item.current.get(), &person_charlie());

    drop(filter_tx);
    source_tx.send(Sequenced::new(person_dave())).unwrap();
    let item = composed.next().await.unwrap();
    assert_eq!(item.previous.unwrap().get(), &person_charlie());
    assert_eq!(item.current.get(), &person_dave());
}

#[tokio::test]
async fn test_fluxion_stream_combine_latest_composition() {
    // Arrange
    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let person_stream = UnboundedReceiverStream::new(person_rx);
    let animal_stream = UnboundedReceiverStream::new(animal_rx);
    let plant_stream = UnboundedReceiverStream::new(plant_rx);

    let combined = FluxionStream::new(person_stream)
        .combine_latest(vec![animal_stream, plant_stream], COMBINE_FILTER);

    // Act
    person_tx.send(Sequenced::new(person_alice())).unwrap();
    animal_tx.send(Sequenced::new(animal_dog())).unwrap();
    plant_tx.send(Sequenced::new(plant_rose())).unwrap();

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
    let (primary_tx, primary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (secondary_tx, secondary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let primary_stream = UnboundedReceiverStream::new(primary_rx);
    let secondary_stream = UnboundedReceiverStream::new(secondary_rx);

    let combined =
        FluxionStream::new(primary_stream).with_latest_from(secondary_stream, WITH_LATEST_FILTER);

    // Act
    secondary_tx.send(Sequenced::new(person_alice())).unwrap();
    primary_tx.send(Sequenced::new(person_bob())).unwrap();

    // Assert
    let mut combined = Box::pin(combined);
    let (sec, prim) = combined.next().await.unwrap();
    assert_eq!(prim.get(), &person_bob());
    assert_eq!(sec.get(), &person_alice());

    primary_tx.send(Sequenced::new(person_charlie())).unwrap();
    let (sec, prim) = combined.next().await.unwrap();
    assert_eq!(prim.get(), &person_charlie());
    assert_eq!(sec.get(), &person_alice());
}

#[tokio::test]
async fn test_fluxion_stream_combine_with_previous() {
    // Arrange
    let (tx, rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let stream = UnboundedReceiverStream::new(rx);

    let stream = FluxionStream::new(stream).combine_with_previous();

    // Act & Assert
    tx.send(Sequenced::new(person_alice())).unwrap();

    let mut stream = Box::pin(stream);
    let item = stream.next().await.unwrap();
    assert!(item.previous.is_none());
    assert_eq!(item.current.get(), &person_alice());

    tx.send(Sequenced::new(person_bob())).unwrap();
    let item = stream.next().await.unwrap();
    assert_eq!(item.previous.unwrap().get(), &person_alice());
    assert_eq!(item.current.get(), &person_bob());
}

#[tokio::test]
async fn test_fluxion_stream_take_while_with() {
    // Arrange
    let (source_tx, source_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_rx) = mpsc::unbounded_channel::<Sequenced<bool>>();

    let source_stream = UnboundedReceiverStream::new(source_rx);
    let filter_stream = UnboundedReceiverStream::new(filter_rx);

    let composed = FluxionStream::new(source_stream).take_while_with(filter_stream, |f| *f);

    // Act & Assert
    filter_tx.send(Sequenced::new(true)).unwrap();
    source_tx.send(Sequenced::new(person_alice())).unwrap();

    let mut composed = Box::pin(composed);
    assert_eq!(composed.next().await.unwrap(), person_alice());

    source_tx.send(Sequenced::new(person_bob())).unwrap();
    assert_eq!(composed.next().await.unwrap(), person_bob());

    filter_tx.send(Sequenced::new(false)).unwrap();
    source_tx.send(Sequenced::new(person_charlie())).unwrap();

    assert_no_element_emitted(&mut composed, 100).await;
}

#[tokio::test]
async fn test_fluxion_stream_take_latest_when_take_while() {
    // Arrange
    let (source_tx, source_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (latest_filter_tx, latest_filter_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (while_filter_tx, while_filter_rx) = mpsc::unbounded_channel::<Sequenced<bool>>();

    let source_stream = UnboundedReceiverStream::new(source_rx);
    let latest_filter_stream = UnboundedReceiverStream::new(latest_filter_rx);
    let while_filter_stream = UnboundedReceiverStream::new(while_filter_rx);

    let composed = FluxionStream::new(source_stream)
        .take_latest_when(latest_filter_stream, LATEST_FILTER)
        .take_while_with(while_filter_stream, |f| *f);

    // Act & Assert
    while_filter_tx.send(Sequenced::new(true)).unwrap();
    latest_filter_tx
        .send(Sequenced::new(person_alice()))
        .unwrap();
    source_tx.send(Sequenced::new(person_alice())).unwrap();

    let mut composed = Box::pin(composed);
    let result = composed.next().await.unwrap();
    assert_eq!(result, person_alice());

    source_tx.send(Sequenced::new(person_bob())).unwrap();
    let result = composed.next().await.unwrap();
    assert_eq!(result, person_bob());

    while_filter_tx.send(Sequenced::new(false)).unwrap();
    source_tx.send(Sequenced::new(person_charlie())).unwrap();
    assert_no_element_emitted(&mut composed, 100).await;
}

#[tokio::test]
async fn test_fluxion_stream_combine_latest_and_take_while() {
    // Arrange
    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_rx) = mpsc::unbounded_channel::<Sequenced<bool>>();

    let person_stream = UnboundedReceiverStream::new(person_rx);
    let animal_stream = UnboundedReceiverStream::new(animal_rx);
    let plant_stream = UnboundedReceiverStream::new(plant_rx);
    let filter_stream = UnboundedReceiverStream::new(filter_rx);

    let composed = FluxionStream::new(person_stream)
        .combine_latest(vec![animal_stream, plant_stream], COMBINE_FILTER)
        .take_while_with(filter_stream, |f| *f);

    // Act & Assert
    filter_tx.send(Sequenced::new(true)).unwrap();
    person_tx.send(Sequenced::new(person_alice())).unwrap();
    animal_tx.send(Sequenced::new(animal_dog())).unwrap();
    plant_tx.send(Sequenced::new(plant_rose())).unwrap();

    let mut composed = Box::pin(composed);
    let result = composed.next().await.unwrap();
    let state = result.get_state();
    assert_eq!(state.len(), 3);
    assert_eq!(state[0], person_alice());
    assert_eq!(state[1], animal_dog());
    assert_eq!(state[2], plant_rose());

    person_tx.send(Sequenced::new(person_bob())).unwrap();
    let result = composed.next().await.unwrap();
    let state = result.get_state();
    assert_eq!(state[0], person_bob());
    assert_eq!(state[1], animal_dog());
    assert_eq!(state[2], plant_rose());

    filter_tx.send(Sequenced::new(false)).unwrap();
    person_tx.send(Sequenced::new(person_charlie())).unwrap();
    assert_no_element_emitted(&mut composed, 100).await;
}

#[tokio::test]
async fn test_fluxion_stream_ordered_merge() {
    // Arrange
    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let person_stream = UnboundedReceiverStream::new(person_rx);
    let animal_stream = UnboundedReceiverStream::new(animal_rx);
    let plant_stream = UnboundedReceiverStream::new(plant_rx);

    let merged = FluxionStream::new(person_stream).ordered_merge(vec![
        FluxionStream::new(animal_stream),
        FluxionStream::new(plant_stream),
    ]);

    let mut merged = Box::pin(merged);

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice())).unwrap();
    animal_tx.send(Sequenced::new(animal_dog())).unwrap();
    plant_tx.send(Sequenced::new(plant_rose())).unwrap();

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
    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let person_stream = UnboundedReceiverStream::new(person_rx);
    let animal_stream = UnboundedReceiverStream::new(animal_rx);

    let composed = FluxionStream::new(person_stream)
        .ordered_merge(vec![FluxionStream::new(animal_stream)])
        .combine_with_previous();

    let mut composed = Box::pin(composed);

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice())).unwrap();
    let item = composed.next().await.unwrap();
    assert!(item.previous.is_none());
    assert_eq!(item.current.get(), &person_alice());

    animal_tx.send(Sequenced::new(animal_dog())).unwrap();
    let item = composed.next().await.unwrap();
    assert_eq!(item.previous.unwrap().get(), &person_alice());
    assert_eq!(item.current.get(), &animal_dog());

    person_tx.send(Sequenced::new(person_bob())).unwrap();
    let item = composed.next().await.unwrap();
    assert_eq!(item.previous.unwrap().get(), &animal_dog());
    assert_eq!(item.current.get(), &person_bob());
}

#[tokio::test]
async fn test_combine_latest_then_combine_with_previous() {
    // Arrange
    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let person_stream = UnboundedReceiverStream::new(person_rx);
    let animal_stream = UnboundedReceiverStream::new(animal_rx);

    let composed = FluxionStream::new(person_stream)
        .combine_latest(vec![animal_stream], COMBINE_FILTER)
        .combine_with_previous();

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice())).unwrap();
    animal_tx.send(Sequenced::new(animal_dog())).unwrap();

    let mut composed = Box::pin(composed);
    let item = composed.next().await.unwrap();
    assert!(item.previous.is_none());
    let curr_state = item.current.get().get_state();
    assert_eq!(curr_state[0], person_alice());
    assert_eq!(curr_state[1], animal_dog());

    person_tx.send(Sequenced::new(person_bob())).unwrap();
    let item = composed.next().await.unwrap();
    let prev_seq = item.previous.unwrap();
    let prev_state = prev_seq.get().get_state();
    assert_eq!(prev_state[0], person_alice());
    assert_eq!(prev_state[1], animal_dog());
    let curr_state = item.current.get().get_state();
    assert_eq!(curr_state[0], person_bob());
    assert_eq!(curr_state[1], animal_dog());
}

#[tokio::test]
async fn test_combine_latest_then_take_latest_when() {
    // Arrange
    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_rx) = mpsc::unbounded_channel::<Sequenced<CombinedState<TestData>>>();

    let person_stream = UnboundedReceiverStream::new(person_rx);
    let animal_stream = UnboundedReceiverStream::new(animal_rx);
    let filter_stream = UnboundedReceiverStream::new(filter_rx);

    let filter_mapped = filter_stream.map(|seq| {
        let order = seq.order();
        OrderedWrapper::with_order(seq.into_inner(), order)
    });

    let composed = FluxionStream::new(person_stream)
        .combine_latest(vec![animal_stream], COMBINE_FILTER)
        .take_latest_when(filter_mapped, LATEST_FILTER_COMBINED);

    // Act & Assert
    let filter_state = CombinedState::new(vec![person_alice()]);
    filter_tx.send(Sequenced::new(filter_state)).unwrap();
    person_tx.send(Sequenced::new(person_alice())).unwrap();
    animal_tx.send(Sequenced::new(animal_dog())).unwrap();

    let mut composed = Box::pin(composed);
    let result = composed.next().await.unwrap();
    let state = result.get().get_state();
    assert_eq!(state.len(), 2);
    assert_eq!(state[0], person_alice());
    assert_eq!(state[1], animal_dog());

    person_tx.send(Sequenced::new(person_bob())).unwrap();
    let result = composed.next().await.unwrap();
    let state = result.get().get_state();
    assert_eq!(state[0], person_bob());
    assert_eq!(state[1], animal_dog());
}

#[tokio::test]
async fn test_ordered_merge_then_take_while_with() {
    // Arrange
    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_rx) = mpsc::unbounded_channel::<Sequenced<bool>>();

    let person_stream = UnboundedReceiverStream::new(person_rx);
    let animal_stream = UnboundedReceiverStream::new(animal_rx);
    let filter_stream = UnboundedReceiverStream::new(filter_rx);

    let composed = FluxionStream::new(person_stream)
        .ordered_merge(vec![FluxionStream::new(animal_stream)])
        .take_while_with(filter_stream, |f| *f);

    // Act & Assert
    filter_tx.send(Sequenced::new(true)).unwrap();
    person_tx.send(Sequenced::new(person_alice())).unwrap();

    let mut composed = Box::pin(composed);
    assert_eq!(composed.next().await.unwrap(), person_alice());

    animal_tx.send(Sequenced::new(animal_dog())).unwrap();
    assert_eq!(composed.next().await.unwrap(), animal_dog());

    person_tx.send(Sequenced::new(person_bob())).unwrap();
    assert_eq!(composed.next().await.unwrap(), person_bob());

    filter_tx.send(Sequenced::new(false)).unwrap();
    person_tx.send(Sequenced::new(person_charlie())).unwrap();
    assert_no_element_emitted(&mut composed, 100).await;
}

#[tokio::test]
async fn test_triple_composition_combine_latest_take_while_ordered_merge() {
    // Arrange
    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_rx) = mpsc::unbounded_channel::<Sequenced<bool>>();

    let person_stream = UnboundedReceiverStream::new(person_rx);
    let animal_stream = UnboundedReceiverStream::new(animal_rx);
    let filter_stream = UnboundedReceiverStream::new(filter_rx);

    let composed = FluxionStream::new(person_stream)
        .combine_latest(vec![animal_stream], COMBINE_FILTER)
        .take_while_with(filter_stream, |f| *f);

    // Act & Assert
    filter_tx.send(Sequenced::new(true)).unwrap();
    person_tx.send(Sequenced::new(person_alice())).unwrap();
    animal_tx.send(Sequenced::new(animal_dog())).unwrap();

    let mut composed = Box::pin(composed);
    let result = composed.next().await.unwrap();
    assert_eq!(result.get_state().len(), 2);
    assert_eq!(result.get_state()[0], person_alice());
    assert_eq!(result.get_state()[1], animal_dog());

    person_tx.send(Sequenced::new(person_bob())).unwrap();
    let result = composed.next().await.unwrap();
    assert_eq!(result.get_state()[0], person_bob());
    assert_eq!(result.get_state()[1], animal_dog());

    filter_tx.send(Sequenced::new(false)).unwrap();
    person_tx.send(Sequenced::new(person_charlie())).unwrap();
    assert_no_element_emitted(&mut composed, 100).await;
}

#[tokio::test]
async fn test_ordered_merge_then_take_latest_when() {
    // Arrange
    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let person_stream = UnboundedReceiverStream::new(person_rx);
    let animal_stream = UnboundedReceiverStream::new(animal_rx);
    let filter_stream = UnboundedReceiverStream::new(filter_rx);

    let composed = FluxionStream::new(person_stream)
        .ordered_merge(vec![FluxionStream::new(animal_stream)])
        .take_latest_when(filter_stream, LATEST_FILTER);

    // Act & Assert
    filter_tx.send(Sequenced::new(person_alice())).unwrap();
    person_tx.send(Sequenced::new(person_alice())).unwrap();

    let mut composed = Box::pin(composed);
    assert_eq!(composed.next().await.unwrap().get(), &person_alice());

    animal_tx.send(Sequenced::new(animal_dog())).unwrap();
    assert_eq!(composed.next().await.unwrap().get(), &animal_dog());

    person_tx.send(Sequenced::new(person_bob())).unwrap();
    assert_eq!(composed.next().await.unwrap().get(), &person_bob());

    drop(filter_tx);
    person_tx.send(Sequenced::new(person_charlie())).unwrap();
    assert_eq!(composed.next().await.unwrap().get(), &person_charlie());
}

#[tokio::test]
async fn test_take_latest_when_then_ordered_merge() {
    // Arrange
    static LATEST_FILTER_LOCAL: fn(&TestData) -> bool = |_| true;

    let (source_tx, source_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let source_stream = UnboundedReceiverStream::new(source_rx);
    let filter_stream = UnboundedReceiverStream::new(filter_rx);
    let animal_stream = UnboundedReceiverStream::new(animal_rx);

    let composed = FluxionStream::new(source_stream)
        .take_latest_when(filter_stream, LATEST_FILTER_LOCAL)
        .ordered_merge(vec![FluxionStream::new(animal_stream)]);

    // Act & Assert
    filter_tx.send(Sequenced::new(person_alice())).unwrap();
    source_tx.send(Sequenced::new(person_alice())).unwrap();
    animal_tx.send(Sequenced::new(animal_dog())).unwrap();

    let mut composed = Box::pin(composed);
    let result1 = composed.next().await.unwrap();
    let result2 = composed.next().await.unwrap();

    let values: Vec<_> = vec![result1.get(), result2.get()];
    assert!(values.contains(&&person_alice()));
    assert!(values.contains(&&animal_dog()));

    source_tx.send(Sequenced::new(person_bob())).unwrap();
    let result = composed.next().await.unwrap();
    assert_eq!(result.get(), &person_bob());

    drop(filter_tx);
    source_tx.send(Sequenced::new(person_charlie())).unwrap();
    let result = composed.next().await.unwrap();
    assert_eq!(result.get(), &person_charlie());
}

#[tokio::test]
async fn test_emit_when_composite_with_ordered_merge_and_combine_with_previous() {
    // Arrange
    let (person1_tx, person1_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (person2_tx, person2_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (threshold_tx, threshold_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let person1_stream = UnboundedReceiverStream::new(person1_rx);
    let person2_stream = UnboundedReceiverStream::new(person2_rx);
    let threshold_stream = UnboundedReceiverStream::new(threshold_rx);

    let filter_fn = |state: &CombinedState<TestData>| -> bool {
        let values = state.get_state();
        // Extract the current person's age - note that emit_when unwraps to Inner type (TestData)
        let current_age = match &values[0] {
            TestData::Person(p) => p.age,
            _ => return false,
        };
        // Extract the threshold age
        let threshold_age = match &values[1] {
            TestData::Person(p) => p.age,
            _ => return false,
        };
        current_age >= threshold_age
    };

    // Map the threshold stream to match the WithPrevious type
    let threshold_mapped = threshold_stream.map(|seq| WithPrevious::new(None, seq));

    // Chained composition: merge -> combine_with_previous -> emit_when
    let mut output_stream = Box::pin(
        FluxionStream::new(person1_stream)
            .ordered_merge(vec![FluxionStream::new(person2_stream)])
            .combine_with_previous()
            .emit_when(threshold_mapped, filter_fn),
    );

    // Act: Set threshold to Bob (age 30)
    threshold_tx.send(Sequenced::new(person_bob())).unwrap();

    // Act: Send Alice (25) from stream 1 - below threshold
    person1_tx.send(Sequenced::new(person_alice())).unwrap();

    // Assert: Should not emit (25 < 30)
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Send Charlie (35) from stream 2 - above threshold
    person2_tx.send(Sequenced::new(person_charlie())).unwrap();

    // Assert: Should emit (35 >= 30)
    let emitted = output_stream.next().await.unwrap();
    assert_eq!(
        emitted.current.get(),
        &person_charlie(),
        "Expected Charlie (35) to be emitted when >= threshold (30)"
    );

    // Act: Send Dave (28) from stream 1 - below threshold
    person1_tx.send(Sequenced::new(person_dave())).unwrap();

    // Assert: Should not emit (28 < 30)
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Send Diane (40) from stream 2 - above threshold
    person2_tx.send(Sequenced::new(person_diane())).unwrap();

    // Assert: Should emit (40 >= 30)
    let emitted = output_stream.next().await.unwrap();
    assert_eq!(
        emitted.current.get(),
        &person_diane(),
        "Expected Diane (40) to be emitted when >= threshold (30)"
    );

    // Act: Lower threshold to Alice (25)
    threshold_tx.send(Sequenced::new(person_alice())).unwrap();

    // Assert: Should re-emit Diane since she still meets the new threshold
    let emitted = output_stream.next().await.unwrap();
    assert_eq!(
        emitted.current.get(),
        &person_diane(),
        "Expected Diane (40) to be re-emitted when threshold changes to 25"
    );

    // Act: Send Bob (30) from stream 1 - meets new threshold
    person1_tx.send(Sequenced::new(person_bob())).unwrap();

    // Assert: Should emit (30 >= 25)
    let emitted = output_stream.next().await.unwrap();
    assert_eq!(
        emitted.current.get(),
        &person_bob(),
        "Expected Bob (30) to be emitted when >= threshold (25)"
    );
}

#[tokio::test]
async fn test_map_ordered_basic() {
    // Arrange
    let (tx, rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let stream = UnboundedReceiverStream::new(rx);

    let stream = FluxionStream::new(stream)
        .combine_with_previous()
        .map_ordered(|item| {
            format!(
                "Previous: {:?}, Current: {}",
                item.previous.map(|p| p.get().to_string()),
                item.current.get()
            )
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice())).unwrap();

    let mut stream = Box::pin(stream);
    let result = stream.next().await.unwrap();
    assert_eq!(
        result,
        "Previous: None, Current: Person[name=Alice, age=25]"
    );

    tx.send(Sequenced::new(person_bob())).unwrap();
    let result = stream.next().await.unwrap();
    assert_eq!(
        result,
        "Previous: Some(\"Person[name=Alice, age=25]\"), Current: Person[name=Bob, age=30]"
    );

    tx.send(Sequenced::new(person_charlie())).unwrap();
    let result = stream.next().await.unwrap();
    assert_eq!(
        result,
        "Previous: Some(\"Person[name=Bob, age=30]\"), Current: Person[name=Charlie, age=35]"
    );
}

#[tokio::test]
async fn test_map_ordered_to_struct() {
    // Arrange
    #[derive(Debug, PartialEq)]
    struct AgeComparison {
        previous_age: Option<u32>,
        current_age: u32,
        age_increased: bool,
    }

    let (tx, rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let stream = UnboundedReceiverStream::new(rx);

    let stream = FluxionStream::new(stream)
        .combine_with_previous()
        .map_ordered(|item| {
            let current_age = match item.current.get() {
                TestData::Person(p) => p.age,
                _ => 0,
            };
            let previous_age = item.previous.and_then(|prev| match prev.get() {
                TestData::Person(p) => Some(p.age),
                _ => None,
            });
            let age_increased = previous_age.is_some_and(|prev| current_age > prev);

            AgeComparison {
                previous_age,
                current_age,
                age_increased,
            }
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice())).unwrap(); // Age 25

    let mut stream = Box::pin(stream);
    let result = stream.next().await.unwrap();
    assert_eq!(
        result,
        AgeComparison {
            previous_age: None,
            current_age: 25,
            age_increased: false,
        }
    );

    tx.send(Sequenced::new(person_bob())).unwrap(); // Age 30
    let result = stream.next().await.unwrap();
    assert_eq!(
        result,
        AgeComparison {
            previous_age: Some(25),
            current_age: 30,
            age_increased: true,
        }
    );

    tx.send(Sequenced::new(person_alice())).unwrap(); // Age 25 again
    let result = stream.next().await.unwrap();
    assert_eq!(
        result,
        AgeComparison {
            previous_age: Some(30),
            current_age: 25,
            age_increased: false,
        }
    );

    tx.send(Sequenced::new(person_charlie())).unwrap(); // Age 35
    let result = stream.next().await.unwrap();
    assert_eq!(
        result,
        AgeComparison {
            previous_age: Some(25),
            current_age: 35,
            age_increased: true,
        }
    );
}

#[tokio::test]
async fn test_map_ordered_extract_age_difference() {
    // Arrange
    let (tx, rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let stream = UnboundedReceiverStream::new(rx);

    let stream = FluxionStream::new(stream)
        .combine_with_previous()
        .map_ordered(|item| {
            let current_age = match item.current.get() {
                TestData::Person(p) => p.age,
                _ => 0,
            };
            let previous_age = item.previous.and_then(|prev| match prev.get() {
                TestData::Person(p) => Some(p.age),
                _ => None,
            });

            match previous_age {
                Some(prev) => current_age as i32 - prev as i32,
                None => 0,
            }
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice())).unwrap(); // Age 25

    let mut stream = Box::pin(stream);
    let result = stream.next().await.unwrap();
    assert_eq!(result, 0); // No previous

    tx.send(Sequenced::new(person_bob())).unwrap(); // Age 30
    let result = stream.next().await.unwrap();
    assert_eq!(result, 5); // 30 - 25 = 5

    tx.send(Sequenced::new(person_dave())).unwrap(); // Age 28
    let result = stream.next().await.unwrap();
    assert_eq!(result, -2); // 28 - 30 = -2

    tx.send(Sequenced::new(person_charlie())).unwrap(); // Age 35
    let result = stream.next().await.unwrap();
    assert_eq!(result, 7); // 35 - 28 = 7
}
