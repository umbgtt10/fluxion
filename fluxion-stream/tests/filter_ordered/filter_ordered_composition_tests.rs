// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{StreamItem, Timestamped};
use fluxion_stream::{CombinedState, FluxionStream, MergedStream};
use fluxion_test_utils::{
    helpers::unwrap_stream,
    test_channel,
    test_data::{
        animal_dog, person_alice, person_bob, person_charlie, person_dave, person_diane,
        plant_rose, TestData,
    },
    unwrap_value, Sequenced,
};
use tokio::sync::mpsc;

static COMBINE_FILTER: fn(&CombinedState<TestData, u64>) -> bool = |_| true;

#[tokio::test]
async fn test_ordered_merge_filter_ordered() -> anyhow::Result<()> {
    // Arrange - merge two streams, then filter for specific types
    let (s1_tx, s1_rx) = test_channel::<Sequenced<TestData>>();
    let (s2_tx, s2_rx) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(s1_rx)
        .ordered_merge(vec![FluxionStream::new(s2_rx)])
        .filter_ordered(|test_data| !matches!(test_data, TestData::Animal(_))); // Filter out animals

    // Act & Assert
    s1_tx.send(Sequenced::new(person_alice()))?;
    {
        let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
        assert_eq!(&val.value, &person_alice());
    }

    s2_tx.send(Sequenced::new(animal_dog()))?; // Filtered out
    s1_tx.send(Sequenced::new(plant_rose()))?;
    {
        let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
        assert_eq!(&val.value, &plant_rose());
    }

    s2_tx.send(Sequenced::new(person_bob()))?;
    {
        let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
        assert_eq!(&val.value, &person_bob());
    }

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_filter_ordered() -> anyhow::Result<()> {
    // Arrange - combine latest from multiple streams, then filter
    let (p_tx, p_rx) = test_channel::<Sequenced<TestData>>();
    let (a_tx, a_rx) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(p_rx)
        .combine_latest(vec![a_rx], COMBINE_FILTER)
        .filter_ordered(|wrapper| {
            // Filter: only emit when first item is a person with age > 30
            let state = &wrapper.clone().into_inner();
            match &state.values()[0] {
                TestData::Person(p) => p.age > 30,
                _ => false,
            }
        });

    // Act & Assert
    p_tx.send(Sequenced::new(person_alice()))?; // 25
    a_tx.send(Sequenced::new(animal_dog()))?;
    // Combined but filtered out (age <= 30)

    p_tx.send(Sequenced::new(person_charlie()))?; // 35
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    let inner = result.clone().into_inner();
    let state = inner.values();
    assert_eq!(&state[0], &person_charlie());
    assert_eq!(&state[1], &animal_dog());

    p_tx.send(Sequenced::new(person_diane()))?; // 40
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    let inner = result.clone().into_inner();
    let state = inner.values();
    assert_eq!(&state[0], &person_diane());

    Ok(())
}

#[tokio::test]
async fn test_merge_with_chaining_filter_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

    // Act: Chain merge_with with filter_ordered (only values > 2)
    let mut result = MergedStream::seed::<Sequenced<usize>>(0)
        .merge_with(stream, |_item: TestData, state| {
            *state += 1;
            *state
        })
        .into_fluxion_stream()
        .filter_ordered(|&value| value > 2);

    // Send first value - state will be 1 (filtered out)
    tx.send(Sequenced::new(person_alice()))?;

    // Send second value - state will be 2 (filtered out)
    tx.send(Sequenced::new(person_bob()))?;

    // Send third value - state will be 3 (kept)
    tx.send(Sequenced::new(person_charlie()))?;

    // Assert: only the third emission passes the filter
    let StreamItem::Value(first_kept) = unwrap_stream(&mut result, 500).await else {
        panic!("Expected Value");
    };
    assert_eq!(
        first_kept.into_inner(),
        3,
        "Third emission passes filter: 3 > 2"
    );

    // Send fourth value - state will be 4 (kept)
    tx.send(Sequenced::new(person_dave()))?;

    // Assert: fourth emission also passes
    let StreamItem::Value(second_kept) = unwrap_stream(&mut result, 500).await else {
        panic!("Expected Value");
    };
    assert_eq!(
        second_kept.into_inner(),
        4,
        "Fourth emission passes filter: 4 > 2"
    );

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

    let mut result = FluxionStream::new(stream)
        .scan_ordered(0, accumulator)
        .filter_ordered(|count| count % 2 == 0); // Only even counts

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?; // count=1, filtered out
    tx.send(Sequenced::new(person_bob()))?; // count=2, emitted
    assert_eq!(
        unwrap_value(Some(
            unwrap_stream::<Sequenced<i32>, _>(&mut result, 500).await,
        ))
        .value,
        2
    );

    tx.send(Sequenced::new(person_charlie()))?; // count=3, filtered out
    tx.send(Sequenced::new(person_dave()))?; // count=4, emitted
    assert_eq!(
        unwrap_value(Some(
            unwrap_stream::<Sequenced<i32>, _>(&mut result, 500).await,
        ))
        .value,
        4
    );

    drop(tx);

    Ok(())
}
