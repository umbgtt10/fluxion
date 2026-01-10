// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::StreamItem;

use fluxion_stream_multi::{
    CombineLatestExt, CombineWithPreviousExt, DistinctUntilChangedExt, FilterOrderedExt,
    MapOrderedExt,
};
use fluxion_test_utils::{
    helpers::unwrap_stream,
    test_channel,
    test_data::{person_alice, person_bob, person_charlie, TestData},
    unwrap_value, Sequenced,
};

#[tokio::test]
async fn test_filter_ordered_distinct_until_changed() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    // Composition: filter -> distinct_until_changed
    let mut result = stream
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .distinct_until_changed();

    // Act & Assert
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        person_alice()
    );

    tx.unbounded_send(Sequenced::new(person_alice()))?; // Duplicate, filtered by distinct
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        person_bob()
    );

    tx.unbounded_send(Sequenced::new(person_bob()))?; // Duplicate, filtered by distinct
    tx.unbounded_send(Sequenced::new(person_charlie()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        person_charlie()
    );

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_with_map_composition() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    // Composition: map -> distinct_until_changed
    let mut result = stream
        .map_ordered(|s| {
            let age = match &s.value {
                TestData::Person(p) => p.age,
                _ => 0,
            };
            Sequenced::new(age)
        })
        .distinct_until_changed();

    // Act & Assert
    tx.unbounded_send(Sequenced::new(person_alice()))?; // Age 25
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        25
    );

    tx.unbounded_send(Sequenced::new(person_alice()))?; // Age 25 - same
    tx.unbounded_send(Sequenced::new(person_bob()))?; // Age 30 - different
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        30
    );

    tx.unbounded_send(Sequenced::new(person_bob()))?; // Age 30 - same
    tx.unbounded_send(Sequenced::new(person_bob()))?; // Age 30 - same
    tx.unbounded_send(Sequenced::new(person_charlie()))?; // Age 35 - different
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        35
    );

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_with_combine_with_previous_composition() -> anyhow::Result<()>
{
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    // Composition: combine_with_previous -> distinct_until_changed
    let mut result = stream.combine_with_previous().distinct_until_changed();

    // Act & Assert
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(val) if val.current.value == person_alice() && val.previous.is_none()
    ));

    tx.unbounded_send(Sequenced::new(person_alice()))?; // Duplicate, filtered by distinct
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(val) if val.current.value == person_bob() && matches!(&val.previous, Some(prev) if prev.value == person_alice())
    ));

    tx.unbounded_send(Sequenced::new(person_bob()))?; // Duplicate, filtered by distinct
    tx.unbounded_send(Sequenced::new(person_bob()))?; // Duplicate, filtered by distinct
    tx.unbounded_send(Sequenced::new(person_charlie()))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(val) if val.current.value == person_charlie() && matches!(&val.previous, Some(prev) if prev.value == person_bob())
    ));

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_with_distinct_until_changed_composition() -> anyhow::Result<()> {
    // Arrange
    let (stream1_tx, stream1) = test_channel::<Sequenced<TestData>>();
    let (stream2_tx, stream2) = test_channel::<Sequenced<TestData>>();

    // Composition: combine_latest -> map to combined age -> distinct_until_changed
    let mut result = stream1
        .combine_latest(vec![stream2], |_| true)
        .map_ordered(|state| {
            let age1 = match &state.values()[0] {
                TestData::Person(p) => p.age,
                _ => 0,
            };
            let age2 = match &state.values()[1] {
                TestData::Person(p) => p.age,
                _ => 0,
            };
            let sum = age1 + age2;
            Sequenced::new(sum)
        })
        .distinct_until_changed();

    // Act & Assert
    // Initial values: Alice (25) + Bob (30) = 55
    stream1_tx.unbounded_send(Sequenced::new(person_alice()))?;
    stream2_tx.unbounded_send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        55
    );

    // Update stream1: Bob (30) + Bob (30) = 60 (different, should emit)
    stream1_tx.unbounded_send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        60
    );

    // Update stream2: Bob (30) + Charlie (35) = 65 (different, should emit)
    stream2_tx.unbounded_send(Sequenced::new(person_charlie()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        65
    );

    // Update stream1: Charlie (35) + Charlie (35) = 70 (different, should emit)
    stream1_tx.unbounded_send(Sequenced::new(person_charlie()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        70
    );

    // Update stream1: Charlie (35) + Charlie (35) = 70 (same! should NOT emit)
    stream1_tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Update stream1: Alice (25) + Charlie (35) = 60 (different, should emit)
    stream1_tx.unbounded_send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        60
    );

    drop(stream1_tx);
    drop(stream2_tx);

    Ok(())
}
