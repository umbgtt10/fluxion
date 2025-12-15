// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::{CombineLatestExt, MapOrderedExt, OrderedStreamExt, ScanOrderedExt};
use fluxion_test_utils::{
    helpers::unwrap_stream,
    test_channel,
    test_data::{animal_dog, person_alice, person_bob, person_charlie, TestData},
    unwrap_value, Sequenced,
};

#[tokio::test]
async fn test_scan_ordered_chained() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    // First scan: running sum of ages
    // Second scan: count of emissions
    let mut result = stream
        .scan_ordered::<Sequenced<i32>, _, _>(0, |sum: &mut i32, value: &TestData| {
            if let TestData::Person(p) = value {
                *sum += p.age as i32;
            }
            *sum
        })
        .scan_ordered(0, |count: &mut i32, _sum: &i32| {
            *count += 1;
            *count
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?; // age=25, sum=25, count=1
    assert_eq!(
        unwrap_value(Some(
            unwrap_stream::<Sequenced<i32>, _>(&mut result, 500).await,
        ))
        .value,
        1
    );

    tx.send(Sequenced::new(person_bob()))?; // age=28, sum=53, count=2
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        2
    );

    tx.send(Sequenced::new(person_charlie()))?; // age=35, sum=88, count=3
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        3
    );

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_scan_ordered_composed_with_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    // Act: Chain map_ordered before scan_ordered
    // 1. Map each item to value 10
    // 2. Scan (accumulate) the values
    let mut result = stream.map_ordered(|_item| Sequenced::new(10)).scan_ordered(
        0,
        |sum: &mut i32, value: &i32| {
            *sum += value;
            *sum
        },
    );

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(
            unwrap_stream::<Sequenced<i32>, _>(&mut result, 500).await
        ))
        .value,
        10
    );

    tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        20
    );

    tx.send(Sequenced::new(person_charlie()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 500).await)).value,
        30
    );

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_scan_ordered_name_change() -> anyhow::Result<()> {
    // Arrange - track when name changes between consecutive items using scan_ordered
    let (s1_tx, s1_rx) = test_channel::<Sequenced<TestData>>();
    let (s2_tx, s2_rx) = test_channel::<Sequenced<TestData>>();

    let mut stream = s1_rx.ordered_merge(vec![s2_rx]).scan_ordered(
        None::<String>,
        |last_name: &mut Option<String>, item: &TestData| {
            let current_name = match item {
                TestData::Person(p) => p.name.clone(),
                _ => "Unknown".to_string(),
            };
            let result = match last_name {
                Some(prev) if prev != &current_name => {
                    format!("Name changed from {} to {}", prev, current_name)
                }
                Some(_) => format!("Same name: {}", current_name),
                None => format!("First entry: {}", current_name),
            };
            *last_name = Some(current_name);
            result
        },
    );

    // Act & Assert
    s1_tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value::<Sequenced<String>>(Some(unwrap_stream(&mut stream, 500).await)).value,
        "First entry: Alice"
    );

    s2_tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value::<Sequenced<String>>(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Same name: Alice"
    );

    s1_tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value::<Sequenced<String>>(Some(unwrap_stream(&mut stream, 500).await)).value,
        "Name changed from Alice to Bob"
    );

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_scan_ordered_total_age() -> anyhow::Result<()> {
    // Arrange - maintain a running total of ages from combined streams
    let (person_tx, person_rx) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_rx) = test_channel::<Sequenced<TestData>>();

    let person_stream = person_rx;
    let animal_stream = animal_rx;

    let combine_filter = |_: &fluxion_stream::CombinedState<TestData, u64>| true;

    let mut stream = person_stream
        .combine_latest(vec![animal_stream], combine_filter)
        .scan_ordered(
            0u32,
            |total_age: &mut u32, state: &fluxion_stream::CombinedState<TestData, u64>| {
                let values = state.values();
                let current_sum: u32 = values
                    .iter()
                    .map(|v| match v {
                        TestData::Person(p) => p.age,
                        _ => 0,
                    })
                    .sum();
                *total_age += current_sum;
                *total_age
            },
        );

    // Act & Assert
    person_tx.send(Sequenced::new(person_alice()))?; // 25
    animal_tx.send(Sequenced::new(animal_dog()))?; // 0 (animal)
    assert_eq!(
        unwrap_value::<Sequenced<u32>>(Some(unwrap_stream(&mut stream, 500).await)).value,
        25
    );

    person_tx.send(Sequenced::new(person_bob()))?; // 30
    assert_eq!(
        unwrap_value::<Sequenced<u32>>(Some(unwrap_stream(&mut stream, 500).await)).value,
        55
    );

    person_tx.send(Sequenced::new(person_charlie()))?; // 35
    assert_eq!(
        unwrap_value::<Sequenced<u32>>(Some(unwrap_stream(&mut stream, 500).await)).value,
        90
    );

    Ok(())
}
