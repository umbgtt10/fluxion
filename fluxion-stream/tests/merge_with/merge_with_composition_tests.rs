// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::Timestamped;
use fluxion_stream::{FluxionStream, MergedStream};
use fluxion_test_utils::{
    test_channel,
    test_data::{person_alice, person_bob, person_charlie, person_dave},
    unwrap_stream, Sequenced, TestData,
};

#[tokio::test]
async fn test_merge_with_chaining_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    // Act: Chain map_ordered before merge_with
    // We map each item to 2, then sum them up in merge_with
    let mapped_stream = FluxionStream::new(stream).map_ordered(|_seq| Sequenced::new(2usize));

    let mut result =
        MergedStream::seed::<Sequenced<usize>>(0).merge_with(mapped_stream, |value, state| {
            *state += value;
            *state
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_stream(&mut result, 500).await.into_inner(),
        2,
        "First emission: 0+2 = 2"
    );

    tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_stream(&mut result, 500).await.into_inner(),
        4,
        "Second emission: 2+2 = 4"
    );

    Ok(())
}

#[tokio::test]
async fn test_merge_with_chaining_multiple_operators_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    // Act: Chain map, filter, and map before merge_with
    // 1. Map to name length
    // 2. Filter length > 3
    // 3. Map length + 10
    // 4. Merge (Sum)
    let processed_stream = FluxionStream::new(stream)
        .map_ordered(|seq| {
            let inner = seq.into_inner();
            let len = match inner {
                TestData::Person(p) => p.name.len(),
                TestData::Animal(a) => a.name.len(),
                TestData::Plant(p) => p.species.len(),
            };
            Sequenced::new(len)
        })
        .filter_ordered(|&len| len > 3)
        .map_ordered(|seq| {
            let value = seq.into_inner();
            Sequenced::new(value + 10)
        });

    let mut result =
        MergedStream::seed::<Sequenced<usize>>(0).merge_with(processed_stream, |value, state| {
            *state += value;
            *state
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_stream(&mut result, 500).await.into_inner(),
        15,
        "Alice: 5 > 3, 5+10=15, State=15"
    );

    tx.send(Sequenced::new(person_bob()))?;
    tx.send(Sequenced::new(person_charlie()))?;
    assert_eq!(
        unwrap_stream(&mut result, 500).await.into_inner(),
        32,
        "Charlie: 7 > 3, 7+10=17, State=15+17=32"
    );

    tx.send(Sequenced::new(person_dave()))?;
    assert_eq!(
        unwrap_stream(&mut result, 500).await.into_inner(),
        46,
        "Dave: 4 > 3, 4+10=14, State=32+14=46"
    );

    Ok(())
}
