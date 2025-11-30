// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::StreamItem;
use fluxion_stream::FluxionStream;
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel,
    test_data::{animal_dog, person_alice, person_bob, person_charlie, TestData},
    unwrap_value, Sequenced,
};

#[tokio::test]
async fn test_take_latest_when_ordered_merge() -> anyhow::Result<()> {
    // Arrange
    static LATEST_FILTER_LOCAL: fn(&TestData) -> bool = |_| true;

    let (source_tx, source_rx) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();

    let source_stream = source_rx;
    let filter_stream = filter_rx;
    let animal_stream = animal_rx;

    let mut composed = FluxionStream::new(source_stream)
        .take_latest_when(FluxionStream::new(filter_stream), LATEST_FILTER_LOCAL)
        .ordered_merge(vec![FluxionStream::new(FluxionStream::new(animal_stream))]);

    // Act & Assert
    source_tx.send(Sequenced::new(person_alice()))?;
    filter_tx.send(Sequenced::new(person_alice()))?;
    animal_tx.send(Sequenced::new(animal_dog()))?;

    let result1 = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    let result2 = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));

    let values: Vec<_> = vec![&result1.value, &result2.value];
    assert!(values.contains(&&person_alice()));
    assert!(values.contains(&&animal_dog()));

    source_tx.send(Sequenced::new(person_bob()))?;
    filter_tx.send(Sequenced::new(person_bob()))?;
    let result = unwrap_value(Some(unwrap_stream(&mut composed, 500).await));
    assert_eq!(&result.value, &person_bob());

    drop(filter_tx);
    source_tx.send(Sequenced::new(person_charlie()))?;
    assert_no_element_emitted(&mut composed, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_combine_with_previous_map_ordered_merge() -> anyhow::Result<()> {
    // Arrange - Apply operators individually then merge
    let (s1_tx, s1_rx) = test_channel::<Sequenced<TestData>>();
    let (s2_tx, s2_rx) = test_channel::<Sequenced<TestData>>();

    // Stream 1: Track previous value
    let stream1 = FluxionStream::new(s1_rx)
        .combine_with_previous()
        .map_ordered(|item| {
            let prev = item
                .previous
                .map(|p| match p.value {
                    TestData::Person(p) => p.name,
                    _ => "Other".to_string(),
                })
                .unwrap_or("None".to_string());

            let curr = match item.current.value {
                TestData::Person(p) => p.name,
                _ => "Other".to_string(),
            };
            Sequenced::new(format!("S1: {} -> {}", prev, curr))
        });

    // Stream 2: Just map
    let stream2 = FluxionStream::new(s2_rx).map_ordered(|item| {
        let curr = match item.value {
            TestData::Animal(a) => a.name,
            _ => "Other".to_string(),
        };
        Sequenced::new(format!("S2: {}", curr))
    });

    let mut merged = stream1.ordered_merge(vec![stream2]);

    // Act & Assert
    s1_tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut merged, 500).await)).value,
        "S1: None -> Alice"
    );

    s2_tx.send(Sequenced::new(animal_dog()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut merged, 500).await)).value,
        "S2: Dog"
    );

    s1_tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut merged, 500).await)).value,
        "S1: Alice -> Bob"
    );

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_map_ordered_merge() -> anyhow::Result<()> {
    // Arrange - Filter then map then merge
    let (s1_tx, s1_rx) = test_channel::<Sequenced<TestData>>();
    let (s2_tx, s2_rx) = test_channel::<Sequenced<TestData>>();

    // Stream 1: Only Persons, extract name
    let stream1 = FluxionStream::new(s1_rx)
        .filter_ordered(|d| matches!(d, TestData::Person(_)))
        .map_ordered(|d| match d.value {
            TestData::Person(p) => Sequenced::new(format!("Person: {}", p.name)),
            _ => Sequenced::new("Should not happen".to_string()),
        });

    // Stream 2: Only Animals, extract name
    let stream2 = FluxionStream::new(s2_rx)
        .filter_ordered(|d| matches!(d, TestData::Animal(_)))
        .map_ordered(|d| match d.value {
            TestData::Animal(a) => Sequenced::new(format!("Animal: {}", a.name)),
            _ => Sequenced::new("Should not happen".to_string()),
        });

    let mut merged = stream1.ordered_merge(vec![stream2]);

    // Act & Assert
    s1_tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut merged, 500).await)).value,
        "Person: Alice"
    );

    s2_tx.send(Sequenced::new(animal_dog()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut merged, 500).await)).value,
        "Animal: Dog"
    );

    // Filtered out
    s1_tx.send(Sequenced::new(animal_dog()))?;
    // Filtered out
    s2_tx.send(Sequenced::new(person_bob()))?;

    // Valid
    s1_tx.send(Sequenced::new(person_charlie()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut merged, 500).await)).value,
        "Person: Charlie"
    );

    Ok(())
}

#[tokio::test]
async fn test_start_with_and_skip_items_into_ordered_merge() -> anyhow::Result<()> {
    // Arrange
    let (s1_tx, s1_rx) = test_channel::<Sequenced<TestData>>();
    let (s2_tx, s2_rx) = test_channel::<Sequenced<TestData>>();

    let initial = vec![StreamItem::Value(Sequenced::new(person_alice()))];

    // Stream 1: Start with Alice
    let stream1 = FluxionStream::new(s1_rx).start_with(initial);

    // Stream 2: Skip 1 item
    let stream2 = FluxionStream::new(s2_rx).skip_items(1);

    // Merge
    let mut merged = stream1.ordered_merge(vec![stream2]);

    // Act & Assert
    // 1. Initial value from Stream 1
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut merged, 500).await)).value,
        person_alice()
    );

    // 2. Send to Stream 2 (Skipped)
    s2_tx.send(Sequenced::new(person_bob()))?;

    // 3. Send to Stream 1
    s1_tx.send(Sequenced::new(person_charlie()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut merged, 500).await)).value,
        person_charlie()
    );

    // 4. Send to Stream 2 (Should be emitted now)
    s2_tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut merged, 500).await)).value,
        person_bob()
    );

    Ok(())
}
