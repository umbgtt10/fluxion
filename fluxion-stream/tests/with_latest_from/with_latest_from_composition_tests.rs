// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{HasTimestamp, Timestamped};
use fluxion_stream::{CombinedState, FluxionStream};
use fluxion_test_utils::{
    helpers::unwrap_stream,
    test_channel,
    test_data::{person_alice, person_bob, person_charlie, person_dave, person_diane, TestData},
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
