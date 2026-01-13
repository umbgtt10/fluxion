// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::into_stream::IntoStream;
use fluxion_core::{StreamItem, Timestamped};
use fluxion_stream::prelude::*;
use fluxion_stream::MergedStream;
use fluxion_test_utils::{
    assert_no_element_emitted,
    helpers::unwrap_stream,
    test_channel,
    test_data::{
        animal_dog, person_alice, person_bob, person_charlie, person_dave, person_diane,
        plant_rose, TestData,
    },
    Sequenced,
};

#[tokio::test]
async fn test_ordered_merge_filter_ordered() -> anyhow::Result<()> {
    // Arrange - merge two streams, then filter for specific types
    let (s1_tx, s1_rx) = test_channel::<Sequenced<TestData>>();
    let (s2_tx, s2_rx) = test_channel::<Sequenced<TestData>>();

    let mut stream = s1_rx
        .ordered_merge(vec![s2_rx])
        .filter_ordered(|test_data| !matches!(test_data, TestData::Animal(_))); // Filter out animals

    // Act & Assert
    s1_tx.try_send(Sequenced::new(person_alice()))?;
    assert!(matches!(
        unwrap_stream(&mut stream, 500).await,
        StreamItem::Value(val) if val.value == person_alice()
    ));

    s2_tx.try_send(Sequenced::new(animal_dog()))?; // Filtered out
    s1_tx.try_send(Sequenced::new(plant_rose()))?;
    assert!(matches!(
        unwrap_stream(&mut stream, 500).await,
        StreamItem::Value(val) if val.value == plant_rose()
    ));

    s2_tx.try_send(Sequenced::new(person_bob()))?;
    assert!(matches!(
        unwrap_stream(&mut stream, 500).await,
        StreamItem::Value(val) if val.value == person_bob()
    ));

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_filter_ordered() -> anyhow::Result<()> {
    // Arrange - combine latest from multiple streams, then filter
    let (p_tx, p_rx) = test_channel::<Sequenced<TestData>>();
    let (a_tx, a_rx) = test_channel::<Sequenced<TestData>>();

    let mut stream = p_rx
        .combine_latest(vec![a_rx], |_| true)
        .filter_ordered(|wrapper| {
            // Filter: only emit when first item is a person with age > 30
            let state = &wrapper.clone().into_inner();
            match &state.values()[0] {
                TestData::Person(p) => p.age > 30,
                _ => false,
            }
        });

    // Act & Assert
    p_tx.try_send(Sequenced::new(person_alice()))?; // 25
    a_tx.try_send(Sequenced::new(animal_dog()))?;
    // Combined but filtered out (age <= 30)

    p_tx.try_send(Sequenced::new(person_charlie()))?; // 35
    assert!(matches!(
        unwrap_stream(&mut stream, 500).await,
        StreamItem::Value(val) if {
            let state = val.values();
            state[0] == person_charlie() && state[1] == animal_dog()
        }
    ));

    p_tx.try_send(Sequenced::new(person_diane()))?; // 40
    assert!(matches!(
        unwrap_stream(&mut stream, 500).await,
        StreamItem::Value(val) if {
            let state = val.values();
            state[0] == person_diane()
        }
    ));

    Ok(())
}

#[tokio::test]
async fn test_merge_with_chaining_filter_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    // Act: Chain merge_with with filter_ordered (only values > 2)
    let mut result = MergedStream::seed::<Sequenced<usize>>(0)
        .merge_with(stream, |_item: TestData, state| {
            *state += 1;
            *state
        })
        .into_stream()
        .filter_ordered(|&value| value > 2);

    // Send first value - state will be 1 (filtered out)
    tx.try_send(Sequenced::new(person_alice()))?;
    assert_no_element_emitted(&mut result, 500).await;

    // Send second value - state will be 2 (filtered out)
    tx.try_send(Sequenced::new(person_bob()))?;
    assert_no_element_emitted(&mut result, 500).await;

    // Send third value - state will be 3 (kept)
    tx.try_send(Sequenced::new(person_charlie()))?;
    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Value(val) if val.value == 3
    ));

    // Send fourth value - state will be 4 (kept)
    tx.try_send(Sequenced::new(person_dave()))?;

    // Assert: fourth emission also passes
    assert!(matches!(
        unwrap_stream(&mut result, 500).await,
        StreamItem::Value(val) if val.value == 4
    ));

    Ok(())
}

#[tokio::test]
async fn test_scan_ordered_composed_with_filter() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let accumulator = |count: &mut i32, _: &TestData| {
        *count += 1;
        *count
    };

    let mut result = stream
        .scan_ordered(0, accumulator)
        .filter_ordered(|count| count % 2 == 0); // Only even counts

    // Act & Assert
    tx.try_send(Sequenced::new(person_alice()))?; // count=1, filtered out
    assert_no_element_emitted(&mut result, 500).await;

    tx.try_send(Sequenced::new(person_bob()))?; // count=2, emitted
    assert!(matches!(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 500).await,
        StreamItem::Value(val) if val.value == 2
    ));

    tx.try_send(Sequenced::new(person_charlie()))?; // count=3, filtered out
    assert_no_element_emitted(&mut result, 500).await;

    tx.try_send(Sequenced::new(person_dave()))?; // count=4, emitted
    assert!(matches!(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 500).await,
        StreamItem::Value(val) if val.value == 4
    ));

    drop(tx);

    Ok(())
}
