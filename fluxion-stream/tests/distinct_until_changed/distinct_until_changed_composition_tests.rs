// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::{DistinctUntilChangedExt, FluxionStream};
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
    let mut result = FluxionStream::new(stream)
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .distinct_until_changed();

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, person_alice());

    tx.send(Sequenced::new(person_alice()))?; // Duplicate, filtered by distinct
    tx.send(Sequenced::new(person_bob()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, person_bob());

    tx.send(Sequenced::new(person_bob()))?; // Duplicate, filtered by distinct
    tx.send(Sequenced::new(person_charlie()))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, person_charlie());

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_with_map_composition() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    // Composition: map -> distinct_until_changed
    let mut result = FluxionStream::new(stream)
        .map_ordered(|s| {
            let age = match &s.value {
                TestData::Person(p) => p.age,
                _ => 0,
            };
            Sequenced::new(age)
        })
        .distinct_until_changed();

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?; // Age 25
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 25);

    tx.send(Sequenced::new(person_alice()))?; // Age 25 - same
    tx.send(Sequenced::new(person_bob()))?; // Age 30 - different
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 30);

    tx.send(Sequenced::new(person_bob()))?; // Age 30 - same
    tx.send(Sequenced::new(person_bob()))?; // Age 30 - same
    tx.send(Sequenced::new(person_charlie()))?; // Age 35 - different
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 35);

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_with_combine_with_previous_composition() -> anyhow::Result<()>
{
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    // Composition: combine_with_previous -> distinct_until_changed
    let mut result = FluxionStream::new(stream)
        .combine_with_previous()
        .distinct_until_changed();

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?;
    let combined = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(combined.current.value, person_alice());
    assert_eq!(combined.previous, None);

    tx.send(Sequenced::new(person_alice()))?; // Duplicate, filtered by distinct
    tx.send(Sequenced::new(person_bob()))?;
    let combined = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(combined.current.value, person_bob());
    assert_eq!(combined.previous.as_ref().unwrap().value, person_alice());

    tx.send(Sequenced::new(person_bob()))?; // Duplicate, filtered by distinct
    tx.send(Sequenced::new(person_bob()))?; // Duplicate, filtered by distinct
    tx.send(Sequenced::new(person_charlie()))?;
    let combined = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(combined.current.value, person_charlie());
    assert_eq!(combined.previous.as_ref().unwrap().value, person_bob());

    drop(tx);

    Ok(())
}
