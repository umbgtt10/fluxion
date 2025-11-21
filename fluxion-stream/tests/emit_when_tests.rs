// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::emit_when::EmitWhenExt;
use fluxion_stream::CombinedState;
use fluxion_test_utils::Sequenced;
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel,
    test_data::{
        animal_ant, animal_bird, animal_cat, animal_dog, animal_spider, person_alice, person_bob,
        person_charlie, person_dave, person_diane, plant_rose, plant_sunflower, TestData,
    },
    unwrap_value,
};
use futures::StreamExt;

#[tokio::test]
async fn test_emit_when_empty_streams() -> anyhow::Result<()> {
    // Arrange
    let filter_fn = |_: &CombinedState<TestData>| -> bool { true };

    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();
    drop(source_tx);
    drop(filter_tx);

    let mut output_stream = source_stream.emit_when(filter_stream, filter_fn);

    // Act & Assert
    let next_item = output_stream.next().await;
    assert!(
        next_item.is_none(),
        "Expected no items from an empty stream with `emit_when`"
    );

    Ok(())
}

#[tokio::test]
async fn test_emit_when_filter_compares_source_and_filter() -> anyhow::Result<()> {
    // Arrange: Emit only when source age > filter legs
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

    let mut output_stream = source_stream.emit_when(filter_stream, filter_fn);

    // Act: Alice age=25, Dog legs=4 => 25 > 4 = true
    source_tx.send(Sequenced::new(person_alice()))?;
    filter_tx.send(Sequenced::new(animal_dog()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut output_stream, 500).await));
    assert_eq!(
        &*emitted_item,
        &person_alice(),
        "Expected Alice to be emitted when age > legs"
    );

    // Act: Update filter to spider (8 legs), still alice age=25 => 25 > 8 = true
    filter_tx.send(Sequenced::new(animal_spider()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut output_stream, 500).await));
    assert_eq!(
        &*emitted_item,
        &person_alice(),
        "Expected Alice to be emitted when age > legs (spider)"
    );

    // Act: Update filter to ant (6 legs), still alice => 25 > 6 = true
    filter_tx.send(Sequenced::new(animal_ant()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut output_stream, 500).await));
    assert_eq!(
        &*emitted_item,
        &person_alice(),
        "Expected Alice to be emitted when age > legs (ant)"
    );

    Ok(())
}

#[tokio::test]
async fn test_emit_when_threshold_comparison() -> anyhow::Result<()> {
    // Arrange: Emit when source value differs from filter by more than threshold
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

    let mut output_stream = source_stream.emit_when(filter_stream, filter_fn);

    // Act: Rose height=15, Sunflower height=180 => diff=165 > 50 = true
    source_tx.send(Sequenced::new(plant_rose()))?;
    filter_tx.send(Sequenced::new(plant_sunflower()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut output_stream, 500).await));
    assert_eq!(
        &*emitted_item,
        &plant_rose(),
        "Expected Rose to be emitted when height difference > 50"
    );

    // Act: Update source to Sunflower => diff=0 < 50 = false
    source_tx.send(Sequenced::new(plant_sunflower()))?;

    // Assert
    assert_no_element_emitted(&mut output_stream, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_emit_when_name_length_comparison() -> anyhow::Result<()> {
    // Arrange: Emit when source name is longer than filter name
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel();

    let filter_fn = |state: &CombinedState<TestData>| -> bool {
        let values = state.values();
        let source_name = match &values[0] {
            TestData::Person(p) => &p.name,
            _ => return false,
        };
        let filter_name = match &values[1] {
            TestData::Animal(a) => &a.name,
            _ => return false,
        };
        source_name.len() > filter_name.len()
    };

    let mut output_stream = source_stream.emit_when(filter_stream, filter_fn);

    // Act: Charlie (7) > Dog (3) = true
    source_tx.send(Sequenced::new(person_charlie()))?;
    filter_tx.send(Sequenced::new(animal_dog()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut output_stream, 500).await));
    assert_eq!(
        &*emitted_item,
        &person_charlie(),
        "Expected Charlie to be emitted when name longer than Dog"
    );

    // Act: Bob (3) > Dog (3) = false
    source_tx.send(Sequenced::new(person_bob()))?;

    // Assert
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Update filter to Cat (3), Bob (3) > Cat (3) = false
    filter_tx.send(Sequenced::new(animal_cat()))?;

    // Assert
    assert_no_element_emitted(&mut output_stream, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_emit_when_multiple_source_updates_with_comparison() -> anyhow::Result<()> {
    // Arrange: Emit when person age is even AND greater than animal legs
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

    let mut output_stream = source_stream.emit_when(filter_stream, filter_fn);

    // Act: Setup filter first - Dog with 4 legs
    filter_tx.send(Sequenced::new(animal_dog()))?;

    // Act: Alice age=25 (odd) > 4 but not even => false
    source_tx.send(Sequenced::new(person_alice()))?;

    // Assert
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Bob age=30 (even) > 4 => true
    source_tx.send(Sequenced::new(person_bob()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut output_stream, 500).await));
    assert_eq!(
        &*emitted_item,
        &person_bob(),
        "Expected Bob (30, even) to be emitted"
    );

    // Act: Dave age=28 (even) > 4 => true
    source_tx.send(Sequenced::new(person_dave()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut output_stream, 500).await));
    assert_eq!(
        &*emitted_item,
        &person_dave(),
        "Expected Dave (28, even) to be emitted"
    );

    // Act: Charlie age=35 (odd) => false
    source_tx.send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_no_element_emitted(&mut output_stream, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_emit_when_stateful_comparison() -> anyhow::Result<()> {
    // Arrange: Emit when source value is strictly greater than filter value
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

    let mut output_stream = source_stream.emit_when(filter_stream, filter_fn);

    // Act: Set threshold to Bob age=30
    filter_tx.send(Sequenced::new(person_bob()))?;

    // Act: Alice age=25 <= 30 = false
    source_tx.send(Sequenced::new(person_alice()))?;

    // Assert
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Charlie age=35 > 30 = true
    source_tx.send(Sequenced::new(person_charlie()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut output_stream, 500).await));
    assert_eq!(
        &*emitted_item,
        &person_charlie(),
        "Expected Charlie to be emitted when age > threshold"
    );

    // Act: Diane age=40 > 30 = true
    source_tx.send(Sequenced::new(person_diane()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut output_stream, 500).await));
    assert_eq!(
        &*emitted_item,
        &person_diane(),
        "Expected Diane to be emitted when age > threshold"
    );

    // Act: Raise threshold to Diane age=40
    filter_tx.send(Sequenced::new(person_diane()))?;

    // Assert: Diane (40) > 40 = false, so no new emission
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Send Bob age=30 as new source => 30 > 40 = false
    source_tx.send(Sequenced::new(person_bob()))?;

    // Assert
    assert_no_element_emitted(&mut output_stream, 100).await;

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

    let mut output_stream = source_stream.emit_when(filter_stream, filter_fn);

    // Act: Establish both values
    source_tx.send(Sequenced::new(person_alice()))?;
    filter_tx.send(Sequenced::new(animal_dog()))?;

    // Assert: Should emit
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut output_stream, 500).await));
    assert_eq!(&*emitted_item, &person_alice());

    // Act: Close filter stream
    drop(filter_tx);

    // Act: Update source
    source_tx.send(Sequenced::new(person_bob()))?;

    // Assert: Should still emit using last known filter value
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut output_stream, 500).await));
    assert_eq!(
        &*emitted_item,
        &person_bob(),
        "Expected source updates to continue after filter stream closes"
    );

    Ok(())
}

#[tokio::test]
async fn test_emit_when_both_values_required() -> anyhow::Result<()> {
    // Arrange: This test highlights that emit_when needs BOTH values
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel();

    let filter_fn = |state: &CombinedState<TestData>| -> bool {
        // Only emit when both are present and satisfy condition
        let values = state.values();
        matches!(&values[0], TestData::Person(_)) && matches!(&values[1], TestData::Animal(_))
    };

    let mut output_stream = source_stream.emit_when(filter_stream, filter_fn);

    // Act: Send only source, no filter yet
    source_tx.send(Sequenced::new(person_alice()))?;

    // Assert: Nothing emitted yet (no filter value)
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Now send filter
    filter_tx.send(Sequenced::new(animal_dog()))?;

    // Assert: Now it should emit
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut output_stream, 500).await));
    assert_eq!(
        &*emitted_item,
        &person_alice(),
        "Expected Alice to be emitted after both values are present"
    );
    Ok(())
}

#[tokio::test]
async fn test_emit_when_filter_stream_updates_trigger_reevaluation() -> anyhow::Result<()> {
    // Arrange: Emit when source age >= filter legs * 10
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

    let mut output_stream = source_stream.emit_when(filter_stream, filter_fn);

    // Act: Alice age=25, Bird legs=2 => 25 >= 20 = true
    source_tx.send(Sequenced::new(person_alice()))?;
    filter_tx.send(Sequenced::new(animal_bird()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut output_stream, 500).await));
    assert_eq!(&*emitted_item, &person_alice());

    // Act: Update filter to Dog legs=4 => 25 >= 40 = false
    filter_tx.send(Sequenced::new(animal_dog()))?;

    // Assert: No emission because condition now false
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Update filter back to Bird => 25 >= 20 = true
    filter_tx.send(Sequenced::new(animal_bird()))?;

    // Assert: Should emit again
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut output_stream, 500).await));
    assert_eq!(&*emitted_item, &person_alice());

    Ok(())
}

#[tokio::test]
async fn test_emit_when_delta_based_filtering() -> anyhow::Result<()> {
    // Arrange: Emit when absolute difference between ages > 10
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

    let mut output_stream = source_stream.emit_when(filter_stream, filter_fn);

    // Act: Alice age=25, Bob age=30 => diff=5 <= 10 = false
    source_tx.send(Sequenced::new(person_alice()))?;
    filter_tx.send(Sequenced::new(person_bob()))?;

    // Assert
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Update source to Diane age=40 => diff=10 (not > 10) = false
    source_tx.send(Sequenced::new(person_diane()))?;

    // Assert
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Update filter to Alice age=25 => diff=15 > 10 = true
    filter_tx.send(Sequenced::new(person_alice()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut output_stream, 500).await));
    assert_eq!(
        &*emitted_item,
        &person_diane(),
        "Expected Diane to be emitted when age difference > 10"
    );

    Ok(())
}

#[tokio::test]
async fn test_emit_when_cross_type_comparison() -> anyhow::Result<()> {
    // Arrange: Emit when person age equals animal legs (silly but valid comparison)
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

    let mut output_stream = source_stream.emit_when(filter_stream, filter_fn);

    // Act: Dog legs=4, Alice age=25 => 4 != 25 = false
    source_tx.send(Sequenced::new(animal_dog()))?;
    filter_tx.send(Sequenced::new(person_alice()))?;

    // Assert
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Update source to Spider legs=8, still Alice => 8 != 25 = false
    source_tx.send(Sequenced::new(animal_spider()))?;

    // Assert
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Update source to Ant legs=6, still Alice => 6 != 25 = false
    source_tx.send(Sequenced::new(animal_ant()))?;

    // Assert
    assert_no_element_emitted(&mut output_stream, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_emit_when_source_stream_closes_after_filter() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel();
    let (filter_tx, filter_stream) = test_channel();

    let filter_fn = |_: &CombinedState<TestData>| -> bool { true };

    let mut output_stream = source_stream.emit_when(filter_stream, filter_fn);

    // Act: Establish both values
    source_tx.send(Sequenced::new(person_alice()))?;
    filter_tx.send(Sequenced::new(animal_dog()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut output_stream, 500).await));
    assert_eq!(&*emitted_item, &person_alice());

    // Act: Close source stream
    drop(source_tx);

    // Act: Update filter
    filter_tx.send(Sequenced::new(animal_cat()))?;

    // Assert: Should emit latest source value
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut output_stream, 500).await));
    assert_eq!(
        &*emitted_item,
        &person_alice(),
        "Expected filter updates to re-emit latest source after source closes"
    );

    // Act: Update filter again
    filter_tx.send(Sequenced::new(animal_spider()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut output_stream, 500).await));
    assert_eq!(&*emitted_item, &person_alice());

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

    let mut output_stream = source_stream.emit_when(filter_stream, filter_fn);

    // Act
    source_tx.send(Sequenced::new(person_alice())).unwrap();
    filter_tx.send(Sequenced::new(animal_dog())).unwrap();

    // Assert: Should panic when filter is evaluated
    let _ = output_stream.next().await;
}

#[tokio::test]
async fn test_emit_when_complex_multi_condition() -> anyhow::Result<()> {
    // Arrange: Complex business logic - emit when:
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

    let mut output_stream = source_stream.emit_when(filter_stream, filter_fn);

    // Act: Diane age=40 (even), Dog legs=4 => 40 % 4 = 0 ✓
    source_tx.send(Sequenced::new(person_diane()))?;
    filter_tx.send(Sequenced::new(animal_dog()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut output_stream, 500).await));
    assert_eq!(&*emitted_item, &person_diane());

    // Act: Bob age=30 (even), Dog legs=4 => 30 % 4 = 2 ✗
    source_tx.send(Sequenced::new(person_bob()))?;

    // Assert
    assert_no_element_emitted(&mut output_stream, 100).await;

    // Act: Update filter to Ant legs=6 => 30 % 6 = 0 ✓
    filter_tx.send(Sequenced::new(animal_ant()))?;

    // Assert
    let emitted_item = unwrap_value(Some(unwrap_stream(&mut output_stream, 500).await));
    assert_eq!(&*emitted_item, &person_bob());

    // Act: Alice age=25 (odd) => fails even check ✗
    source_tx.send(Sequenced::new(person_alice()))?;

    // Assert
    assert_no_element_emitted(&mut output_stream, 100).await;

    Ok(())
}
