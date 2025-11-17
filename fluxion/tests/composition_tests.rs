// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::StreamItem;
use fluxion_rx::{CombinedState, FluxionStream, Ordered, OrderedWrapper};
use fluxion_stream::WithPrevious;
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
    let item = composed.next().await.unwrap().unwrap();
    assert!(
        item.previous.is_none(),
        "First emission should have no previous"
    );
    assert_eq!(item.current.unwrap().get(), &person_alice());

    source_tx.send(Sequenced::new(person_bob())).unwrap();
    let item = composed.next().await.unwrap().unwrap();
    assert_eq!(item.previous.unwrap().unwrap().get(), &person_alice());
    assert_eq!(item.current.unwrap().get(), &person_bob());

    source_tx.send(Sequenced::new(person_charlie())).unwrap();
    let item = composed.next().await.unwrap().unwrap();
    assert_eq!(item.previous.unwrap().unwrap().get(), &person_bob());
    assert_eq!(item.current.unwrap().get(), &person_charlie());

    drop(filter_tx);
    source_tx.send(Sequenced::new(person_dave())).unwrap();
    let item = composed.next().await.unwrap().unwrap();
    assert_eq!(item.previous.unwrap().unwrap().get(), &person_charlie());
    assert_eq!(item.current.unwrap().get(), &person_dave());
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
    let result = combined.next().await.unwrap().unwrap();
    let state = result.get().get_state();
    assert_eq!(state.len(), 3);
    assert_eq!(state[0], person_alice());
    assert_eq!(state[1], animal_dog());
    assert_eq!(state[2], plant_rose());
}

#[tokio::test]
async fn test_fluxion_stream_with_latest_from() {
    // Arrange - Use a custom selector that creates a formatted summary
    let (primary_tx, primary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (secondary_tx, secondary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let primary_stream = UnboundedReceiverStream::new(primary_rx);
    let secondary_stream = UnboundedReceiverStream::new(secondary_rx);

    // Custom selector: create a descriptive string combining both values
    let summary_selector = |state: &CombinedState<TestData>| -> String {
        let primary = &state.get_state()[0];
        let secondary = &state.get_state()[1];
        format!("Primary: {:?}, Latest Secondary: {:?}", primary, secondary)
    };

    let combined =
        FluxionStream::new(primary_stream).with_latest_from(secondary_stream, summary_selector);

    // Act
    secondary_tx.send(Sequenced::new(person_alice())).unwrap();
    primary_tx.send(Sequenced::new(person_bob())).unwrap();

    // Assert
    let mut combined = Box::pin(combined);
    let result = combined.next().await.unwrap().unwrap();
    let summary = result.get();
    assert!(summary.contains("Bob"));
    assert!(summary.contains("Alice"));

    primary_tx.send(Sequenced::new(person_charlie())).unwrap();
    let result = combined.next().await.unwrap().unwrap();
    let summary = result.get();
    assert!(summary.contains("Charlie"));
    assert!(summary.contains("Alice"));
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
    let item = stream.next().await.unwrap().unwrap();
    assert!(item.previous.is_none());
    assert_eq!(item.current.get(), &person_alice());

    tx.send(Sequenced::new(person_bob())).unwrap();
    let item = stream.next().await.unwrap().unwrap();
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
    assert_eq!(composed.next().await.unwrap().unwrap(), person_alice());

    source_tx.send(Sequenced::new(person_bob())).unwrap();
    assert_eq!(composed.next().await.unwrap().unwrap(), person_bob());

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
    let result = composed.next().await.unwrap().unwrap();
    assert_eq!(result, person_alice());

    source_tx.send(Sequenced::new(person_bob())).unwrap();
    let result = composed.next().await.unwrap().unwrap();
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
    let result = composed.next().await.unwrap().unwrap();
    let state = result.get_state();
    assert_eq!(state.len(), 3);
    assert_eq!(state[0], person_alice());
    assert_eq!(state[1], animal_dog());
    assert_eq!(state[2], plant_rose());

    person_tx.send(Sequenced::new(person_bob())).unwrap();
    let result = composed.next().await.unwrap().unwrap();
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
    let item = composed.next().await.unwrap().unwrap();
    assert!(item.previous.is_none());
    assert_eq!(item.current.unwrap().get(), &person_alice());

    animal_tx.send(Sequenced::new(animal_dog())).unwrap();
    let item = composed.next().await.unwrap().unwrap();
    assert_eq!(item.previous.unwrap().unwrap().get(), &person_alice());
    assert_eq!(item.current.unwrap().get(), &animal_dog());

    person_tx.send(Sequenced::new(person_bob())).unwrap();
    let item = composed.next().await.unwrap().unwrap();
    assert_eq!(item.previous.unwrap().unwrap().get(), &animal_dog());
    assert_eq!(item.current.unwrap().get(), &person_bob());
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
    let item = composed.next().await.unwrap().unwrap();
    assert!(item.previous.is_none());
    let curr_binding = item.current.unwrap();
    let curr_state = curr_binding.get().get_state();
    assert_eq!(curr_state[0], person_alice());
    assert_eq!(curr_state[1], animal_dog());

    person_tx.send(Sequenced::new(person_bob())).unwrap();
    let item = composed.next().await.unwrap().unwrap();
    let prev_seq = item.previous.unwrap();
    let prev_binding = prev_seq.unwrap();
    let prev_state = prev_binding.get().get_state();
    assert_eq!(prev_state[0], person_alice());
    assert_eq!(prev_state[1], animal_dog());
    let curr_binding = item.current.unwrap();
    let curr_state = curr_binding.get().get_state();
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
        StreamItem::Value(OrderedWrapper::with_order(seq.into_inner(), order))
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
    let result = composed.next().await.unwrap().unwrap();
    let state = result.get().get_state();
    assert_eq!(state.len(), 2);
    assert_eq!(state[0], person_alice());
    assert_eq!(state[1], animal_dog());

    person_tx.send(Sequenced::new(person_bob())).unwrap();
    let result = composed.next().await.unwrap().unwrap();
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
    assert_eq!(composed.next().await.unwrap().unwrap(), person_alice());

    animal_tx.send(Sequenced::new(animal_dog())).unwrap();
    assert_eq!(composed.next().await.unwrap().unwrap(), animal_dog());

    person_tx.send(Sequenced::new(person_bob())).unwrap();
    assert_eq!(composed.next().await.unwrap().unwrap(), person_bob());

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
    let result = composed.next().await.unwrap().unwrap();
    assert_eq!(result.get_state().len(), 2);
    assert_eq!(result.get_state()[0], person_alice());
    assert_eq!(result.get_state()[1], animal_dog());

    person_tx.send(Sequenced::new(person_bob())).unwrap();
    let result = composed.next().await.unwrap().unwrap();
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
        .take_latest_when(
            FluxionStream::new(filter_stream).map(StreamItem::Value),
            LATEST_FILTER,
        );

    // Act & Assert
    filter_tx.send(Sequenced::new(person_alice())).unwrap();
    person_tx.send(Sequenced::new(person_alice())).unwrap();

    let mut composed = Box::pin(composed);
    assert_eq!(
        composed.next().await.unwrap().unwrap().get(),
        &person_alice()
    );

    animal_tx.send(Sequenced::new(animal_dog())).unwrap();
    assert_eq!(composed.next().await.unwrap().unwrap().get(), &animal_dog());

    person_tx.send(Sequenced::new(person_bob())).unwrap();
    assert_eq!(composed.next().await.unwrap().unwrap().get(), &person_bob());

    drop(filter_tx);
    person_tx.send(Sequenced::new(person_charlie())).unwrap();
    assert_eq!(
        composed.next().await.unwrap().unwrap().get(),
        &person_charlie()
    );
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
        .take_latest_when(FluxionStream::new(filter_stream), LATEST_FILTER_LOCAL)
        .ordered_merge(vec![FluxionStream::new(
            FluxionStream::new(animal_stream).map(StreamItem::Value),
        )]);

    // Act & Assert
    filter_tx.send(Sequenced::new(person_alice())).unwrap();
    source_tx.send(Sequenced::new(person_alice())).unwrap();
    animal_tx.send(Sequenced::new(animal_dog())).unwrap();

    let mut composed = Box::pin(composed);
    let result1 = composed.next().await.unwrap().unwrap();
    let result2 = composed.next().await.unwrap().unwrap();

    let values: Vec<_> = vec![result1.get(), result2.get()];
    assert!(values.contains(&&person_alice()));
    assert!(values.contains(&&animal_dog()));

    source_tx.send(Sequenced::new(person_bob())).unwrap();
    let result = composed.next().await.unwrap().unwrap();
    assert_eq!(result.get(), &person_bob());

    drop(filter_tx);
    source_tx.send(Sequenced::new(person_charlie())).unwrap();
    let result = composed.next().await.unwrap().unwrap();
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
    let threshold_mapped = threshold_stream
        .map(|seq| StreamItem::Value(WithPrevious::new(None, StreamItem::Value(seq))));

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
    let emitted = output_stream.next().await.unwrap().unwrap().unwrap();
    assert_eq!(
        emitted.current.unwrap().get(),
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
    let emitted = output_stream.next().await.unwrap().unwrap().unwrap();
    assert_eq!(
        emitted.current.unwrap().get(),
        &person_diane(),
        "Expected Diane (40) to be emitted when >= threshold (30)"
    );

    // Act: Lower threshold to Alice (25)
    threshold_tx.send(Sequenced::new(person_alice())).unwrap();

    // Assert: Should re-emit Diane since she still meets the new threshold
    let emitted = output_stream.next().await.unwrap().unwrap().unwrap();
    assert_eq!(
        emitted.current.unwrap().get(),
        &person_diane(),
        "Expected Diane (40) to be re-emitted when threshold changes to 25"
    );

    // Act: Send Bob (30) from stream 1 - meets new threshold
    person1_tx.send(Sequenced::new(person_bob())).unwrap();

    // Assert: Should emit (30 >= 25)
    let emitted = output_stream.next().await.unwrap().unwrap().unwrap();
    assert_eq!(
        emitted.current.unwrap().get(),
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
        .map_ordered(|stream_item| {
            let item = stream_item.unwrap();
            StreamItem::Value(format!(
                "Previous: {:?}, Current: {}",
                item.previous.map(|p| p.get().to_string()),
                item.current.get()
            ))
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice())).unwrap();

    let mut stream = Box::pin(stream);
    let result = stream.next().await.unwrap().unwrap().unwrap();
    assert_eq!(
        result,
        "Previous: None, Current: Person[name=Alice, age=25]"
    );

    tx.send(Sequenced::new(person_bob())).unwrap();
    let result = stream.next().await.unwrap().unwrap().unwrap();
    assert_eq!(
        result,
        String::from(
            "Previous: Some(\"Person[name=Alice, age=25]\"), Current: Person[name=Bob, age=30]"
        )
    );

    tx.send(Sequenced::new(person_charlie())).unwrap();
    let result = stream.next().await.unwrap().unwrap().unwrap();
    assert_eq!(
        result,
        String::from(
            "Previous: Some(\"Person[name=Bob, age=30]\"), Current: Person[name=Charlie, age=35]"
        )
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
        .map_ordered(|stream_item| {
            let item = stream_item.unwrap();
            let current_age = match item.current.get() {
                TestData::Person(p) => p.age,
                _ => 0,
            };
            let previous_age = item.previous.and_then(|prev| match prev.get() {
                TestData::Person(p) => Some(p.age),
                _ => None,
            });
            let age_increased = previous_age.is_some_and(|prev| current_age > prev);

            StreamItem::Value(AgeComparison {
                previous_age,
                current_age,
                age_increased,
            })
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice())).unwrap(); // Age 25

    let mut stream = Box::pin(stream);
    let result = stream.next().await.unwrap().unwrap().unwrap();
    assert_eq!(
        result,
        AgeComparison {
            previous_age: None,
            current_age: 25,
            age_increased: false,
        }
    );

    tx.send(Sequenced::new(person_bob())).unwrap(); // Age 30
    let result = stream.next().await.unwrap().unwrap().unwrap();
    assert_eq!(
        result,
        AgeComparison {
            previous_age: Some(25),
            current_age: 30,
            age_increased: true,
        }
    );

    tx.send(Sequenced::new(person_alice())).unwrap(); // Age 25 again
    let result = stream.next().await.unwrap().unwrap().unwrap();
    assert_eq!(
        result,
        AgeComparison {
            previous_age: Some(30),
            current_age: 25,
            age_increased: false,
        }
    );

    tx.send(Sequenced::new(person_charlie())).unwrap(); // Age 35
    let result = stream.next().await.unwrap().unwrap().unwrap();
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
        .map_ordered(|stream_item| {
            let item = stream_item.unwrap();
            let current_age = match item.current.get() {
                TestData::Person(p) => p.age,
                _ => 0,
            };
            let previous_age = item.previous.and_then(|prev| match prev.get() {
                TestData::Person(p) => Some(p.age),
                _ => None,
            });

            StreamItem::Value(match previous_age {
                Some(prev) => current_age as i32 - prev as i32,
                None => 0,
            })
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice())).unwrap(); // Age 25

    let mut stream = Box::pin(stream);
    let result = stream.next().await.unwrap().unwrap().unwrap();
    assert_eq!(result, 0); // No previous

    tx.send(Sequenced::new(person_bob())).unwrap(); // Age 30
    let result = stream.next().await.unwrap().unwrap().unwrap();
    assert_eq!(result, 5); // 30 - 25 = 5

    tx.send(Sequenced::new(person_dave())).unwrap(); // Age 28
    let result = stream.next().await.unwrap().unwrap().unwrap();
    assert_eq!(result, -2); // 28 - 30 = -2

    tx.send(Sequenced::new(person_charlie())).unwrap(); // Age 35
    let result = stream.next().await.unwrap().unwrap().unwrap();
    assert_eq!(result, 7); // 35 - 28 = 7
}

#[tokio::test]
async fn test_map_ordered_with_ordered_merge() {
    // Arrange
    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let person_stream = UnboundedReceiverStream::new(person_rx);
    let animal_stream = UnboundedReceiverStream::new(animal_rx);

    let stream = FluxionStream::new(person_stream)
        .ordered_merge(vec![FluxionStream::new(animal_stream)])
        .combine_with_previous()
        .map_ordered(|stream_item| {
            let item = stream_item.unwrap();
            let curr_str = item.current.get().to_string();
            let prev_str = item.previous.map(|p| p.get().to_string());
            StreamItem::Value(format!("Current: {}, Previous: {:?}", curr_str, prev_str))
        });

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice())).unwrap();
    let mut stream = Box::pin(stream);
    let result = stream.next().await.unwrap().unwrap().unwrap();
    assert!(result.contains("Alice"));
    assert!(result.contains("Previous: None"));

    animal_tx.send(Sequenced::new(animal_dog())).unwrap();
    let result = stream.next().await.unwrap().unwrap().unwrap();
    assert!(result.contains("Dog"));
    assert!(result.contains("Alice"));

    person_tx.send(Sequenced::new(person_bob())).unwrap();
    let result = stream.next().await.unwrap().unwrap().unwrap();
    assert!(result.contains("Bob"));
    assert!(result.contains("Dog"));
}

#[tokio::test]
async fn test_map_ordered_with_combine_latest() {
    // Arrange
    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let person_stream = UnboundedReceiverStream::new(person_rx);
    let animal_stream = UnboundedReceiverStream::new(animal_rx);

    let stream = FluxionStream::new(person_stream)
        .combine_latest(vec![animal_stream], COMBINE_FILTER)
        .combine_with_previous()
        .map_ordered(|stream_item| {
            let item = stream_item.unwrap();
            let curr_binding = item.current.unwrap();
            let curr_state = curr_binding.get().get_state();
            let count = curr_state.len();
            StreamItem::Value(format!("Combined {} streams", count))
        });

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice())).unwrap();
    animal_tx.send(Sequenced::new(animal_dog())).unwrap();

    let mut stream = Box::pin(stream);
    let result = stream.next().await.unwrap().unwrap().unwrap();
    assert_eq!(result, "Combined 2 streams");

    person_tx.send(Sequenced::new(person_bob())).unwrap();
    let result = stream.next().await.unwrap().unwrap().unwrap();
    assert_eq!(result, "Combined 2 streams");
}

#[tokio::test]
async fn test_map_ordered_filter_by_age_change() {
    // Arrange
    let (tx, rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let stream = UnboundedReceiverStream::new(rx);

    let stream = FluxionStream::new(stream)
        .combine_with_previous()
        .map_ordered(|stream_item| {
            let item = stream_item.unwrap();
            let current_age = match item.current.get() {
                TestData::Person(p) => p.age,
                _ => return StreamItem::Value(None),
            };
            let previous_age = item.previous.and_then(|prev| match prev.get() {
                TestData::Person(p) => Some(p.age),
                _ => None,
            });

            StreamItem::Value(match previous_age {
                Some(prev) if current_age != prev => {
                    Some(format!("Age changed from {} to {}", prev, current_age))
                }
                _ => None,
            })
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice())).unwrap(); // Age 25
    let mut stream = Box::pin(stream);
    let result = stream.next().await.unwrap().unwrap().unwrap();
    assert_eq!(result, None); // No previous

    tx.send(Sequenced::new(person_bob())).unwrap(); // Age 30
    let result = stream.next().await.unwrap().unwrap().unwrap();
    assert_eq!(result, Some(String::from("Age changed from 25 to 30")));

    tx.send(Sequenced::new(person_charlie())).unwrap(); // Age 35
    let result = stream.next().await.unwrap().unwrap().unwrap();
    assert_eq!(result, Some(String::from("Age changed from 30 to 35")));
}

#[tokio::test]
async fn test_emit_when_with_map_ordered() {
    // Arrange
    let (source_tx, source_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (threshold_tx, threshold_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let source_stream = UnboundedReceiverStream::new(source_rx);
    let threshold_stream = UnboundedReceiverStream::new(threshold_rx);

    let threshold_mapped =
        threshold_stream.map(|seq| StreamItem::Value(WithPrevious::new(None, seq)));

    let filter_fn = |state: &CombinedState<TestData>| -> bool {
        let values = state.get_state();
        let current_age = match &values[0] {
            TestData::Person(p) => p.age,
            _ => return false,
        };
        let threshold_age = match &values[1] {
            TestData::Person(p) => p.age,
            _ => return false,
        };
        current_age >= threshold_age
    };

    let stream = FluxionStream::new(source_stream)
        .combine_with_previous()
        .emit_when(threshold_mapped, filter_fn)
        .map_ordered(|stream_item| {
            let item = stream_item.unwrap().unwrap();
            StreamItem::Value(format!("Passed filter: {}", item.current.get()))
        });

    // Act & Assert
    threshold_tx.send(Sequenced::new(person_bob())).unwrap(); // Threshold 30

    source_tx.send(Sequenced::new(person_alice())).unwrap(); // 25 - below threshold
    let mut stream = Box::pin(stream);
    assert_no_element_emitted(&mut stream, 100).await;

    source_tx.send(Sequenced::new(person_charlie())).unwrap(); // 35 - above threshold
    let result = stream.next().await.unwrap().unwrap().unwrap();
    assert!(result.contains("Charlie"));
    assert!(result.contains("Passed filter"));
}

#[tokio::test]
async fn test_triple_ordered_merge_then_map_ordered() {
    // Arrange
    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let person_stream = UnboundedReceiverStream::new(person_rx);
    let animal_stream = UnboundedReceiverStream::new(animal_rx);
    let plant_stream = UnboundedReceiverStream::new(plant_rx);

    let stream = FluxionStream::new(person_stream)
        .ordered_merge(vec![
            FluxionStream::new(animal_stream),
            FluxionStream::new(plant_stream),
        ])
        .combine_with_previous()
        .map_ordered(|stream_item| {
            let item = stream_item.unwrap();
            let variant = match item.current.unwrap().get() {
                TestData::Person(_) => "Person",
                TestData::Animal(_) => "Animal",
                TestData::Plant(_) => "Plant",
            };
            StreamItem::Value(variant.to_string())
        });

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice())).unwrap();
    animal_tx.send(Sequenced::new(animal_dog())).unwrap();
    plant_tx.send(Sequenced::new(plant_rose())).unwrap();

    let mut stream = Box::pin(stream);
    let result1 = stream.next().await.unwrap().unwrap().unwrap();
    let result2 = stream.next().await.unwrap().unwrap().unwrap();
    let result3 = stream.next().await.unwrap().unwrap().unwrap();

    let results = [result1, result2, result3];
    assert!(results.contains(&String::from("Person")));
    assert!(results.contains(&String::from("Animal")));
    assert!(results.contains(&String::from("Plant")));
}

#[tokio::test]
async fn test_with_latest_from_then_map_ordered() {
    // Arrange - demonstrate computing a derived metric from combined streams
    let (primary_tx, primary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (secondary_tx, secondary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let primary_stream = UnboundedReceiverStream::new(primary_rx);
    let secondary_stream = UnboundedReceiverStream::new(secondary_rx);

    // Custom selector: compute age difference between two people
    let age_difference_selector = |state: &CombinedState<TestData>| -> String {
        let primary_age = match &state.get_state()[0] {
            TestData::Person(p) => p.age as i32,
            _ => 0,
        };
        let secondary_age = match &state.get_state()[1] {
            TestData::Person(p) => p.age as i32,
            _ => 0,
        };
        let diff = primary_age - secondary_age;
        format!("Age difference: {}", diff)
    };

    let mut stream = FluxionStream::new(primary_stream)
        .with_latest_from(secondary_stream, age_difference_selector);

    // Act & Assert
    secondary_tx.send(Sequenced::new(person_alice())).unwrap(); // 25
    primary_tx.send(Sequenced::new(person_bob())).unwrap(); // 30

    let result = stream.next().await.unwrap().unwrap();
    assert_eq!(result.get(), "Age difference: 5"); // 30 - 25

    primary_tx.send(Sequenced::new(person_charlie())).unwrap(); // 35
    let result = stream.next().await.unwrap().unwrap();
    assert_eq!(result.get(), "Age difference: 10"); // 35 - 25

    // Update secondary
    secondary_tx.send(Sequenced::new(person_diane())).unwrap(); // 40
    primary_tx.send(Sequenced::new(person_dave())).unwrap(); // 28

    let result = stream.next().await.unwrap().unwrap();
    assert_eq!(result.get(), "Age difference: -12"); // 28 - 40
}

#[tokio::test]
async fn test_complex_composition_ordered_merge_and_combine_with_previous() {
    // Arrange: ordered_merge -> combine_with_previous
    let (person1_tx, person1_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (person2_tx, person2_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let person1_stream = UnboundedReceiverStream::new(person1_rx);
    let person2_stream = UnboundedReceiverStream::new(person2_rx);

    let stream = FluxionStream::new(person1_stream)
        .ordered_merge(vec![FluxionStream::new(person2_stream)])
        .combine_with_previous();

    // Act & Assert
    person1_tx.send(Sequenced::new(person_alice())).unwrap(); // 25

    let mut stream = Box::pin(stream);
    let result = stream.next().await.unwrap().unwrap();
    assert!(result.previous.is_none());
    assert_eq!(result.current.unwrap().get(), &person_alice());

    person2_tx.send(Sequenced::new(person_bob())).unwrap(); // 30
    let result = stream.next().await.unwrap().unwrap();
    assert_eq!(result.previous.unwrap().unwrap().get(), &person_alice());
    assert_eq!(result.current.unwrap().get(), &person_bob());

    person1_tx.send(Sequenced::new(person_charlie())).unwrap(); // 35
    let result = stream.next().await.unwrap().unwrap();
    assert_eq!(result.previous.unwrap().unwrap().get(), &person_bob());
    assert_eq!(result.current.unwrap().get(), &person_charlie());
}

#[tokio::test]
async fn test_ordered_merge_with_previous_and_map_ordered_name_change() {
    // Arrange - track when name changes between consecutive items
    let (s1_tx, s1_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (s2_tx, s2_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let s1_stream = UnboundedReceiverStream::new(s1_rx);
    let s2_stream = UnboundedReceiverStream::new(s2_rx);

    let stream = FluxionStream::new(s1_stream)
        .ordered_merge(vec![FluxionStream::new(s2_stream)])
        .combine_with_previous()
        .map_ordered(
            |stream_item: StreamItem<WithPrevious<StreamItem<Sequenced<TestData>>>>| async move {
                let item = stream_item.unwrap();
                let current_binding = item.current.unwrap();
                let current_name = match current_binding.get() {
                    TestData::Person(p) => p.name.clone(),
                    _ => "Unknown".to_string(),
                };
                let prev_name = item.previous.as_ref().map(|p| {
                    let prev_binding = p.clone().unwrap();
                    match prev_binding.get() {
                        TestData::Person(p) => p.name.clone(),
                        _ => "Unknown".to_string(),
                    }
                });

                StreamItem::Value(match prev_name {
                    Some(prev) if prev != current_name => {
                        format!("Name changed from {} to {}", prev, current_name)
                    }
                    Some(_) => format!("Same name: {}", current_name),
                    None => format!("First entry: {}", current_name),
                })
            },
        );

    // Act & Assert
    s1_tx.send(Sequenced::new(person_alice())).unwrap();
    let mut stream = Box::pin(stream);
    assert_eq!(
        stream.next().await.unwrap().unwrap().await.unwrap(),
        "First entry: Alice"
    );

    s2_tx.send(Sequenced::new(person_alice())).unwrap();
    assert_eq!(
        stream.next().await.unwrap().unwrap().await.unwrap(),
        "Same name: Alice"
    );

    s1_tx.send(Sequenced::new(person_bob())).unwrap();
    assert_eq!(
        stream.next().await.unwrap().unwrap().await.unwrap(),
        "Name changed from Alice to Bob"
    );
}

#[tokio::test]
async fn test_combine_latest_with_previous_map_ordered_type_count() {
    // Arrange - count different types across combined streams
    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let person_stream = UnboundedReceiverStream::new(person_rx);
    let animal_stream = UnboundedReceiverStream::new(animal_rx);

    let stream = FluxionStream::new(person_stream)
        .combine_latest(vec![animal_stream], COMBINE_FILTER)
        .combine_with_previous()
        .map_ordered(
            |stream_item: StreamItem<
                WithPrevious<StreamItem<OrderedWrapper<CombinedState<TestData>>>>,
            >| async move {
                let item = stream_item.unwrap();
                let state_binding = item.current.unwrap();
                let state = state_binding.get().get_state();
                let person_count = state
                    .iter()
                    .filter(|d| matches!(d, TestData::Person(_)))
                    .count();
                let animal_count = state
                    .iter()
                    .filter(|d| matches!(d, TestData::Animal(_)))
                    .count();
                StreamItem::Value(format!(
                    "Persons: {}, Animals: {}",
                    person_count, animal_count
                ))
            },
        );

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice())).unwrap();
    animal_tx.send(Sequenced::new(animal_dog())).unwrap();

    let mut stream = Box::pin(stream);
    let result = stream.next().await.unwrap().unwrap().await.unwrap();
    assert_eq!(result, "Persons: 1, Animals: 1");

    person_tx.send(Sequenced::new(person_bob())).unwrap();
    let result = stream.next().await.unwrap().unwrap().await.unwrap();
    assert_eq!(result, "Persons: 1, Animals: 1");
}

#[tokio::test]
async fn test_double_ordered_merge_map_ordered() {
    // Arrange - merge two pairs of streams, then merge results
    let (s1_tx, s1_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (s2_tx, s2_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let s1_stream = UnboundedReceiverStream::new(s1_rx);
    let s2_stream = UnboundedReceiverStream::new(s2_rx);

    let stream = FluxionStream::new(s1_stream)
        .ordered_merge(vec![FluxionStream::new(s2_stream)])
        .combine_with_previous()
        .map_ordered(
            |stream_item: StreamItem<WithPrevious<StreamItem<Sequenced<TestData>>>>| async move {
                let item = stream_item.unwrap();
                let type_name = match item.current.unwrap().get() {
                    TestData::Person(_) => "Person",
                    TestData::Animal(_) => "Animal",
                    TestData::Plant(_) => "Plant",
                };
                let count = if item.previous.is_some() { 2 } else { 1 };
                StreamItem::Value(format!("{} (item #{})", type_name, count))
            },
        );

    // Act & Assert
    s1_tx.send(Sequenced::new(person_alice())).unwrap();
    let mut stream = Box::pin(stream);
    assert_eq!(
        stream.next().await.unwrap().unwrap().await.unwrap(),
        "Person (item #1)"
    );

    s2_tx.send(Sequenced::new(animal_dog())).unwrap();
    assert_eq!(
        stream.next().await.unwrap().unwrap().await.unwrap(),
        "Animal (item #2)"
    );

    s1_tx.send(Sequenced::new(plant_rose())).unwrap();
    assert_eq!(
        stream.next().await.unwrap().unwrap().await.unwrap(),
        "Plant (item #2)"
    );
}

#[tokio::test]
async fn test_ordered_merge_map_ordered_data_extraction() {
    // Arrange - extract specific fields from merged data
    let (s1_tx, s1_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (s2_tx, s2_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let s1_stream = UnboundedReceiverStream::new(s1_rx);
    let s2_stream = UnboundedReceiverStream::new(s2_rx);

    let stream = FluxionStream::new(s1_stream)
        .ordered_merge(vec![FluxionStream::new(s2_stream)])
        .combine_with_previous()
        .map_ordered(
            |stream_item: StreamItem<WithPrevious<StreamItem<Sequenced<TestData>>>>| async move {
                let item = stream_item.unwrap();
                StreamItem::Value(match item.current.unwrap().get() {
                    TestData::Person(p) => format!("Person: {}, Age: {}", p.name, p.age),
                    TestData::Animal(a) => format!("Animal: {}, Legs: {}", a.name, a.legs),
                    TestData::Plant(p) => format!("Plant: {}, Height: {}", p.species, p.height),
                })
            },
        );

    // Act & Assert
    s1_tx.send(Sequenced::new(person_alice())).unwrap();
    let mut stream = Box::pin(stream);
    assert_eq!(
        stream.next().await.unwrap().unwrap().await.unwrap(),
        "Person: Alice, Age: 25"
    );

    s2_tx.send(Sequenced::new(animal_dog())).unwrap();
    assert_eq!(
        stream.next().await.unwrap().unwrap().await.unwrap(),
        "Animal: Dog, Legs: 4"
    );

    s1_tx.send(Sequenced::new(plant_rose())).unwrap();
    assert_eq!(
        stream.next().await.unwrap().unwrap().await.unwrap(),
        "Plant: Rose, Height: 15"
    );
}

#[tokio::test]
async fn test_filter_ordered_with_map_ordered() {
    // Arrange - filter for people only, then map to names
    let (tx, rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let stream = UnboundedReceiverStream::new(rx);

    let stream = FluxionStream::new(stream)
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .map_ordered(|stream_item: StreamItem<Sequenced<TestData>>| async move {
            let item = stream_item.unwrap();
            StreamItem::Value(match item.get() {
                TestData::Person(p) => format!("Person: {}", p.name),
                _ => unreachable!(),
            })
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice())).unwrap();
    let mut stream = Box::pin(stream);
    assert_eq!(
        stream.next().await.unwrap().unwrap().await.unwrap(),
        "Person: Alice"
    );

    tx.send(Sequenced::new(animal_dog())).unwrap(); // Filtered out
    tx.send(Sequenced::new(person_bob())).unwrap();
    assert_eq!(
        stream.next().await.unwrap().unwrap().await.unwrap(),
        "Person: Bob"
    );

    tx.send(Sequenced::new(plant_rose())).unwrap(); // Filtered out
    tx.send(Sequenced::new(person_charlie())).unwrap();
    assert_eq!(
        stream.next().await.unwrap().unwrap().await.unwrap(),
        "Person: Charlie"
    );
}

#[tokio::test]
async fn test_filter_ordered_with_combine_with_previous() {
    // Arrange - filter for adults (age > 25), then track changes
    let (tx, rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let stream = UnboundedReceiverStream::new(rx);

    let stream = FluxionStream::new(stream)
        .filter_ordered(|test_data| match test_data {
            TestData::Person(p) => p.age > 25,
            _ => false,
        })
        .combine_with_previous();

    // Act & Assert
    tx.send(Sequenced::new(person_alice())).unwrap(); // 25 - filtered
    tx.send(Sequenced::new(person_bob())).unwrap(); // 30 - kept

    let mut stream = Box::pin(stream);
    let item = stream.next().await.unwrap().unwrap();
    assert!(item.previous.is_none());
    assert_eq!(item.current.unwrap().get(), &person_bob());

    tx.send(Sequenced::new(person_charlie())).unwrap(); // 35 - kept
    let item = stream.next().await.unwrap().unwrap();
    assert_eq!(item.previous.unwrap().unwrap().get(), &person_bob());
    assert_eq!(item.current.unwrap().get(), &person_charlie());

    tx.send(Sequenced::new(person_dave())).unwrap(); // 28 - kept
    let item = stream.next().await.unwrap().unwrap();
    assert_eq!(item.previous.unwrap().unwrap().get(), &person_charlie());
    assert_eq!(item.current.unwrap().get(), &person_dave());
}

#[tokio::test]
async fn test_ordered_merge_with_filter_ordered() {
    // Arrange - merge two streams, then filter for specific types
    let (s1_tx, s1_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (s2_tx, s2_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let s1_stream = UnboundedReceiverStream::new(s1_rx);
    let s2_stream = UnboundedReceiverStream::new(s2_rx);

    let stream = FluxionStream::new(s1_stream)
        .ordered_merge(vec![FluxionStream::new(s2_stream)])
        .filter_ordered(|test_data| !matches!(test_data, TestData::Animal(_))); // Filter out animals

    // Act & Assert
    s1_tx.send(Sequenced::new(person_alice())).unwrap();
    let mut stream = Box::pin(stream);
    assert_eq!(stream.next().await.unwrap().unwrap().get(), &person_alice());

    s2_tx.send(Sequenced::new(animal_dog())).unwrap(); // Filtered out
    s1_tx.send(Sequenced::new(plant_rose())).unwrap();
    assert_eq!(stream.next().await.unwrap().unwrap().get(), &plant_rose());

    s2_tx.send(Sequenced::new(person_bob())).unwrap();
    assert_eq!(stream.next().await.unwrap().unwrap().get(), &person_bob());
}

#[tokio::test]
async fn test_filter_ordered_with_take_latest_when() {
    // Arrange - filter source stream, then apply take_latest_when
    let (source_tx, source_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let source_stream = UnboundedReceiverStream::new(source_rx);
    let filter_stream = UnboundedReceiverStream::new(filter_rx);

    let stream = FluxionStream::new(source_stream)
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .take_latest_when(
            FluxionStream::new(filter_stream).map(StreamItem::Value),
            FILTER,
        );

    // Act & Assert
    source_tx.send(Sequenced::new(person_alice())).unwrap();
    source_tx.send(Sequenced::new(animal_dog())).unwrap(); // Filtered
    source_tx.send(Sequenced::new(person_bob())).unwrap();

    filter_tx.send(Sequenced::new(person_alice())).unwrap(); // Trigger emission

    let mut stream = Box::pin(stream);
    assert_eq!(stream.next().await.unwrap().unwrap().get(), &person_bob());

    source_tx.send(Sequenced::new(person_charlie())).unwrap();
    source_tx.send(Sequenced::new(plant_rose())).unwrap(); // Filtered

    filter_tx.send(Sequenced::new(person_bob())).unwrap(); // Trigger emission
    assert_eq!(
        stream.next().await.unwrap().unwrap().get(),
        &person_charlie()
    );
}

#[tokio::test]
async fn test_filter_ordered_map_ordered_combine_with_previous() {
    // Arrange - complex pipeline: filter -> map -> combine_with_previous
    let (tx, rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let stream = UnboundedReceiverStream::new(rx);

    let stream = FluxionStream::new(stream)
        .filter_ordered(|data| match data {
            TestData::Person(p) => p.age >= 30,
            _ => false,
        })
        .combine_with_previous()
        .map_ordered(|stream_item| async move {
            let item = stream_item.unwrap();
            let current = match item.current.get() {
                TestData::Person(p) => p.name.clone(),
                _ => unreachable!(),
            };
            let previous = item.previous.map(|prev| match prev.get() {
                TestData::Person(p) => p.name.clone(),
                _ => unreachable!(),
            });
            StreamItem::Value(format!("Current: {}, Previous: {:?}", current, previous))
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice())).unwrap(); // 25 - filtered
    tx.send(Sequenced::new(person_bob())).unwrap(); // 30 - kept

    let mut stream = Box::pin(stream);
    assert_eq!(
        stream.next().await.unwrap().unwrap().await.unwrap(),
        "Current: Bob, Previous: None"
    );

    tx.send(Sequenced::new(person_charlie())).unwrap(); // 35 - kept
    assert_eq!(
        stream.next().await.unwrap().unwrap().await.unwrap(),
        "Current: Charlie, Previous: Some(\"Bob\")"
    );

    tx.send(Sequenced::new(person_dave())).unwrap(); // 28 - filtered
    tx.send(Sequenced::new(person_diane())).unwrap(); // 40 - kept
    assert_eq!(
        stream.next().await.unwrap().unwrap().await.unwrap(),
        "Current: Diane, Previous: Some(\"Charlie\")"
    );
}

#[tokio::test]
async fn test_combine_latest_with_filter_ordered() {
    // Arrange - combine latest from multiple streams, then filter
    let (p_tx, p_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (a_tx, a_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let p_stream = UnboundedReceiverStream::new(p_rx);
    let a_stream = UnboundedReceiverStream::new(a_rx);

    let stream = FluxionStream::new(p_stream)
        .combine_latest(vec![a_stream], COMBINE_FILTER)
        .filter_ordered(|wrapper| {
            // Filter: only emit when first item is a person with age > 30
            let state = wrapper.get();
            match &state.get_state()[0] {
                TestData::Person(p) => p.age > 30,
                _ => false,
            }
        });

    // Act & Assert
    p_tx.send(Sequenced::new(person_alice())).unwrap(); // 25
    a_tx.send(Sequenced::new(animal_dog())).unwrap();
    // Combined but filtered out (age <= 30)

    p_tx.send(Sequenced::new(person_charlie())).unwrap(); // 35
    let mut stream = Box::pin(stream);
    let result = stream.next().await.unwrap().unwrap();
    let state = result.get().get_state();
    assert_eq!(state[0], person_charlie());
    assert_eq!(state[1], animal_dog());

    p_tx.send(Sequenced::new(person_diane())).unwrap(); // 40
    let result = stream.next().await.unwrap().unwrap();
    let state = result.get().get_state();
    assert_eq!(state[0], person_diane());
}

#[tokio::test]
async fn test_filter_ordered_with_latest_from() {
    // Arrange - filter primary stream, then combine with custom selector
    let (primary_tx, primary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (secondary_tx, secondary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let primary_stream = UnboundedReceiverStream::new(primary_rx);
    let secondary_stream = UnboundedReceiverStream::new(secondary_rx);

    // Custom selector: extract name from person and combine with secondary info
    let name_combiner = |state: &CombinedState<TestData>| -> String {
        let person_name = match &state.get_state()[0] {
            TestData::Person(p) => p.name.clone(),
            _ => String::from("Unknown"),
        };
        let secondary_info = match &state.get_state()[1] {
            TestData::Animal(a) => format!("with animal {} ({} legs)", a.name, a.legs),
            TestData::Person(p) => format!("with person {} (age {})", p.name, p.age),
            TestData::Plant(p) => format!("with plant {} (height {})", p.species, p.height),
        };
        format!("{} {}", person_name, secondary_info)
    };

    let stream = FluxionStream::new(primary_stream)
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .with_latest_from(
            FluxionStream::new(secondary_stream).map(StreamItem::Value),
            name_combiner,
        );

    // Act & Assert
    secondary_tx.send(Sequenced::new(animal_dog())).unwrap();
    primary_tx.send(Sequenced::new(plant_rose())).unwrap(); // Filtered
    primary_tx.send(Sequenced::new(person_alice())).unwrap(); // Kept

    let mut stream = Box::pin(stream);
    let result = stream.next().await.unwrap().unwrap();
    let combined_name = result.get();
    assert_eq!(combined_name, "Alice with animal Dog (4 legs)");

    // Update secondary to a person
    secondary_tx.send(Sequenced::new(person_bob())).unwrap();
    primary_tx.send(Sequenced::new(person_charlie())).unwrap();

    let result = stream.next().await.unwrap().unwrap();
    let combined_name = result.get();
    assert_eq!(combined_name, "Charlie with person Bob (age 30)");

    // Send animal (filtered) and plant (filtered)
    primary_tx.send(Sequenced::new(animal_dog())).unwrap(); // Filtered
    primary_tx.send(Sequenced::new(plant_rose())).unwrap(); // Filtered

    // Verify no emission yet by checking with a timeout
    assert_no_element_emitted(&mut stream, 100).await;
}

#[tokio::test]
async fn test_with_latest_from_in_middle_of_chain() {
    let (primary_tx, primary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (secondary_tx, secondary_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let primary_stream = UnboundedReceiverStream::new(primary_rx);
    let secondary_stream = UnboundedReceiverStream::new(secondary_rx);

    // Custom selector: combine ages
    let age_combiner = |state: &CombinedState<TestData>| -> u32 {
        let primary_age = match &state.get_state()[0] {
            TestData::Person(p) => p.age,
            _ => 0,
        };
        let secondary_age = match &state.get_state()[1] {
            TestData::Person(p) => p.age,
            _ => 0,
        };
        primary_age + secondary_age
    };

    let mut stream = FluxionStream::new(primary_stream)
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .with_latest_from(
            FluxionStream::new(secondary_stream).map(StreamItem::Value),
            age_combiner,
        )
        .map_ordered(|stream_item: StreamItem<OrderedWrapper<u32>>| async move {
            let age_sum = stream_item.unwrap();
            StreamItem::Value(format!("Combined age: {}", age_sum.get()))
        });

    // Act & Assert
    secondary_tx.send(Sequenced::new(person_alice())).unwrap(); // 25
    primary_tx.send(Sequenced::new(animal_dog())).unwrap(); // Filtered
    primary_tx.send(Sequenced::new(person_bob())).unwrap(); // 30

    let result = stream.next().await.unwrap().unwrap().await;
    assert_eq!(result, StreamItem::Value("Combined age: 55".to_string())); // 30 + 25

    primary_tx.send(Sequenced::new(person_charlie())).unwrap(); // 35
    let result = stream.next().await.unwrap().unwrap().await;
    assert_eq!(result, StreamItem::Value("Combined age: 60".to_string())); // 35 + 25

    // Update secondary
    secondary_tx.send(Sequenced::new(person_diane())).unwrap(); // 40
    primary_tx.send(Sequenced::new(person_dave())).unwrap(); // 28

    let result = stream.next().await.unwrap().unwrap().await;
    assert_eq!(result, StreamItem::Value("Combined age: 68".to_string())); // 28 + 40
}

#[tokio::test]
async fn test_take_while_with_in_middle_of_chain() {
    // Arrange
    let (source_tx, source_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (other_tx, other_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let (predicate_tx, predicate_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();

    let source_stream = UnboundedReceiverStream::new(source_rx);
    let other_stream = UnboundedReceiverStream::new(other_rx);
    let predicate_stream = UnboundedReceiverStream::new(predicate_rx);

    // Chain ordered operations, then take_while_with at the end
    let stream = FluxionStream::new(source_stream)
        .ordered_merge(vec![FluxionStream::new(other_stream)])
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .take_while_with(predicate_stream, |_| true);

    // Act & Assert
    predicate_tx.send(Sequenced::new(person_alice())).unwrap();
    source_tx.send(Sequenced::new(animal_dog())).unwrap(); // Filtered by filter_ordered
    source_tx.send(Sequenced::new(person_bob())).unwrap(); // Kept
    other_tx.send(Sequenced::new(person_charlie())).unwrap(); // Kept

    let mut stream = Box::pin(stream);
    let result1 = stream.next().await.unwrap().unwrap();
    let result2 = stream.next().await.unwrap().unwrap();

    assert_eq!(result1, person_bob());
    assert_eq!(result2, person_charlie());
}
