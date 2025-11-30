// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::HasTimestamp;
use fluxion_core::Timestamped;
use fluxion_stream::{CombinedState, FluxionStream};
use fluxion_test_utils::{
    helpers::unwrap_stream,
    test_channel,
    test_data::{person_alice, person_bob, person_charlie, TestData},
    test_wrapper::TestWrapper,
    unwrap_value, Sequenced,
};

#[tokio::test]
async fn test_with_latest_from() -> anyhow::Result<()> {
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
    let summary = result.clone().into_inner();
    assert!(summary.contains("Bob"));
    assert!(summary.contains("Alice"));

    primary_tx.send(Sequenced::new(person_charlie()))?;
    let result = unwrap_value(Some(unwrap_stream(&mut combined, 500).await));
    let summary = result.clone().into_inner();
    assert!(summary.contains("Charlie"));
    assert!(summary.contains("Alice"));

    Ok(())
}
