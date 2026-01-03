// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use fluxion_core::Timestamped;
use fluxion_stream::prelude::*;
use fluxion_stream::MergedStream;
use fluxion_test_utils::{
    person::Person,
    test_channel,
    test_data::{person_alice, person_bob, person_charlie, person_dave, TestData},
    unwrap_stream, Sequenced,
};

#[tokio::test]
async fn test_merge_with_chaining_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel::<Sequenced<TestData>>();
    let (tx2, stream2) = test_channel::<Sequenced<TestData>>();

    let mapped_stream1 = stream1.map_ordered(|seq| {
        let data = seq.into_inner();
        match data {
            TestData::Person(mut p) => {
                p.age = 2;
                Sequenced::new(TestData::Person(p))
            }
            other => Sequenced::new(other),
        }
    });

    let mapped_stream2 = stream2.map_ordered(|seq| {
        let data = seq.into_inner();
        match data {
            TestData::Person(mut p) => {
                p.age = 3;
                Sequenced::new(TestData::Person(p))
            }
            other => Sequenced::new(other),
        }
    });

    let mut result = MergedStream::seed::<Sequenced<Person>>(Person::new("State".to_string(), 0))
        .merge_with(mapped_stream1, |item: TestData, state: &mut Person| {
            if let TestData::Person(p) = item {
                state.age += p.age;
            }
            state.clone()
        })
        .merge_with(mapped_stream2, |item: TestData, state: &mut Person| {
            if let TestData::Person(p) = item {
                state.age += p.age;
            }
            state.clone()
        });

    // Act & Assert
    tx1.unbounded_send(Sequenced::new(person_alice()))?;
    assert_eq!(unwrap_stream(&mut result, 500).await.into_inner().age, 2);

    tx2.unbounded_send(Sequenced::new(person_bob()))?;
    assert_eq!(unwrap_stream(&mut result, 500).await.into_inner().age, 5);

    Ok(())
}

#[tokio::test]
async fn test_merge_with_chaining_multiple_operators_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let processed_stream = stream
        .map_ordered(|seq| {
            let data = seq.into_inner();
            match data {
                TestData::Person(mut p) => {
                    p.age = p.name.len() as u32;
                    Sequenced::new(TestData::Person(p))
                }
                other => Sequenced::new(other),
            }
        })
        .filter_ordered(|item| {
            if let TestData::Person(p) = item {
                p.age > 3
            } else {
                true
            }
        })
        .map_ordered(|seq| {
            let data = seq.into_inner();
            match data {
                TestData::Person(mut p) => {
                    p.age += 10;
                    Sequenced::new(TestData::Person(p))
                }
                other => Sequenced::new(other),
            }
        });

    let mut result = MergedStream::seed::<Sequenced<Person>>(Person::new("State".to_string(), 0))
        .merge_with(processed_stream, |item: TestData, state: &mut Person| {
            if let TestData::Person(p) = item {
                state.age += p.age;
            }
            state.clone()
        });

    // Act & Assert
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    assert_eq!(unwrap_stream(&mut result, 500).await.into_inner().age, 15);

    tx.unbounded_send(Sequenced::new(person_bob()))?;

    tx.unbounded_send(Sequenced::new(person_charlie()))?;
    assert_eq!(unwrap_stream(&mut result, 500).await.into_inner().age, 32);

    tx.unbounded_send(Sequenced::new(person_dave()))?;
    assert_eq!(unwrap_stream(&mut result, 500).await.into_inner().age, 46);

    Ok(())
}
