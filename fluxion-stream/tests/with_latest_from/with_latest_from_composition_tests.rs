// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{HasTimestamp, Timestamped};
use fluxion_stream::{CombinedState, FluxionStream};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel,
    test_data::{
        animal_dog, person_alice, person_bob, person_charlie, person_dave, person_diane,
        plant_rose, TestData,
    },
    test_wrapper::TestWrapper,
    unwrap_value, Sequenced,
};

static FILTER: fn(&TestData) -> bool = |_| true;

#[tokio::test]
async fn test_take_latest_when_with_latest_from_custom_selector() -> anyhow::Result<()> {
    // Arrange - take_latest_when -> with_latest_from composition
    let (source_tx, source_rx) = test_channel::<Sequenced<TestData>>();
    let (trigger_tx, trigger_rx) = test_channel::<Sequenced<TestData>>();
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

    let mut stream = FluxionStream::new(source_rx)
        .take_latest_when(trigger_rx, FILTER)
        .with_latest_from(secondary_rx, age_difference_selector);

    // Act & Assert
    secondary_tx.send(Sequenced::new(person_alice()))?; // 25
    source_tx.send(Sequenced::new(person_bob()))?; // 30
    trigger_tx.send(Sequenced::new(person_alice()))?; // trigger emission

    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(result.clone().into_inner(), "Age difference: 5"); // 30 - 25

    source_tx.send(Sequenced::new(person_charlie()))?; // 35
    trigger_tx.send(Sequenced::new(person_bob()))?; // trigger emission
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(result.clone().into_inner(), "Age difference: 10"); // 35 - 25

    // Update secondary
    secondary_tx.send(Sequenced::new(person_diane()))?; // 40
    source_tx.send(Sequenced::new(person_dave()))?; // 28
    trigger_tx.send(Sequenced::new(person_charlie()))?; // trigger emission

    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(result.clone().into_inner(), "Age difference: -12"); // 28 - 40

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
    assert_eq!(combined_name, "Alice with animal Dog (4 legs)");

    // Update secondary to a person
    secondary_tx.send(Sequenced::new(person_bob()))?;
    primary_tx.send(Sequenced::new(person_charlie()))?;

    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    let combined_name = result.clone().into_inner();
    assert_eq!(combined_name, "Charlie with person Bob (age 30)");

    // Send animal (filtered) and plant (filtered)
    primary_tx.send(Sequenced::new(animal_dog()))?; // Filtered
    primary_tx.send(Sequenced::new(plant_rose()))?; // Filtered

    // Verify no emission yet by checking with a timeout
    assert_no_element_emitted(&mut stream, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_composition_end_of_chain() -> anyhow::Result<()> {
    let (primary_tx, primary_rx) = test_channel::<Sequenced<TestData>>();
    let (secondary_tx, secondary_rx) = test_channel::<Sequenced<TestData>>();

    // Custom selector: combine ages
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

    let mut stream = FluxionStream::new(primary_rx)
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .with_latest_from(FluxionStream::new(secondary_rx), age_combiner);

    // Act & Assert
    secondary_tx.send(Sequenced::new(person_alice()))?; // 25
    primary_tx.send(Sequenced::new(animal_dog()))?; // Filtered
    primary_tx.send(Sequenced::new(person_bob()))?; // 30
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Combined age: 55"
    ); // 30 + 25

    primary_tx.send(Sequenced::new(person_charlie()))?; // 35
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Combined age: 60"
    ); // 35 + 25

    // Update secondary
    secondary_tx.send(Sequenced::new(person_diane()))?; // 40
    primary_tx.send(Sequenced::new(person_dave()))?; // 28

    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Combined age: 68"
    ); // 28 + 40

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_into_with_latest_from() -> anyhow::Result<()> {
    // Arrange
    let (s1_tx, s1_rx) = test_channel::<Sequenced<TestData>>();
    let (s2_tx, s2_rx) = test_channel::<Sequenced<TestData>>();
    let (s3_tx, s3_rx) = test_channel::<Sequenced<TestData>>();

    // Merge s1 and s2, then combine with s3
    let mut stream = FluxionStream::new(s1_rx)
        .ordered_merge(vec![s2_rx])
        .with_latest_from(
            FluxionStream::new(s3_rx),
            |state: &CombinedState<TestData, u64>| -> Sequenced<String> {
                let values = state.values();
                let primary_name = match &values[0] {
                    TestData::Person(p) => p.name.clone(),
                    TestData::Animal(a) => a.name.clone(),
                    _ => "Unknown".to_string(),
                };
                let secondary_name = match &values[1] {
                    TestData::Person(p) => p.name.clone(),
                    _ => "Unknown".to_string(),
                };
                Sequenced::new(format!("{} with {}", primary_name, secondary_name))
            },
        );

    // Act & Assert
    // 1. Set secondary value
    s3_tx.send(Sequenced::new(person_alice()))?;

    // 2. Send to Stream 1 (Primary)
    s1_tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Bob with Alice"
    );

    // 3. Send to Stream 2 (Primary)
    s2_tx.send(Sequenced::new(animal_dog()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Dog with Alice"
    );

    // 4. Update secondary
    s3_tx.send(Sequenced::new(person_charlie()))?;

    // 5. Send to Stream 1
    s1_tx.send(Sequenced::new(person_dave()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Dave with Charlie"
    );

    Ok(())
}
