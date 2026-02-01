// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{HasTimestamp, Timestamped};
use fluxion_stream::prelude::*;
use fluxion_stream::CombinedState;
use fluxion_test_utils::helpers::{test_channel, unwrap_stream, unwrap_value};
use fluxion_test_utils::{
    helpers::assert_no_element_emitted,
    sequenced::Sequenced,
    test_data::{
        animal_dog, person_alice, person_bob, person_charlie, person_dave, person_diane,
        plant_rose, TestData,
    },
    test_wrapper::TestWrapper,
};

static FILTER: fn(&TestData) -> bool = |_| true;

#[tokio::test]
async fn test_take_latest_when_with_latest_from_custom_selector() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_rx) = test_channel::<Sequenced<TestData>>();
    let (trigger_tx, trigger_rx) = test_channel::<Sequenced<TestData>>();
    let (secondary_tx, secondary_rx) = test_channel::<Sequenced<TestData>>();

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

    let mut stream = source_rx
        .take_latest_when(trigger_rx, FILTER)
        .with_latest_from(secondary_rx, age_difference_selector);

    // Act
    secondary_tx.unbounded_send(Sequenced::new(person_alice()))?;
    source_tx.unbounded_send(Sequenced::new(person_bob()))?;
    trigger_tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(result.clone().into_inner(), "Age difference: 5");

    // Act
    source_tx.unbounded_send(Sequenced::new(person_charlie()))?;
    trigger_tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(result.clone().into_inner(), "Age difference: 10");

    // Act
    secondary_tx.unbounded_send(Sequenced::new(person_diane()))?;
    source_tx.unbounded_send(Sequenced::new(person_dave()))?;
    trigger_tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(result.clone().into_inner(), "Age difference: -12");

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_with_latest_from() -> anyhow::Result<()> {
    // Arrange
    let (primary_tx, primary_rx) = test_channel::<Sequenced<TestData>>();
    let (secondary_tx, secondary_rx) = test_channel::<Sequenced<TestData>>();

    let name_combiner = |state: &CombinedState<TestData, u64>| -> TestWrapper<String> {
        let person_name = match &state.values()[0] {
            TestData::Person(p) => p.name.clone(),
            _ => String::from("Unknown"),
        };
        let secondary_info = match &state.values()[1] {
            TestData::Animal(a) => format!("with animal {} ({} legs)", a.species, a.legs),
            TestData::Person(p) => format!("with person {} (age {})", p.name, p.age),
            TestData::Plant(p) => format!("with plant {} (height {})", p.species, p.height),
        };
        TestWrapper::new(
            format!("{} {}", person_name, secondary_info),
            state.timestamp(),
        )
    };

    let mut stream = primary_rx
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .with_latest_from(secondary_rx, name_combiner);

    // Act
    secondary_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    primary_tx.unbounded_send(Sequenced::new(plant_rose()))?;
    primary_tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    let combined_name = result.clone().into_inner();
    assert_eq!(combined_name, "Alice with animal Dog (4 legs)");

    // Act
    secondary_tx.unbounded_send(Sequenced::new(person_bob()))?;
    primary_tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    let combined_name = result.clone().into_inner();
    assert_eq!(combined_name, "Charlie with person Bob (age 30)");

    // Act
    primary_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    primary_tx.unbounded_send(Sequenced::new(plant_rose()))?;
    assert_no_element_emitted(&mut stream, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_composition_end_of_chain() -> anyhow::Result<()> {
    let (primary_tx, primary_rx) = test_channel::<Sequenced<TestData>>();
    let (secondary_tx, secondary_rx) = test_channel::<Sequenced<TestData>>();

    let age_combiner = |state: &CombinedState<TestData, u64>| -> Sequenced<String> {
        let primary_age = match &state.values()[0] {
            TestData::Person(p) => p.age,
            _ => 0,
        };
        let secondary_age = match &state.values()[1] {
            TestData::Person(p) => p.age,
            _ => 0,
        };
        Sequenced::new(format!("Combined age: {}", primary_age + secondary_age))
    };

    let mut stream = primary_rx
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .with_latest_from(secondary_rx, age_combiner);

    // Act
    secondary_tx.unbounded_send(Sequenced::new(person_alice()))?;
    primary_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    primary_tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Combined age: 55"
    );

    // Act
    primary_tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Combined age: 60"
    );

    // Act
    secondary_tx.unbounded_send(Sequenced::new(person_diane()))?;
    primary_tx.unbounded_send(Sequenced::new(person_dave()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Combined age: 68"
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_into_with_latest_from() -> anyhow::Result<()> {
    // Arrange
    let (s1_tx, s1_rx) = test_channel::<Sequenced<TestData>>();
    let (s2_tx, s2_rx) = test_channel::<Sequenced<TestData>>();
    let (s3_tx, s3_rx) = test_channel::<Sequenced<TestData>>();

    // Merge s1 and s2, then combine with s3
    let mut stream = s1_rx.ordered_merge(vec![s2_rx]).with_latest_from(
        s3_rx,
        |state: &CombinedState<TestData, u64>| -> Sequenced<String> {
            let values = state.values();
            let primary_name = match &values[0] {
                TestData::Person(p) => p.name.clone(),
                TestData::Animal(a) => a.species.clone(),
                _ => "Unknown".to_string(),
            };
            let secondary_name = match &values[1] {
                TestData::Person(p) => p.name.clone(),
                _ => "Unknown".to_string(),
            };
            Sequenced::new(format!("{} with {}", primary_name, secondary_name))
        },
    );

    // Act
    s3_tx.unbounded_send(Sequenced::new(person_alice()))?;

    s1_tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Bob with Alice"
    );

    // Act
    s2_tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Dog with Alice"
    );

    // Act
    s3_tx.unbounded_send(Sequenced::new(person_charlie()))?;

    s1_tx.unbounded_send(Sequenced::new(person_dave()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Dave with Charlie"
    );

    Ok(())
}
