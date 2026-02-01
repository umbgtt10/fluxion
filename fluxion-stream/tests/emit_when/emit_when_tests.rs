// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::emit_when::EmitWhenExt;
use fluxion_stream::CombinedState;
use fluxion_test_utils::{
    helpers::{
        assert_no_element_emitted, assert_stream_ended, test_channel, unwrap_stream, unwrap_value,
    },
    sequenced::Sequenced,
    test_data::{
        animal_ant, animal_bird, animal_cat, animal_dog, animal_spider, person_alice, person_bob,
        person_charlie, person_dave, person_diane, plant_rose, plant_sunflower, TestData,
    },
};

#[tokio::test]
async fn test_emit_when_empty_streams() -> anyhow::Result<()> {
    // Arrange
    let filter_fn = |_: &CombinedState<TestData>| -> bool { true };

    // Act
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();
    drop(source_tx);
    drop(filter_tx);

    let mut result = source_stream.emit_when(filter_stream, filter_fn);

    // Assert
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_emit_when_filter_compares_source_and_filter() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel();

    let filter_fn = |state: &CombinedState<TestData>| -> bool {
        let values = state.values();
        let source_age = match &values[0] {
            TestData::Person(p) => p.age,
            _ => return false,
        };
        let filter_legs = match &values[1] {
            TestData::Animal(a) => a.legs,
            _ => return false,
        };
        source_age > filter_legs
    };

    let mut result = source_stream.emit_when(filter_stream, filter_fn);

    // Act
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(
        &emitted_item.value,
        &person_alice(),
        "Expected Alice to be emitted when age > legs"
    );

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_spider()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(
        &emitted_item.value,
        &person_alice(),
        "Expected Alice to be emitted when age > legs (spider)"
    );

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_ant()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(
        &emitted_item.value,
        &person_alice(),
        "Expected Alice to be emitted when age > legs (ant)"
    );

    Ok(())
}

#[tokio::test]
async fn test_emit_when_threshold_comparison() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel();

    let filter_fn = |state: &CombinedState<TestData>| -> bool {
        let values = state.values();
        let source_height = match &values[0] {
            TestData::Plant(p) => p.height,
            _ => return false,
        };
        let filter_height = match &values[1] {
            TestData::Plant(p) => p.height,
            _ => return false,
        };
        source_height.abs_diff(filter_height) > 50
    };

    let mut result = source_stream.emit_when(filter_stream, filter_fn);

    // Act
    source_tx.unbounded_send(Sequenced::new(plant_rose()))?;
    filter_tx.unbounded_send(Sequenced::new(plant_sunflower()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(
        &emitted_item.value,
        &plant_rose(),
        "Expected Rose to be emitted when height difference > 50"
    );

    // Act
    source_tx.unbounded_send(Sequenced::new(plant_sunflower()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_emit_when_name_length_comparison() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel();

    let filter_fn = |state: &CombinedState<TestData>| -> bool {
        let values = state.values();
        let source_name = match &values[0] {
            TestData::Person(p) => &p.name,
            _ => return false,
        };
        let filter_name = match &values[1] {
            TestData::Animal(a) => &a.species,
            _ => return false,
        };
        source_name.len() > filter_name.len()
    };

    let mut result = source_stream.emit_when(filter_stream, filter_fn);

    // Act
    source_tx.unbounded_send(Sequenced::new(person_charlie()))?;
    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(
        &emitted_item.value,
        &person_charlie(),
        "Expected Charlie to be emitted when name longer than Dog"
    );

    // Act
    source_tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_cat()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_emit_when_multiple_source_updates_with_comparison() -> anyhow::Result<()> {
    // Arrange Emit when person age is even AND greater than animal legs
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel();

    let filter_fn = |state: &CombinedState<TestData>| -> bool {
        let values = state.values();
        let source_age = match &values[0] {
            TestData::Person(p) => p.age,
            _ => return false,
        };
        let filter_legs = match &values[1] {
            TestData::Animal(a) => a.legs,
            _ => return false,
        };
        source_age % 2 == 0 && source_age > filter_legs
    };

    let mut result = source_stream.emit_when(filter_stream, filter_fn);

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Act
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(
        &emitted_item.value,
        &person_bob(),
        "Expected Bob (30, even) to be emitted"
    );

    // Act
    source_tx.unbounded_send(Sequenced::new(person_dave()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(
        &emitted_item.value,
        &person_dave(),
        "Expected Dave (28, even) to be emitted"
    );

    // Act
    source_tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_emit_when_stateful_comparison() -> anyhow::Result<()> {
    // Arrange Emit when source value is strictly greater than filter value
    // This test shows emit_when is useful for "greater than threshold" patterns
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel();

    let filter_fn = |state: &CombinedState<TestData>| -> bool {
        let values = state.values();
        let source_age = match &values[0] {
            TestData::Person(p) => p.age,
            _ => return false,
        };
        let threshold_age = match &values[1] {
            TestData::Person(p) => p.age,
            _ => return false,
        };
        source_age > threshold_age
    };

    let mut result = source_stream.emit_when(filter_stream, filter_fn);

    // Act
    filter_tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Act
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(
        &emitted_item.value,
        &person_charlie(),
        "Expected Charlie to be emitted when age > threshold"
    );

    // Act
    source_tx.unbounded_send(Sequenced::new(person_diane()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(
        &emitted_item.value,
        &person_diane(),
        "Expected Diane to be emitted when age > threshold"
    );

    // Act
    filter_tx.unbounded_send(Sequenced::new(person_diane()))?;

    // Assert: Diane (40) > 40 = false, so no new emission
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_emit_when_filter_stream_closes() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel();

    let filter_fn = |state: &CombinedState<TestData>| -> bool {
        let values = state.values();
        matches!(&values[0], TestData::Person(_)) && matches!(&values[1], TestData::Animal(_))
    };

    let mut result = source_stream.emit_when(filter_stream, filter_fn);

    // Act
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(&emitted_item.value, &person_alice());

    // Act
    drop(filter_tx);

    // Act
    source_tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(
        &emitted_item.value,
        &person_bob(),
        "Expected source updates to continue after filter stream closes"
    );

    Ok(())
}

#[tokio::test]
async fn test_emit_when_both_values_required() -> anyhow::Result<()> {
    // Arrange This test highlights that emit_when needs BOTH values
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel();

    let filter_fn = |state: &CombinedState<TestData>| -> bool {
        // Only emit when both are present and satisfy condition
        let values = state.values();
        matches!(&values[0], TestData::Person(_)) && matches!(&values[1], TestData::Animal(_))
    };

    let mut result = source_stream.emit_when(filter_stream, filter_fn);

    // Act
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert: Nothing emitted yet (no filter value)
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(
        &emitted_item.value,
        &person_alice(),
        "Expected Alice to be emitted after both values are present"
    );
    Ok(())
}

#[tokio::test]
async fn test_emit_when_filter_stream_updates_trigger_reevaluation() -> anyhow::Result<()> {
    // Arrange Emit when source age >= filter legs * 10
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel();

    let filter_fn = |state: &CombinedState<TestData>| -> bool {
        let values = state.values();
        let source_age = match &values[0] {
            TestData::Person(p) => p.age,
            _ => return false,
        };
        let filter_legs = match &values[1] {
            TestData::Animal(a) => a.legs,
            _ => return false,
        };
        source_age >= filter_legs * 10
    };

    let mut result = source_stream.emit_when(filter_stream, filter_fn);

    // Act
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    filter_tx.unbounded_send(Sequenced::new(animal_bird()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(&emitted_item.value, &person_alice());

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_bird()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(&emitted_item.value, &person_alice());

    Ok(())
}

#[tokio::test]
async fn test_emit_when_delta_based_filtering() -> anyhow::Result<()> {
    // Arrange Emit when absolute difference between ages > 10
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel();

    let filter_fn = |state: &CombinedState<TestData>| -> bool {
        let values = state.values();
        let source_age = match &values[0] {
            TestData::Person(p) => p.age,
            _ => return false,
        };
        let filter_age = match &values[1] {
            TestData::Person(p) => p.age,
            _ => return false,
        };
        source_age.abs_diff(filter_age) > 10
    };

    let mut result = source_stream.emit_when(filter_stream, filter_fn);

    // Act
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    filter_tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(Sequenced::new(person_diane()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(
        &emitted_item.value,
        &person_diane(),
        "Expected Diane to be emitted when age difference > 10"
    );

    Ok(())
}

#[tokio::test]
async fn test_emit_when_cross_type_comparison() -> anyhow::Result<()> {
    // Arrange Emit when person age equals animal legs (silly but valid comparison)
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel();

    let filter_fn = |state: &CombinedState<TestData>| -> bool {
        let values = state.values();
        let source_legs = match &values[0] {
            TestData::Animal(a) => a.legs,
            _ => return false,
        };
        let filter_age = match &values[1] {
            TestData::Person(p) => p.age,
            _ => return false,
        };
        source_legs == filter_age
    };

    let mut result = source_stream.emit_when(filter_stream, filter_fn);

    // Act
    source_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    filter_tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(Sequenced::new(animal_spider()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(Sequenced::new(animal_ant()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_emit_when_source_stream_closes_after_filter() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel();

    let filter_fn = |_: &CombinedState<TestData>| -> bool { true };

    let mut result = source_stream.emit_when(filter_stream, filter_fn);

    // Act
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(&emitted_item.value, &person_alice());

    // Act
    drop(source_tx);

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_cat()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(
        &emitted_item.value,
        &person_alice(),
        "Expected filter updates to re-emit latest source after source closes"
    );

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_spider()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(&emitted_item.value, &person_alice());

    Ok(())
}

#[tokio::test]
#[should_panic(expected = "Filter function must not panic!")]
async fn test_emit_when_filter_panics() {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel();

    let filter_fn =
        |_: &CombinedState<TestData>| -> bool { panic!("Filter function must not panic!") };

    let mut result = source_stream.emit_when(filter_stream, filter_fn);

    // Act
    source_tx
        .unbounded_send(Sequenced::new(person_alice()))
        .unwrap();
    filter_tx
        .unbounded_send(Sequenced::new(animal_dog()))
        .unwrap();

    // Assert: Should panic when filter is evaluated
    let _ = unwrap_stream(&mut result, 100).await;
}

#[tokio::test]
async fn test_emit_when_complex_multi_condition() -> anyhow::Result<()> {
    // Arrange Complex business logic - emit when:
    // - Source is a Person with even age
    // - Filter is an Animal with legs > 2
    // - Person age is divisible by animal legs
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel();

    let filter_fn = |state: &CombinedState<TestData>| -> bool {
        let values = state.values();
        let source_age = match &values[0] {
            TestData::Person(p) => p.age,
            _ => return false,
        };
        let filter_legs = match &values[1] {
            TestData::Animal(a) => a.legs,
            _ => return false,
        };

        // All conditions must be true
        source_age % 2 == 0 && filter_legs > 2 && source_age % filter_legs == 0
    };

    let mut result = source_stream.emit_when(filter_stream, filter_fn);

    // Act
    source_tx.unbounded_send(Sequenced::new(person_diane()))?;
    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(&emitted_item.value, &person_diane());

    // Act
    source_tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_ant()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(&emitted_item.value, &person_bob());

    // Act
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    Ok(())
}
