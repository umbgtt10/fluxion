// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::FluxionStream;
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel,
    test_data::{person_alice, person_bob, person_charlie, person_dave, TestData},
    unwrap_value, Sequenced,
};

#[tokio::test]
async fn test_distinct_until_changed_by_with_filter_composition() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    // Composition: filter -> distinct_until_changed_by (age parity check)
    let mut result = FluxionStream::new(stream)
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .distinct_until_changed_by(|a, b| {
            let age_a = match a {
                TestData::Person(p) => p.age,
                _ => 0,
            };
            let age_b = match b {
                TestData::Person(p) => p.age,
                _ => 0,
            };
            age_a % 2 == age_b % 2 // Same parity
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?; // Age 25 (odd) - emitted
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        person_alice()
    );

    tx.send(Sequenced::new(person_charlie()))?; // Age 35 (odd) - filtered by distinct_by
    assert_no_element_emitted(&mut result, 100).await;

    tx.send(Sequenced::new(person_bob()))?; // Age 30 (even) - emitted (parity changed)
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        person_bob()
    );

    tx.send(Sequenced::new(person_dave()))?; // Age 28 (even) - filtered by distinct_by
    assert_no_element_emitted(&mut result, 100).await;

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_with_map_composition() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    // Composition: map to age -> distinct_until_changed_by (age difference threshold)
    let mut result = FluxionStream::new(stream)
        .map_ordered(|s| {
            let age = match &s.value {
                TestData::Person(p) => p.age,
                _ => 0,
            };
            Sequenced::new(age)
        })
        .distinct_until_changed_by(|a, b| {
            // Consider ages "same" if difference < 3
            (*a as i32 - *b as i32).abs() < 3
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?; // Age 25 - emitted (first)
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        25
    );

    tx.send(Sequenced::new(person_dave()))?; // Age 28, diff=3 >= 3 - emitted
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        28
    );

    tx.send(Sequenced::new(person_bob()))?; // Age 30, diff=2 (from 28) < 3 - filtered
    assert_no_element_emitted(&mut result, 100).await;

    tx.send(Sequenced::new(person_charlie()))?; // Age 35, diff=5 (from 28) >= 3 - emitted
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        35
    );

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_with_combine_latest_composition() -> anyhow::Result<()> {
    // Arrange
    let (stream1_tx, stream1) = test_channel::<Sequenced<TestData>>();
    let (stream2_tx, stream2) = test_channel::<Sequenced<TestData>>();

    // Composition: combine_latest -> map to max age -> distinct_until_changed_by (threshold)
    let mut result = FluxionStream::new(stream1)
        .combine_latest(vec![stream2], |_| true)
        .map_ordered(|state| {
            let ages: Vec<u32> = state
                .values()
                .iter()
                .map(|td| match td {
                    TestData::Person(p) => p.age,
                    _ => 0,
                })
                .collect();
            let max = *ages.iter().max().unwrap();
            Sequenced::new(max)
        })
        .distinct_until_changed_by(|a, b| {
            // Only emit if max age changes by at least 10
            (*a as i32 - *b as i32).abs() < 10
        });

    // Act & Assert
    stream1_tx.send(Sequenced::new(person_bob()))?; // Age 30
    stream2_tx.send(Sequenced::new(person_alice()))?; // Age 25
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        30
    ); // max(30, 25) = 30

    // Small changes - filtered
    stream1_tx.send(Sequenced::new(person_charlie()))?; // Age 35, max=35, diff=5 < 10
    assert_no_element_emitted(&mut result, 100).await;

    drop(stream1_tx);
    drop(stream2_tx);

    Ok(())
}
