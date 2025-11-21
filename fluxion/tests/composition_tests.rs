// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::StreamItem;
use fluxion_core::Timestamped;
use fluxion_rx::{CombinedState, FluxionStream};
use fluxion_stream::MergedStream;
use fluxion_stream::WithPrevious;
use fluxion_test_utils::helpers::assert_no_element_emitted;
use fluxion_test_utils::helpers::unwrap_stream;
use fluxion_test_utils::test_channel;
use fluxion_test_utils::test_data::{
    animal_dog, person_alice, person_bob, person_charlie, person_dave, person_diane, plant_rose,
    TestData,
};
use fluxion_test_utils::test_wrapper::TestWrapper;
use fluxion_test_utils::unwrap_value;
use fluxion_test_utils::Sequenced;
use futures::StreamExt;
use tokio::sync::mpsc;

static FILTER: fn(&TestData) -> bool = |_| true;
static COMBINE_FILTER: fn(&CombinedState<TestData, u64>) -> bool = |_| true;
static LATEST_FILTER: fn(&TestData) -> bool = |_| true;

#[tokio::test]
async fn test_fluxion_stream_composition() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut composed = FluxionStream::new(source_stream)
        .take_latest_when(filter_stream, FILTER)
        .combine_with_previous();

    // Act & Assert - send source first, then filter triggers emission
    source_tx.send(Sequenced::new(person_alice()))?;
    filter_tx.send(Sequenced::new(person_alice()))?;

    let element = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert!(
        element.previous.is_none(),
        "First emission should have no previous"
    );
    assert_eq!(&element.current.value, &person_alice());

    // Update source, then trigger with filter
    source_tx.send(Sequenced::new(person_bob()))?;
    filter_tx.send(Sequenced::new(person_bob()))?;
    let element = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert_eq!(&element.previous.unwrap().value, &person_alice());
    assert_eq!(&element.current.value, &person_bob());

    // Update source, then trigger with filter
    source_tx.send(Sequenced::new(person_charlie()))?;
    filter_tx.send(Sequenced::new(person_charlie()))?;
    let element = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert_eq!(&element.previous.unwrap().value, &person_bob());
    assert_eq!(&element.current.value, &person_charlie());

    // Update source, then trigger with filter
    source_tx.send(Sequenced::new(person_dave()))?;
    filter_tx.send(Sequenced::new(person_dave()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert_eq!(&item.previous.unwrap().value, &person_charlie());
    assert_eq!(&item.current.value, &person_dave());

    Ok(())
}

#[tokio::test]
async fn test_fluxion_stream_combine_latest_composition() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_stream) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_stream) = test_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_stream) = test_channel::<Sequenced<TestData>>();

    let mut combined = FluxionStream::new(person_stream)
        .combine_latest(vec![animal_stream, plant_stream], COMBINE_FILTER);

    // Act
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;
    plant_tx.send(Sequenced::new(plant_rose()))?;

    // Assert
    let element = unwrap_value(Some(unwrap_stream(&mut combined, 500).await));
    let inner = element.clone().into_inner();
    let state = inner.values();
    assert_eq!(state.len(), 3);
    assert_eq!(&state[0], &person_alice());
    assert_eq!(&state[1], &animal_dog());
    assert_eq!(&state[2], &plant_rose());

    Ok(())
}

#[tokio::test]
async fn test_fluxion_stream_with_latest_from() -> anyhow::Result<()> {
    // Arrange - Use a custom selector that creates a formatted summary
    let (primary_tx, primary_rx) = test_channel::<Sequenced<TestData>>();
    let (secondary_tx, secondary_rx) = test_channel::<Sequenced<TestData>>();

    let primary_stream = primary_rx;
    let secondary_stream = secondary_rx;

    // Custom selector: create a descriptive string combining both values
    let summary_selector = |state: &CombinedState<TestData, u64>| -> TestWrapper<String> {
        let primary = &state.values()[0];
        let secondary = &state.values()[1];
        let summary = format!("Primary: {:?}, Latest Secondary: {:?}", primary, secondary);
        TestWrapper::new(summary, state.timestamp())
    };

    let mut combined =
        FluxionStream::new(primary_stream).with_latest_from(secondary_stream, summary_selector);

    // Act
    secondary_tx.send(Sequenced::new(person_alice()))?;
    primary_tx.send(Sequenced::new(person_bob()))?;

    // Assert
    let result = unwrap_value(Some(unwrap_stream(&mut combined, 500).await));
    let inner = result.clone().into_inner();
    let summary = inner.value();
    assert!(summary.contains("Bob"));
    assert!(summary.contains("Alice"));

    primary_tx.send(Sequenced::new(person_charlie()))?;
    let result = unwrap_value(Some(unwrap_stream(&mut combined, 500).await));
    let inner = result.clone().into_inner();
    let summary = inner.value();
    assert!(summary.contains("Charlie"));
    assert!(summary.contains("Alice"));

    Ok(())
}

#[tokio::test]
async fn test_fluxion_stream_combine_with_previous() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut stream = FluxionStream::new(stream).combine_with_previous();

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?;

    let element = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert!(element.previous.is_none());
    assert_eq!(&element.current.value, &person_alice());

    tx.send(Sequenced::new(person_bob()))?;
    let element = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&element.previous.unwrap().value, &person_alice());
    assert_eq!(&element.current.value, &person_bob());

    Ok(())
}

#[tokio::test]
async fn test_fluxion_stream_take_while_with() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_rx) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_rx) = test_channel::<Sequenced<bool>>();

    let source_stream = source_rx;
    let filter_stream = filter_rx;

    let mut composed = FluxionStream::new(source_stream).take_while_with(filter_stream, |f| *f);

    // Act & Assert
    filter_tx.send(Sequenced::new(true))?;
    source_tx.send(Sequenced::new(person_alice()))?;
    let val = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert_eq!(&val.value, &person_alice());

    source_tx.send(Sequenced::new(person_bob()))?;
    let val = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert_eq!(&val.value, &person_bob());

    filter_tx.send(Sequenced::new(false))?;
    source_tx.send(Sequenced::new(person_charlie()))?;

    assert_no_element_emitted(&mut composed, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_fluxion_stream_take_latest_when_take_while() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_rx) = test_channel::<Sequenced<TestData>>();
    let (latest_filter_tx, latest_filter_rx) = test_channel::<Sequenced<TestData>>();
    let (while_filter_tx, while_filter_rx) = test_channel::<Sequenced<bool>>();

    let source_stream = source_rx;
    let latest_filter_stream = latest_filter_rx;
    let while_filter_stream = while_filter_rx;

    let mut composed = FluxionStream::new(source_stream)
        .take_latest_when(latest_filter_stream, LATEST_FILTER)
        .take_while_with(while_filter_stream, |f| *f);

    // Act & Assert
    while_filter_tx.send(Sequenced::new(true))?;
    source_tx.send(Sequenced::new(person_alice()))?;
    latest_filter_tx
        .send(Sequenced::new(person_alice()))
        .unwrap();
    let element = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert_eq!(&element.value, &person_alice());

    source_tx.send(Sequenced::new(person_bob()))?;
    latest_filter_tx.send(Sequenced::new(person_bob()))?;
    let element = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert_eq!(&element.value, &person_bob());

    while_filter_tx.send(Sequenced::new(false))?;
    source_tx.send(Sequenced::new(person_charlie()))?;
    latest_filter_tx.send(Sequenced::new(person_charlie()))?;
    assert_no_element_emitted(&mut composed, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_fluxion_stream_combine_latest_and_take_while() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_rx) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_rx) = test_channel::<Sequenced<bool>>();

    let person_stream = person_rx;
    let animal_stream = animal_rx;
    let plant_stream = plant_rx;
    let filter_stream = filter_rx;

    let mut composed = FluxionStream::new(person_stream)
        .combine_latest(vec![animal_stream, plant_stream], COMBINE_FILTER)
        .take_while_with(filter_stream, |f| *f);

    // Act & Assert
    filter_tx.send(Sequenced::new(true))?;
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;
    plant_tx.send(Sequenced::new(plant_rose()))?;
    let result = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    let inner = result.clone().into_inner();
    let state = inner.values();
    assert_eq!(state.len(), 3);
    assert_eq!(&state[0], &person_alice());
    assert_eq!(&state[1], &animal_dog());
    assert_eq!(&state[2], &plant_rose());

    person_tx.send(Sequenced::new(person_bob()))?;
    let result = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    let inner = result.clone().into_inner();
    let state = inner.values();
    assert_eq!(&state[0], &person_bob());
    assert_eq!(&state[1], &animal_dog());
    assert_eq!(&state[2], &plant_rose());

    filter_tx.send(Sequenced::new(false))?;
    person_tx.send(Sequenced::new(person_charlie()))?;
    assert_no_element_emitted(&mut composed, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_fluxion_stream_ordered_merge() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_rx) = test_channel::<Sequenced<TestData>>();

    let person_stream = person_rx;
    let animal_stream = animal_rx;
    let plant_stream = plant_rx;

    let mut merged = FluxionStream::new(person_stream).ordered_merge(vec![
        FluxionStream::new(animal_stream),
        FluxionStream::new(plant_stream),
    ]);

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;
    plant_tx.send(Sequenced::new(plant_rose()))?;

    let result1 = merged.next().await.unwrap();
    assert_eq!(&result1.unwrap().value, &person_alice());

    let result2 = merged.next().await.unwrap();
    assert_eq!(&result2.unwrap().value, &animal_dog());

    let result3 = merged.next().await.unwrap();
    assert_eq!(&result3.unwrap().value, &plant_rose());

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_then_combine_with_previous() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();

    let person_stream = person_rx;
    let animal_stream = animal_rx;

    let mut composed = FluxionStream::new(person_stream)
        .ordered_merge(vec![FluxionStream::new(animal_stream)])
        .combine_with_previous();

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert!(item.previous.is_none());
    assert_eq!(&item.current.value, &person_alice());

    animal_tx.send(Sequenced::new(animal_dog()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert_eq!(&item.previous.unwrap().value, &person_alice());
    assert_eq!(&item.current.value, &animal_dog());

    person_tx.send(Sequenced::new(person_bob()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert_eq!(&item.previous.unwrap().value, &animal_dog());
    assert_eq!(&item.current.value, &person_bob());

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_then_combine_with_previous() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();

    let person_stream = person_rx;
    let animal_stream = animal_rx;

    let mut composed = FluxionStream::new(person_stream)
        .combine_latest(vec![animal_stream], COMBINE_FILTER)
        .combine_with_previous();

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;

    let item = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert!(item.previous.is_none());
    let curr_binding = item.current;
    let inner = curr_binding.clone().into_inner();
    let curr_state = inner.values();
    assert_eq!(&curr_state[0], &person_alice());
    assert_eq!(&curr_state[1], &animal_dog());

    person_tx.send(Sequenced::new(person_bob()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    let prev_seq = item.previous.unwrap();
    let prev_binding = prev_seq;
    let inner = prev_binding.clone().into_inner();
    let prev_state = inner.values();
    assert_eq!(&prev_state[0], &person_alice());
    assert_eq!(&prev_state[1], &animal_dog());

    let curr_binding = item.current;
    let inner = curr_binding.clone().into_inner();
    let curr_state = inner.values();
    assert_eq!(&curr_state[0], &person_bob());
    assert_eq!(&curr_state[1], &animal_dog());

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_then_take_latest_when() -> anyhow::Result<()> {
    // Arrange: Chain combine_latest with take_latest_when
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();
    let (trigger_person_tx, trigger_person_rx) = test_channel::<Sequenced<TestData>>();
    let (trigger_animal_tx, trigger_animal_rx) = test_channel::<Sequenced<TestData>>();

    let person_stream = person_rx;
    let animal_stream = animal_rx;
    let trigger_person_stream = trigger_person_rx;
    let trigger_animal_stream = trigger_animal_rx;

    // Create trigger combined stream first
    let trigger_combined = FluxionStream::new(trigger_person_stream)
        .combine_latest(vec![trigger_animal_stream], COMBINE_FILTER);

    // Chain: combine_latest then take_latest_when
    let mut composed = FluxionStream::new(person_stream)
        .combine_latest(vec![animal_stream], COMBINE_FILTER)
        .take_latest_when(
            trigger_combined,
            |state| state.values().len() >= 2, // Trigger when trigger stream has both values
        );

    // Act & Assert: First populate the source stream
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;

    // No emission yet - waiting for trigger
    assert_no_element_emitted(&mut composed, 100).await;

    // Now trigger emission by populating trigger streams
    trigger_person_tx.send(Sequenced::new(person_bob()))?;
    trigger_animal_tx.send(Sequenced::new(plant_rose()))?;

    // Should emit the latest from source stream
    let result = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    let inner = result.clone().into_inner();
    let values = inner.values();
    assert_eq!(values.len(), 2);
    assert_eq!(&values[0], &person_alice());
    assert_eq!(&values[1], &animal_dog());

    // Update source stream
    person_tx.send(Sequenced::new(person_charlie()))?;

    // Trigger another emission
    trigger_person_tx.send(Sequenced::new(person_dave()))?;

    let result = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    let inner = result.clone().into_inner();
    let values = inner.values();
    assert_eq!(values.len(), 2);
    assert_eq!(&values[0], &person_charlie());
    assert_eq!(&values[1], &animal_dog());

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_then_take_while_with() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_rx) = test_channel::<Sequenced<bool>>();

    let person_stream = person_rx;
    let animal_stream = animal_rx;
    let filter_stream = filter_rx;

    let mut composed = FluxionStream::new(person_stream)
        .ordered_merge(vec![FluxionStream::new(animal_stream)])
        .take_while_with(filter_stream, |f| *f);

    // Act & Assert
    filter_tx.send(Sequenced::new(true))?;
    person_tx.send(Sequenced::new(person_alice()))?;
    {
        let val = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
        assert_eq!(&val.value, &person_alice());
    }

    animal_tx.send(Sequenced::new(animal_dog()))?;
    {
        let val = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
        assert_eq!(&val.value, &animal_dog());
    }

    person_tx.send(Sequenced::new(person_bob()))?;
    {
        let val = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
        assert_eq!(&val.value, &person_bob());
    }

    filter_tx.send(Sequenced::new(false))?;
    person_tx.send(Sequenced::new(person_charlie()))?;
    assert_no_element_emitted(&mut composed, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_triple_composition_combine_latest_take_while_ordered_merge() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_rx) = test_channel::<Sequenced<bool>>();

    let person_stream = person_rx;
    let animal_stream = animal_rx;
    let filter_stream = filter_rx;

    let mut composed = FluxionStream::new(person_stream)
        .combine_latest(vec![animal_stream], COMBINE_FILTER)
        .take_while_with(filter_stream, |f| *f);

    // Act & Assert
    filter_tx.send(Sequenced::new(true))?;
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;
    let result = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert_eq!(result.clone().into_inner().values().len(), 2);
    assert_eq!(result.clone().into_inner().values()[0], person_alice());
    assert_eq!(result.clone().into_inner().values()[1], animal_dog());

    person_tx.send(Sequenced::new(person_bob()))?;
    let result = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert_eq!(result.clone().into_inner().values()[0], person_bob());
    assert_eq!(result.clone().into_inner().values()[1], animal_dog());

    filter_tx.send(Sequenced::new(false))?;
    person_tx.send(Sequenced::new(person_charlie()))?;
    assert_no_element_emitted(&mut composed, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_then_take_latest_when() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_rx) = test_channel::<Sequenced<TestData>>();

    let person_stream = person_rx;
    let animal_stream = animal_rx;
    let filter_stream = filter_rx;

    let mut composed = FluxionStream::new(person_stream)
        .ordered_merge(vec![FluxionStream::new(animal_stream)])
        .take_latest_when(FluxionStream::new(filter_stream), LATEST_FILTER);

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice()))?;
    filter_tx.send(Sequenced::new(person_alice()))?;
    {
        let val = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
        assert_eq!(&val.value, &person_alice());
    }

    animal_tx.send(Sequenced::new(animal_dog()))?;
    filter_tx.send(Sequenced::new(person_alice()))?;
    {
        let val = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
        assert_eq!(&val.value, &animal_dog());
    }

    person_tx.send(Sequenced::new(person_bob()))?;
    filter_tx.send(Sequenced::new(person_alice()))?;
    {
        let val = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
        assert_eq!(&val.value, &person_bob());
    }

    drop(filter_tx);
    person_tx.send(Sequenced::new(person_charlie()))?;
    // After filter stream closes, no more emissions should occur
    assert_no_element_emitted(&mut composed, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_then_ordered_merge() -> anyhow::Result<()> {
    // Arrange
    static LATEST_FILTER_LOCAL: fn(&TestData) -> bool = |_| true;

    let (source_tx, source_rx) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();

    let source_stream = source_rx;
    let filter_stream = filter_rx;
    let animal_stream = animal_rx;

    let mut composed = FluxionStream::new(source_stream)
        .take_latest_when(FluxionStream::new(filter_stream), LATEST_FILTER_LOCAL)
        .ordered_merge(vec![FluxionStream::new(FluxionStream::new(animal_stream))]);

    // Act & Assert
    source_tx.send(Sequenced::new(person_alice()))?;
    filter_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;

    let result1 = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    let result2 = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));

    let values: Vec<_> = vec![&result1.value, &result2.value];
    assert!(values.contains(&&person_alice()));
    assert!(values.contains(&&animal_dog()));

    source_tx.send(Sequenced::new(person_bob()))?;
    filter_tx.send(Sequenced::new(person_bob()))?;
    let result = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert_eq!(&result.value, &person_bob());

    drop(filter_tx);
    source_tx.send(Sequenced::new(person_charlie()))?;
    // After filter stream closes, no more emissions should occur
    assert_no_element_emitted(&mut composed, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_emit_when_composite_with_ordered_merge_and_combine_with_previous(
) -> anyhow::Result<()> {
    // Arrange
    let (person1_tx, person1_rx) = test_channel::<Sequenced<TestData>>();
    let (person2_tx, person2_rx) = test_channel::<Sequenced<TestData>>();
    let (threshold_tx, threshold_rx) = test_channel::<Sequenced<TestData>>();

    let person1_stream = person1_rx;
    let person2_stream = person2_rx;
    let threshold_stream = threshold_rx;

    let filter_fn = |state: &CombinedState<TestData>| -> bool {
        let values = state.values();
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
    let threshold_mapped =
        threshold_stream.map(|seq| StreamItem::Value(WithPrevious::new(None, seq.unwrap())));

    // Chained composition: merge -> combine_with_previous -> emit_when
    let mut output_stream = FluxionStream::new(person1_stream)
        .ordered_merge(vec![FluxionStream::new(person2_stream)])
        .combine_with_previous()
        .emit_when(threshold_mapped, filter_fn);

    // Act: Set threshold to Bob (age 30)
    threshold_tx.send(Sequenced::new(person_bob()))?;

    // Act: Send Alice (25) from stream 1 - below threshold
    person1_tx.send(Sequenced::new(person_alice()))?;

    // Assert: Should not emit (25 < 30)
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Send Charlie (35) from stream 2 - above threshold
    person2_tx.send(Sequenced::new(person_charlie()))?;

    // Assert: Should emit (35 >= 30)
    let emitted = unwrap_value(Some(unwrap_stream(&mut output_stream, 500).await));
    assert_eq!(
        &emitted.current.value,
        &person_charlie(),
        "Expected Charlie (35) to be emitted when >= threshold (30)"
    );

    // Act: Send Dave (28) from stream 1 - below threshold
    person1_tx.send(Sequenced::new(person_dave()))?;

    // Assert: Should not emit (28 < 30)
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Send Diane (40) from stream 2 - above threshold
    person2_tx.send(Sequenced::new(person_diane()))?;

    // Assert: Should emit (40 >= 30)
    let emitted = unwrap_value(Some(unwrap_stream(&mut output_stream, 500).await));
    assert_eq!(
        &emitted.current.value,
        &person_diane(),
        "Expected Diane (40) to be emitted when >= threshold (30)"
    );

    // Act: Lower threshold to Alice (25)
    threshold_tx.send(Sequenced::new(person_alice()))?;

    // Assert: Should re-emit Diane since she still meets the new threshold
    let emitted = unwrap_value(Some(unwrap_stream(&mut output_stream, 500).await));
    assert_eq!(
        &emitted.current.value,
        &person_diane(),
        "Expected Diane (40) to be re-emitted when threshold changes to 25"
    );

    // Act: Send Bob (30) from stream 1 - meets new threshold
    person1_tx.send(Sequenced::new(person_bob()))?;

    // Assert: Should emit (30 >= 25)
    let emitted = unwrap_value(Some(unwrap_stream(&mut output_stream, 500).await));
    assert_eq!(
        &emitted.current.value,
        &person_bob(),
        "Expected Bob (30) to be emitted when >= threshold (25)"
    );

    Ok(())
}

#[tokio::test]
async fn test_map_ordered_basic() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(stream)
        .combine_with_previous()
        .map_ordered(|stream_item| {
            let item = stream_item;
            StreamItem::Value(format!(
                "Previous: {:?}, Current: {}",
                item.previous.map(|p| p.value.to_string()),
                &item.current.value
            ))
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?;

    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap();
    assert_eq!(
        result,
        "Previous: None, Current: Person[name=Alice, age=25]"
    );

    tx.send(Sequenced::new(person_bob()))?;
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap();
    assert_eq!(
        result,
        String::from(
            "Previous: Some(\"Person[name=Alice, age=25]\"), Current: Person[name=Bob, age=30]"
        )
    );

    tx.send(Sequenced::new(person_charlie()))?;
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap();
    assert_eq!(
        result,
        String::from(
            "Previous: Some(\"Person[name=Bob, age=30]\"), Current: Person[name=Charlie, age=35]"
        )
    );

    Ok(())
}

#[tokio::test]
async fn test_map_ordered_to_struct() -> anyhow::Result<()> {
    // Arrange
    #[derive(Debug, PartialEq)]
    struct AgeComparison {
        previous_age: Option<u32>,
        current_age: u32,
        age_increased: bool,
    }

    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(stream)
        .combine_with_previous()
        .map_ordered(|stream_item| {
            let item = stream_item;
            let current_age = match &item.current.value {
                TestData::Person(p) => p.age,
                _ => 0,
            };
            let previous_age = item.previous.and_then(|prev| match &prev.value {
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
    tx.send(Sequenced::new(person_alice()))?; // Age 25
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        AgeComparison {
            previous_age: None,
            current_age: 25,
            age_increased: false,
        }
    );

    tx.send(Sequenced::new(person_bob()))?; // Age 30
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        AgeComparison {
            previous_age: Some(25),
            current_age: 30,
            age_increased: true,
        }
    );

    tx.send(Sequenced::new(person_alice()))?; // Age 25 again
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        AgeComparison {
            previous_age: Some(30),
            current_age: 25,
            age_increased: false,
        }
    );

    tx.send(Sequenced::new(person_charlie()))?; // Age 35
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        AgeComparison {
            previous_age: Some(25),
            current_age: 35,
            age_increased: true,
        }
    );

    Ok(())
}

#[tokio::test]
async fn test_map_ordered_extract_age_difference() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(stream)
        .combine_with_previous()
        .map_ordered(|stream_item| {
            let item = stream_item;
            let current_age = match &item.current.value {
                TestData::Person(p) => p.age,
                _ => 0,
            };
            let previous_age = item.previous.and_then(|prev| match &prev.value {
                TestData::Person(p) => Some(p.age),
                _ => None,
            });

            StreamItem::Value(match previous_age {
                Some(prev) => current_age as i32 - prev as i32,
                None => 0,
            })
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?; // Age 25
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        0
    ); // No previous

    tx.send(Sequenced::new(person_bob()))?; // Age 30
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        5
    ); // 30 - 25 = 5

    tx.send(Sequenced::new(person_dave()))?; // Age 28
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        -2
    ); // 28 - 30 = -2

    tx.send(Sequenced::new(person_charlie()))?; // Age 35
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        7
    ); // 35 - 28 = 7
    Ok(())
}

#[tokio::test]
async fn test_map_ordered_with_ordered_merge() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();

    let person_stream = person_rx;
    let animal_stream = animal_rx;

    let stream = FluxionStream::new(person_stream)
        .ordered_merge(vec![FluxionStream::new(animal_stream)])
        .combine_with_previous()
        .map_ordered(|stream_item| {
            let item = stream_item;
            let curr_str = &item.current.value.to_string();
            let prev_str = item.previous.map(|p| p.value.to_string());
            StreamItem::Value(format!("Current: {}, Previous: {:?}", curr_str, prev_str))
        });

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice()))?;
    let mut stream = Box::pin(stream);
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap();
    assert!(result.contains("Alice"));
    assert!(result.contains("Previous: None"));

    animal_tx.send(Sequenced::new(animal_dog()))?;
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap();
    assert!(result.contains("Dog"));
    assert!(result.contains("Alice"));

    person_tx.send(Sequenced::new(person_bob()))?;
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap();
    assert!(result.contains("Bob"));
    assert!(result.contains("Dog"));

    Ok(())
}

#[tokio::test]
async fn test_map_ordered_with_combine_latest() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();

    let person_stream = person_rx;
    let animal_stream = animal_rx;

    let mut stream = FluxionStream::new(person_stream)
        .combine_latest(vec![animal_stream], COMBINE_FILTER)
        .combine_with_previous()
        .map_ordered(|stream_item| {
            let item = stream_item;
            let curr_binding = item.current;
            let inner = curr_binding.clone().into_inner();
            let curr_state = inner.values();
            let count = curr_state.len();
            StreamItem::Value(format!("Combined {} streams", count))
        });

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        "Combined 2 streams"
    );

    person_tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        "Combined 2 streams"
    );

    Ok(())
}

#[tokio::test]
async fn test_map_ordered_filter_by_age_change() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(stream)
        .combine_with_previous()
        .map_ordered(|stream_item| {
            let item = stream_item;
            let current_age = match &item.current.value {
                TestData::Person(p) => p.age,
                _ => return StreamItem::Value(None),
            };
            let previous_age = item.previous.and_then(|prev| match &prev.value {
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
    tx.send(Sequenced::new(person_alice()))?; // Age 25
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        None
    ); // No previous

    tx.send(Sequenced::new(person_bob()))?; // Age 30
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        Some(String::from("Age changed from 25 to 30"))
    );

    tx.send(Sequenced::new(person_charlie()))?; // Age 35
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap(),
        Some(String::from("Age changed from 30 to 35"))
    );

    Ok(())
}

#[tokio::test]
async fn test_emit_when_with_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_rx) = test_channel::<Sequenced<TestData>>();
    let (threshold_tx, threshold_rx) = test_channel::<Sequenced<TestData>>();

    let source_stream = source_rx;
    let threshold_stream = threshold_rx;

    let threshold_mapped =
        threshold_stream.map(|seq| StreamItem::Value(WithPrevious::new(None, seq.unwrap())));

    let filter_fn = |state: &CombinedState<TestData>| -> bool {
        let values = state.values();
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

    let mut stream = FluxionStream::new(source_stream)
        .combine_with_previous()
        .emit_when(threshold_mapped, filter_fn)
        .map_ordered(|stream_item| {
            let item = stream_item;
            StreamItem::Value(format!("Passed filter: {}", &item.current.value))
        });

    // Act & Assert
    threshold_tx.send(Sequenced::new(person_bob()))?; // Threshold 30
    source_tx.send(Sequenced::new(person_alice()))?; // 25 - below threshold
    assert_no_element_emitted(&mut stream, 100).await;

    source_tx.send(Sequenced::new(person_charlie()))?; // 35 - above threshold
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).unwrap();
    assert!(result.contains("Charlie"));
    assert!(result.contains("Passed filter"));

    Ok(())
}

#[tokio::test]
async fn test_triple_ordered_merge_then_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_rx) = test_channel::<Sequenced<TestData>>();

    let person_stream = person_rx;
    let animal_stream = animal_rx;
    let plant_stream = plant_rx;

    let mut stream = FluxionStream::new(person_stream)
        .ordered_merge(vec![
            FluxionStream::new(animal_stream),
            FluxionStream::new(plant_stream),
        ])
        .combine_with_previous()
        .map_ordered(|stream_item| {
            let item = stream_item;
            let variant = match &item.current.value {
                TestData::Person(_) => "Person",
                TestData::Animal(_) => "Animal",
                TestData::Plant(_) => "Plant",
            };
            StreamItem::Value(variant.to_string())
        });

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;
    plant_tx.send(Sequenced::new(plant_rose()))?;

    let result1 = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    let result2 = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    let result3 = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));

    let results = [result1, result2, result3];
    assert!(results.contains(&StreamItem::Value(String::from("Person"))));
    assert!(results.contains(&StreamItem::Value(String::from("Animal"))));
    assert!(results.contains(&StreamItem::Value(String::from("Plant"))));

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_then_map_ordered() -> anyhow::Result<()> {
    // Arrange - demonstrate computing a derived metric from combined streams
    let (primary_tx, primary_rx) = test_channel::<Sequenced<TestData>>();
    let (secondary_tx, secondary_rx) = test_channel::<Sequenced<TestData>>();

    // Custom selector: compute age difference between two people
    let age_difference_selector = |state: &CombinedState<TestData, u64>| -> TestWrapper<String> {
        let primary_age = match &state.values()[0] {
            TestData::Person(p) => p.age as i32,
            _ => 0,
        };
        let secondary_age = match &state.values()[1] {
            TestData::Person(p) => p.age as i32,
            _ => 0,
        };
        let diff = primary_age - secondary_age;
        TestWrapper::new(format!("Age difference: {}", diff), state.timestamp())
    };

    let mut stream =
        FluxionStream::new(primary_rx).with_latest_from(secondary_rx, age_difference_selector);

    // Act & Assert
    secondary_tx.send(Sequenced::new(person_alice()))?; // 25
    primary_tx.send(Sequenced::new(person_bob()))?; // 30

    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(result.clone().into_inner().value(), "Age difference: 5"); // 30 - 25

    primary_tx.send(Sequenced::new(person_charlie()))?; // 35
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(result.clone().into_inner().value(), "Age difference: 10"); // 35 - 25

    // Update secondary
    secondary_tx.send(Sequenced::new(person_diane()))?; // 40
    primary_tx.send(Sequenced::new(person_dave()))?; // 28

    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(result.clone().into_inner().value(), "Age difference: -12"); // 28 - 40

    Ok(())
}

#[tokio::test]
async fn test_complex_composition_ordered_merge_and_combine_with_previous() -> anyhow::Result<()> {
    // Arrange: ordered_merge -> combine_with_previous
    let (person1_tx, person1_rx) = test_channel::<Sequenced<TestData>>();
    let (person2_tx, person2_rx) = test_channel::<Sequenced<TestData>>();

    let person1_stream = person1_rx;
    let person2_stream = person2_rx;

    let mut stream = FluxionStream::new(person1_stream)
        .ordered_merge(vec![FluxionStream::new(person2_stream)])
        .combine_with_previous();

    // Act & Assert
    person1_tx.send(Sequenced::new(person_alice()))?; // 25

    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert!(result.previous.is_none());
    assert_eq!(&result.current.value, &person_alice());

    person2_tx.send(Sequenced::new(person_bob()))?; // 30
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&result.previous.unwrap().value, &person_alice());
    assert_eq!(&result.current.value, &person_bob());

    person1_tx.send(Sequenced::new(person_charlie()))?; // 35
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&result.previous.unwrap().value, &person_bob());
    assert_eq!(&result.current.value, &person_charlie());

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_with_previous_and_map_ordered_name_change() -> anyhow::Result<()> {
    // Arrange - track when name changes between consecutive items
    let (s1_tx, s1_rx) = test_channel::<Sequenced<TestData>>();
    let (s2_tx, s2_rx) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(s1_rx)
        .ordered_merge(vec![FluxionStream::new(s2_rx)])
        .combine_with_previous()
        .map_ordered(|stream_item| async move {
            let item = stream_item;
            let current_binding = item.current;
            let current_name = match &current_binding.value {
                TestData::Person(p) => p.name.clone(),
                _ => "Unknown".to_string(),
            };
            let prev_name = item.previous.as_ref().map(|p| match &p.value {
                TestData::Person(person) => person.name.clone(),
                _ => "Unknown".to_string(),
            });

            StreamItem::Value(match prev_name {
                Some(prev) if prev != current_name => {
                    format!("Name changed from {} to {}", prev, current_name)
                }
                Some(_) => format!("Same name: {}", current_name),
                None => format!("First entry: {}", current_name),
            })
        });

    // Act & Assert
    s1_tx.send(Sequenced::new(person_alice()))?;

    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "First entry: Alice"
    );

    s2_tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Same name: Alice"
    );

    s1_tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Name changed from Alice to Bob"
    );

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_with_previous_map_ordered_type_count() -> anyhow::Result<()> {
    // Arrange - count different types across combined streams
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();

    let person_stream = person_rx;
    let animal_stream = animal_rx;

    let mut stream = FluxionStream::new(person_stream)
        .combine_latest(vec![animal_stream], COMBINE_FILTER)
        .combine_with_previous()
        .map_ordered(|stream_item| async move {
            let item = stream_item;
            let state_binding = item.current;
            let inner = state_binding.clone().into_inner();
            let state = inner.values();
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
        });

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;

    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Persons: 1, Animals: 1"
    );

    person_tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Persons: 1, Animals: 1"
    );

    Ok(())
}

#[tokio::test]
async fn test_double_ordered_merge_map_ordered() -> anyhow::Result<()> {
    // Arrange - merge two pairs of streams, then merge results
    let (s1_tx, s1_rx) = test_channel::<Sequenced<TestData>>();
    let (s2_tx, s2_rx) = test_channel::<Sequenced<TestData>>();

    let s1_stream = s1_rx;
    let s2_stream = s2_rx;

    let mut stream = FluxionStream::new(s1_stream)
        .ordered_merge(vec![FluxionStream::new(s2_stream)])
        .combine_with_previous()
        .map_ordered(|stream_item| async move {
            let item = stream_item;
            let type_name = match &item.current.value {
                TestData::Person(_) => "Person",
                TestData::Animal(_) => "Animal",
                TestData::Plant(_) => "Plant",
            };
            let count = if item.previous.is_some() { 2 } else { 1 };
            StreamItem::Value(format!("{} (item #{})", type_name, count))
        });

    // Act & Assert
    s1_tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Person (item #1)"
    );

    s2_tx.send(Sequenced::new(animal_dog()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Animal (item #2)"
    );

    s1_tx.send(Sequenced::new(plant_rose()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Plant (item #2)"
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_map_ordered_data_extraction() -> anyhow::Result<()> {
    // Arrange - extract specific fields from merged data
    let (s1_tx, s1_rx) = test_channel::<Sequenced<TestData>>();
    let (s2_tx, s2_rx) = test_channel::<Sequenced<TestData>>();

    let s1_stream = s1_rx;
    let s2_stream = s2_rx;

    let mut stream = FluxionStream::new(s1_stream)
        .ordered_merge(vec![FluxionStream::new(s2_stream)])
        .combine_with_previous()
        .map_ordered(|stream_item| async move {
            let item = stream_item;
            StreamItem::Value(match &item.current.value {
                TestData::Person(p) => format!("Person: {}, Age: {}", p.name, p.age),
                TestData::Animal(a) => format!("Animal: {}, Legs: {}", a.name, a.legs),
                TestData::Plant(p) => format!("Plant: {}, Height: {}", p.species, p.height),
            })
        });

    // Act & Assert
    s1_tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Person: Alice, Age: 25"
    );

    s2_tx.send(Sequenced::new(animal_dog()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Animal: Dog, Legs: 4"
    );

    s1_tx.send(Sequenced::new(plant_rose()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Plant: Rose, Height: 15"
    );
    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_with_map_ordered() -> anyhow::Result<()> {
    // Arrange - filter for people only, then map to names
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(stream)
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .map_ordered(|stream_item| async move {
            let item = stream_item;
            StreamItem::Value(match &item.value {
                TestData::Person(p) => format!("Person: {}", p.name),
                _ => unreachable!(),
            })
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Person: Alice"
    );

    tx.send(Sequenced::new(animal_dog()))?; // Filtered out
    tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Person: Bob"
    );

    tx.send(Sequenced::new(plant_rose()))?; // Filtered out
    tx.send(Sequenced::new(person_charlie()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Person: Charlie"
    );

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_with_combine_with_previous() -> anyhow::Result<()> {
    // Arrange - filter for adults (age > 25), then track changes
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(stream)
        .filter_ordered(|test_data| match test_data {
            TestData::Person(p) => p.age > 25,
            _ => false,
        })
        .combine_with_previous();

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?; // 25 - filtered
    tx.send(Sequenced::new(person_bob()))?; // 30 - kept
    let item = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert!(item.previous.is_none());
    assert_eq!(&item.current.value, &person_bob());

    tx.send(Sequenced::new(person_charlie()))?; // 35 - kept
    let item = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&item.previous.unwrap().value, &person_bob());
    assert_eq!(&item.current.value, &person_charlie());

    tx.send(Sequenced::new(person_dave()))?; // 28 - kept
    let item = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&item.previous.unwrap().value, &person_charlie());
    assert_eq!(&item.current.value, &person_dave());

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_with_filter_ordered() -> anyhow::Result<()> {
    // Arrange - merge two streams, then filter for specific types
    let (s1_tx, s1_rx) = test_channel::<Sequenced<TestData>>();
    let (s2_tx, s2_rx) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(s1_rx)
        .ordered_merge(vec![FluxionStream::new(s2_rx)])
        .filter_ordered(|test_data| !matches!(test_data, TestData::Animal(_))); // Filter out animals

    // Act & Assert
    s1_tx.send(Sequenced::new(person_alice()))?;
    {
        let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
        assert_eq!(&val.value, &person_alice());
    }

    s2_tx.send(Sequenced::new(animal_dog()))?; // Filtered out
    s1_tx.send(Sequenced::new(plant_rose()))?;
    {
        let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
        assert_eq!(&val.value, &plant_rose());
    }

    s2_tx.send(Sequenced::new(person_bob()))?;
    {
        let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
        assert_eq!(&val.value, &person_bob());
    }

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_with_take_latest_when() -> anyhow::Result<()> {
    // Arrange - filter source stream, then apply take_latest_when
    let (source_tx, source_rx) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_rx) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(source_rx)
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .take_latest_when(FluxionStream::new(filter_rx), FILTER);

    // Act & Assert
    source_tx.send(Sequenced::new(person_alice()))?;
    source_tx.send(Sequenced::new(animal_dog()))?; // Filtered
    source_tx.send(Sequenced::new(person_bob()))?;

    filter_tx.send(Sequenced::new(person_alice()))?; // Trigger emission

    {
        let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));

        assert_eq!(&val.value, &person_bob());
    }

    source_tx.send(Sequenced::new(person_charlie()))?;
    source_tx.send(Sequenced::new(plant_rose()))?; // Filtered

    filter_tx.send(Sequenced::new(person_bob()))?; // Trigger emission
    {
        let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
        assert_eq!(&val.value, &person_charlie());
    }

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_map_ordered_combine_with_previous() -> anyhow::Result<()> {
    // Arrange - complex pipeline: filter -> map -> combine_with_previous
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(stream)
        .filter_ordered(|data| match data {
            TestData::Person(p) => p.age >= 30,
            _ => false,
        })
        .combine_with_previous()
        .map_ordered(|stream_item| async move {
            let item = stream_item;
            let current = match &item.current.value {
                TestData::Person(p) => p.name.clone(),
                _ => unreachable!(),
            };
            let previous = item.previous.map(|prev| match &prev.value {
                TestData::Person(p) => p.name.clone(),
                _ => unreachable!(),
            });
            StreamItem::Value(format!("Current: {}, Previous: {:?}", current, previous))
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?; // 25 - filtered
    tx.send(Sequenced::new(person_bob()))?; // 30 - kept

    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Current: Bob, Previous: None"
    );

    tx.send(Sequenced::new(person_charlie()))?; // 35 - kept
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Current: Charlie, Previous: Some(\"Bob\")"
    );

    tx.send(Sequenced::new(person_dave()))?; // 28 - filtered
    tx.send(Sequenced::new(person_diane()))?; // 40 - kept
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Current: Diane, Previous: Some(\"Charlie\")"
    );

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_with_filter_ordered() -> anyhow::Result<()> {
    // Arrange - combine latest from multiple streams, then filter
    let (p_tx, p_rx) = test_channel::<Sequenced<TestData>>();
    let (a_tx, a_rx) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(p_rx)
        .combine_latest(vec![a_rx], COMBINE_FILTER)
        .filter_ordered(|wrapper| {
            // Filter: only emit when first item is a person with age > 30
            let state = &wrapper.clone().into_inner();
            match &state.values()[0] {
                TestData::Person(p) => p.age > 30,
                _ => false,
            }
        });

    // Act & Assert
    p_tx.send(Sequenced::new(person_alice()))?; // 25
    a_tx.send(Sequenced::new(animal_dog()))?;
    // Combined but filtered out (age <= 30)

    p_tx.send(Sequenced::new(person_charlie()))?; // 35
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    let inner = result.clone().into_inner();
    let state = inner.values();
    assert_eq!(&state[0], &person_charlie());
    assert_eq!(&state[1], &animal_dog());

    p_tx.send(Sequenced::new(person_diane()))?; // 40
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    let inner = result.clone().into_inner();
    let state = inner.values();
    assert_eq!(&state[0], &person_diane());

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_with_latest_from() -> anyhow::Result<()> {
    // Arrange - filter primary stream, then combine with custom selector
    let (primary_tx, primary_rx) = test_channel::<Sequenced<TestData>>();
    let (secondary_tx, secondary_rx) = test_channel::<Sequenced<TestData>>();

    // Custom selector: extract name from person and combine with secondary info
    let name_combiner = |state: &CombinedState<TestData, u64>| -> TestWrapper<String> {
        let person_name = match &state.values()[0] {
            TestData::Person(p) => p.name.clone(),
            _ => String::from("Unknown"),
        };
        let secondary_info = match &state.values()[1] {
            TestData::Animal(a) => format!("with animal {} ({} legs)", a.name, a.legs),
            TestData::Person(p) => format!("with person {} (age {})", p.name, p.age),
            TestData::Plant(p) => format!("with plant {} (height {})", p.species, p.height),
        };
        TestWrapper::new(
            format!("{} {}", person_name, secondary_info),
            state.timestamp(),
        )
    };

    let mut stream = FluxionStream::new(primary_rx)
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .with_latest_from(FluxionStream::new(secondary_rx), name_combiner);

    // Act & Assert
    secondary_tx.send(Sequenced::new(animal_dog()))?;
    primary_tx.send(Sequenced::new(plant_rose()))?; // Filtered
    primary_tx.send(Sequenced::new(person_alice()))?; // Kept

    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    let combined_name = result.clone().into_inner();
    assert_eq!(combined_name.value(), "Alice with animal Dog (4 legs)");

    // Update secondary to a person
    secondary_tx.send(Sequenced::new(person_bob()))?;
    primary_tx.send(Sequenced::new(person_charlie()))?;

    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    let combined_name = result.clone().into_inner();
    assert_eq!(combined_name.value(), "Charlie with person Bob (age 30)");

    // Send animal (filtered) and plant (filtered)
    primary_tx.send(Sequenced::new(animal_dog()))?; // Filtered
    primary_tx.send(Sequenced::new(plant_rose()))?; // Filtered

    // Verify no emission yet by checking with a timeout
    assert_no_element_emitted(&mut stream, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_in_middle_of_chain() -> anyhow::Result<()> {
    let (primary_tx, primary_rx) = test_channel::<Sequenced<TestData>>();
    let (secondary_tx, secondary_rx) = test_channel::<Sequenced<TestData>>();

    // Custom selector: combine ages
    let age_combiner = |state: &CombinedState<TestData, u64>| -> TestWrapper<u32> {
        let primary_age = match &state.values()[0] {
            TestData::Person(p) => p.age,
            _ => 0,
        };
        let secondary_age = match &state.values()[1] {
            TestData::Person(p) => p.age,
            _ => 0,
        };
        TestWrapper::new(primary_age + secondary_age, state.timestamp())
    };

    let mut stream = FluxionStream::new(primary_rx)
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .with_latest_from(FluxionStream::new(secondary_rx), age_combiner)
        .map_ordered(|stream_item| async move {
            let inner = stream_item.clone().into_inner();
            let age_sum = inner.value();
            StreamItem::Value(format!("Combined age: {}", age_sum))
        });

    // Act & Assert
    secondary_tx.send(Sequenced::new(person_alice()))?; // 25
    primary_tx.send(Sequenced::new(animal_dog()))?; // Filtered
    primary_tx.send(Sequenced::new(person_bob()))?; // 30

    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).await;
    assert_eq!(result, StreamItem::Value("Combined age: 55".to_string())); // 30 + 25

    primary_tx.send(Sequenced::new(person_charlie()))?; // 35
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).await;
    assert_eq!(result, StreamItem::Value("Combined age: 60".to_string())); // 35 + 25

    // Update secondary
    secondary_tx.send(Sequenced::new(person_diane()))?; // 40
    primary_tx.send(Sequenced::new(person_dave()))?; // 28

    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).await;
    assert_eq!(result, StreamItem::Value("Combined age: 68".to_string())); // 28 + 40

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_in_middle_of_chain() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_rx) = test_channel::<Sequenced<TestData>>();
    let (other_tx, other_rx) = test_channel::<Sequenced<TestData>>();
    let (predicate_tx, predicate_rx) = test_channel::<Sequenced<TestData>>();

    let source_stream = source_rx;
    let other_stream = other_rx;
    let predicate_stream = predicate_rx;

    // Chain ordered operations, then take_while_with at the end
    let mut stream = FluxionStream::new(source_stream)
        .ordered_merge(vec![FluxionStream::new(other_stream)])
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .take_while_with(predicate_stream, |_| true);

    // Act & Assert
    predicate_tx.send(Sequenced::new(person_alice()))?;
    source_tx.send(Sequenced::new(animal_dog()))?; // Filtered by filter_ordered
    source_tx.send(Sequenced::new(person_bob()))?; // Kept
    other_tx.send(Sequenced::new(person_charlie()))?; // Kept
    let result1 = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    let result2 = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));

    assert_eq!(&result1.value, &person_bob());
    assert_eq!(&result2.value, &person_charlie());

    Ok(())
}

#[tokio::test]
async fn test_merge_with_chaining_with_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

    // Act: Chain merge_with with map_ordered that doubles the counter
    let mut result = MergedStream::seed(0)
        .merge_with::<_, _, Sequenced<usize>>(stream, |_item: TestData, state| {
            *state += 1;
            *state
        })
        .into_fluxion_stream()
        .map_ordered(|seq| {
            let value = seq.into_inner();
            Sequenced::new(value * 2)
        });

    // Send first value
    tx.send(Sequenced::new(person_alice()))?;

    // Assert first result: state=1, doubled=2
    let StreamItem::Value(first) = result.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(first.into_inner(), 2, "First emission: (0+1)*2 = 2");

    // Send second value
    tx.send(Sequenced::new(person_bob()))?;

    // Assert second result: state=2, doubled=4
    let StreamItem::Value(second) = result.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(second.into_inner(), 4, "Second emission: (1+1)*2 = 4");

    Ok(())
}

#[tokio::test]
async fn test_merge_with_chaining_with_filter_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

    // Act: Chain merge_with with filter_ordered (only values > 2)
    let mut result = MergedStream::seed(0)
        .merge_with::<_, _, Sequenced<usize>>(stream, |_item: TestData, state| {
            *state += 1;
            *state
        })
        .into_fluxion_stream()
        .filter_ordered(|&value| value > 2);

    // Send first value - state will be 1 (filtered out)
    tx.send(Sequenced::new(person_alice()))?;

    // Send second value - state will be 2 (filtered out)
    tx.send(Sequenced::new(person_bob()))?;

    // Send third value - state will be 3 (kept)
    tx.send(Sequenced::new(person_charlie()))?;

    // Assert: only the third emission passes the filter
    let StreamItem::Value(first_kept) = result.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(
        first_kept.into_inner(),
        3,
        "Third emission passes filter: 3 > 2"
    );

    // Send fourth value - state will be 4 (kept)
    tx.send(Sequenced::new(person_dave()))?;

    // Assert: fourth emission also passes
    let StreamItem::Value(second_kept) = result.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(
        second_kept.into_inner(),
        4,
        "Fourth emission passes filter: 4 > 2"
    );

    Ok(())
}

#[tokio::test]
async fn test_merge_with_chaining_multiple_operators() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

    // Act: Chain merge_with with map, filter, and another map
    let mut result = MergedStream::seed(0)
        .merge_with::<_, _, Sequenced<usize>>(stream, |_item: TestData, state| {
            *state += 1;
            *state
        })
        .into_fluxion_stream()
        .map_ordered(|seq| {
            let value = seq.into_inner();
            Sequenced::new(value * 3)
        })
        .filter_ordered(|&value| value > 6)
        .map_ordered(|seq| {
            let value = seq.into_inner();
            Sequenced::new(value + 10)
        });

    // Send first value - state: 1, *3=3 (filtered out: 3 <= 6)
    tx.send(Sequenced::new(person_alice()))?;

    // Send second value - state: 2, *3=6 (filtered out: 6 <= 6)
    tx.send(Sequenced::new(person_bob()))?;

    // Send third value - state: 3, *3=9, +10=19 (kept: 9 > 6)
    tx.send(Sequenced::new(person_charlie()))?;

    // Assert: first kept value
    let StreamItem::Value(first_kept) = result.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(
        first_kept.into_inner(),
        19,
        "Third emission: 3*3=9, 9+10=19"
    );

    // Send fourth value - state: 4, *3=12, +10=22 (kept: 12 > 6)
    tx.send(Sequenced::new(person_dave()))?;

    // Assert: second kept value
    let StreamItem::Value(second_kept) = result.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(
        second_kept.into_inner(),
        22,
        "Fourth emission: 4*3=12, 12+10=22"
    );

    Ok(())
}
