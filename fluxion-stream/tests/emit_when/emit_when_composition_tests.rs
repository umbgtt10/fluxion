// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::StreamItem;
use fluxion_stream::prelude::*;
use fluxion_stream::{CombinedState, WithPrevious};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel,
    test_data::{person_alice, person_bob, person_charlie, person_dave, person_diane, TestData},
    unwrap_value, Sequenced,
};
use futures::StreamExt;

#[tokio::test]
async fn test_ordered_merge_combine_with_previous_emit_when() -> anyhow::Result<()> {
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
    let mut output_stream = person1_stream
        .ordered_merge(vec![person2_stream])
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
async fn test_combine_with_previous_emit_when_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_rx) = test_channel::<Sequenced<TestData>>();
    let (threshold_tx, threshold_rx) = test_channel::<Sequenced<TestData>>();

    let source_stream = source_rx;
    let threshold_stream = threshold_rx;

    let filter_fn = |state: &CombinedState<TestData, u64>| -> bool {
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

    // Chain: combine_with_previous -> map_ordered (extract current) -> emit_when
    let mut stream = source_stream
        .combine_with_previous()
        .map_ordered(|wp| wp.current)
        .emit_when(threshold_stream, filter_fn);

    // Act & Assert
    threshold_tx.send(Sequenced::new(person_bob()))?; // Threshold 30
    source_tx.send(Sequenced::new(person_alice()))?; // 25 - below threshold
    assert_no_element_emitted(&mut stream, 100).await;

    source_tx.send(Sequenced::new(person_charlie()))?; // 35 - above threshold
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));

    // Result is TestData (Charlie)
    assert_eq!(result.value, person_charlie());

    Ok(())
}
