// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::StreamItem;

use fluxion_stream::{
    CombineLatestExt, CombineWithPreviousExt, DistinctUntilChangedExt, FilterOrderedExt,
    MapOrderedExt,
};
use fluxion_test_utils::{
    helpers::{test_channel, unwrap_stream, unwrap_value},
    sequenced::Sequenced,
    test_data::{person_alice, person_bob, person_charlie, TestData},
};

#[tokio::test]
async fn test_filter_ordered_distinct_until_changed() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut result = stream
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .distinct_until_changed();

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        person_alice()
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        person_bob()
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
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

    let mut result = stream
        .map_ordered(|s| {
            let age = match &s.value {
                TestData::Person(p) => p.age,
                _ => 0,
            };
            Sequenced::new(age)
        })
        .distinct_until_changed();

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?; // Age 25

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        25
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?; // Age 25 - same
    tx.unbounded_send(Sequenced::new(person_bob()))?; // Age 30 - different

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        30
    );

    // Act
    tx.unbounded_send(Sequenced::new(person_bob()))?; // Age 30 - same
    tx.unbounded_send(Sequenced::new(person_bob()))?; // Age 30 - same
    tx.unbounded_send(Sequenced::new(person_charlie()))?; // Age 35 - different

    // Assert
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

    let mut result = stream.combine_with_previous().distinct_until_changed();

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(val) if val.current.value == person_alice() && val.previous.is_none()
    ));

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(val) if val.current.value == person_bob() && matches!(&val.previous, Some(prev) if prev.value == person_alice())
    ));

    // Act
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(person_bob()))?;
    tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
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

    // Act
    // Initial values: Alice (25) + Bob (30) = 55
    stream1_tx.unbounded_send(Sequenced::new(person_alice()))?;
    stream2_tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        55
    );

    // Act
    stream1_tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        60
    );

    // Act
    stream2_tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        65
    );

    // Act
    stream1_tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        70
    );

    // Act
    stream1_tx.unbounded_send(Sequenced::new(person_charlie()))?;
    stream1_tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        60
    );

    drop(stream1_tx);
    drop(stream2_tx);

    Ok(())
}
