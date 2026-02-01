// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::{
    CombineLatestExt, DistinctUntilChangedByExt, FilterOrderedExt, MapOrderedExt,
};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, test_channel, unwrap_stream, unwrap_value},
    sequenced::Sequenced,
    test_data::{person_alice, person_bob, person_charlie, person_dave, TestData},
};

#[tokio::test]
async fn test_distinct_until_changed_by_with_filter_composition() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut result = stream
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
            age_a % 2 == age_b % 2
        });

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        person_alice()
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        person_bob()
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_dave()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_with_map_composition() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut result = stream
        .map_ordered(|s| {
            let age = match &s.value {
                TestData::Person(p) => p.age,
                _ => 0,
            };
            Sequenced::new(age)
        })
        .distinct_until_changed_by(|a, b| (*a as i32 - *b as i32).abs() < 3);

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        25
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_dave()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        28
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        35
    );

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_with_combine_latest_composition() -> anyhow::Result<()> {
    // Arrange
    let (stream1_tx, stream1) = test_channel::<Sequenced<TestData>>();
    let (stream2_tx, stream2) = test_channel::<Sequenced<TestData>>();

    let mut result = stream1
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
        .distinct_until_changed_by(|a, b| (*a as i32 - *b as i32).abs() < 10);

    // Act
    stream1_tx.unbounded_send(Sequenced::new(person_bob()))?;
    stream2_tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        30
    );

    // Act
    stream1_tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    Ok(())
}
