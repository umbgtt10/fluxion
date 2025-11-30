// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::FluxionError;
use fluxion_core::{HasTimestamp, StreamItem, Timestamped};
use fluxion_rx::{CombinedState, FluxionStream};
use fluxion_stream::{DistinctUntilChangedByExt, DistinctUntilChangedExt, MergedStream};
use fluxion_test_utils::helpers::assert_no_element_emitted;
use fluxion_test_utils::helpers::unwrap_stream;
use fluxion_test_utils::test_channel;
use fluxion_test_utils::test_channel_with_errors;
use fluxion_test_utils::test_data::{
    animal_dog, person_alice, person_bob, person_charlie, person_dave, person_diane, plant_rose,
    TestData,
};
use fluxion_test_utils::test_wrapper::TestWrapper;
use fluxion_test_utils::unwrap_value;
use fluxion_test_utils::Sequenced;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

static FILTER: fn(&TestData) -> bool = |_| true;
static COMBINE_FILTER: fn(&CombinedState<TestData, u64>) -> bool = |_| true;

#[tokio::test]
async fn test_double_ordered_merge_map_ordered() -> anyhow::Result<()> {
    // Arrange - merge two pairs of streams, then merge results
    let (s1_tx, s1_rx) = test_channel::<Sequenced<TestData>>();
    let (s2_tx, s2_rx) = test_channel::<Sequenced<TestData>>();

    let s1_stream = s1_rx;
    let s2_stream = s2_rx;

    let mut stream = FluxionStream::new(s1_stream)
        .ordered_merge(vec![FluxionStream::new(s2_stream)])
        .combine_with_previous()
        .map_ordered(|stream_item| async move {
            let item = stream_item;
            let type_name = match &item.current.value {
                TestData::Person(_) => "Person",
                TestData::Animal(_) => "Animal",
                TestData::Plant(_) => "Plant",
            };
            let count = if item.previous.is_some() { 2 } else { 1 };
            StreamItem::Value(format!("{} (item #{})", type_name, count))
        });

    // Act & Assert
    s1_tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Person (item #1)"
    );

    s2_tx.send(Sequenced::new(animal_dog()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Animal (item #2)"
    );

    s1_tx.send(Sequenced::new(plant_rose()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Plant (item #2)"
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_map_ordered_data_extraction() -> anyhow::Result<()> {
    // Arrange - extract specific fields from merged data
    let (s1_tx, s1_rx) = test_channel::<Sequenced<TestData>>();
    let (s2_tx, s2_rx) = test_channel::<Sequenced<TestData>>();

    let s1_stream = s1_rx;
    let s2_stream = s2_rx;

    let mut stream = FluxionStream::new(s1_stream)
        .ordered_merge(vec![FluxionStream::new(s2_stream)])
        .combine_with_previous()
        .map_ordered(|stream_item| async move {
            let item = stream_item;
            StreamItem::Value(match &item.current.value {
                TestData::Person(p) => format!("Person: {}, Age: {}", p.name, p.age),
                TestData::Animal(a) => format!("Animal: {}, Legs: {}", a.name, a.legs),
                TestData::Plant(p) => format!("Plant: {}, Height: {}", p.species, p.height),
            })
        });

    // Act & Assert
    s1_tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Person: Alice, Age: 25"
    );

    s2_tx.send(Sequenced::new(animal_dog()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Animal: Dog, Legs: 4"
    );

    s1_tx.send(Sequenced::new(plant_rose()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Plant: Rose, Height: 15"
    );
    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_with_map_ordered() -> anyhow::Result<()> {
    // Arrange - filter for people only, then map to names
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(stream)
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .map_ordered(|stream_item| async move {
            let item = stream_item;
            StreamItem::Value(match &item.value {
                TestData::Person(p) => format!("Person: {}", p.name),
                _ => unreachable!(),
            })
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Person: Alice"
    );

    tx.send(Sequenced::new(animal_dog()))?; // Filtered out
    tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Person: Bob"
    );

    tx.send(Sequenced::new(plant_rose()))?; // Filtered out
    tx.send(Sequenced::new(person_charlie()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Person: Charlie"
    );

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_with_combine_with_previous() -> anyhow::Result<()> {
    // Arrange - filter for adults (age > 25), then track changes
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(stream)
        .filter_ordered(|test_data| match test_data {
            TestData::Person(p) => p.age > 25,
            _ => false,
        })
        .combine_with_previous();

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?; // 25 - filtered
    tx.send(Sequenced::new(person_bob()))?; // 30 - kept
    let item = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert!(item.previous.is_none());
    assert_eq!(&item.current.value, &person_bob());

    tx.send(Sequenced::new(person_charlie()))?; // 35 - kept
    let item = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&item.previous.unwrap().value, &person_bob());
    assert_eq!(&item.current.value, &person_charlie());

    tx.send(Sequenced::new(person_dave()))?; // 28 - kept
    let item = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&item.previous.unwrap().value, &person_charlie());
    assert_eq!(&item.current.value, &person_dave());

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_with_filter_ordered() -> anyhow::Result<()> {
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
async fn test_filter_ordered_with_take_latest_when() -> anyhow::Result<()> {
    // Arrange - filter source stream, then apply take_latest_when
    let (source_tx, source_rx) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_rx) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(source_rx)
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .take_latest_when(FluxionStream::new(filter_rx), FILTER);

    // Act & Assert
    source_tx.send(Sequenced::new(person_alice()))?;
    source_tx.send(Sequenced::new(animal_dog()))?; // Filtered
    source_tx.send(Sequenced::new(person_bob()))?;

    filter_tx.send(Sequenced::new(person_alice()))?; // Trigger emission

    {
        let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));

        assert_eq!(&val.value, &person_bob());
    }

    source_tx.send(Sequenced::new(person_charlie()))?;
    source_tx.send(Sequenced::new(plant_rose()))?; // Filtered

    filter_tx.send(Sequenced::new(person_bob()))?; // Trigger emission
    {
        let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
        assert_eq!(&val.value, &person_charlie());
    }

    Ok(())
}

#[tokio::test]
async fn test_filter_ordered_map_ordered_combine_with_previous() -> anyhow::Result<()> {
    // Arrange - complex pipeline: filter -> map -> combine_with_previous
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(stream)
        .filter_ordered(|data| match data {
            TestData::Person(p) => p.age >= 30,
            _ => false,
        })
        .combine_with_previous()
        .map_ordered(|stream_item| async move {
            let item = stream_item;
            let current = match &item.current.value {
                TestData::Person(p) => p.name.clone(),
                _ => unreachable!(),
            };
            let previous = item.previous.map(|prev| match &prev.value {
                TestData::Person(p) => p.name.clone(),
                _ => unreachable!(),
            });
            StreamItem::Value(format!("Current: {}, Previous: {:?}", current, previous))
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?; // 25 - filtered
    tx.send(Sequenced::new(person_bob()))?; // 30 - kept

    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Current: Bob, Previous: None"
    );

    tx.send(Sequenced::new(person_charlie()))?; // 35 - kept
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Current: Charlie, Previous: Some(\"Bob\")"
    );

    tx.send(Sequenced::new(person_dave()))?; // 28 - filtered
    tx.send(Sequenced::new(person_diane()))?; // 40 - kept
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await))
            .await
            .unwrap(),
        "Current: Diane, Previous: Some(\"Charlie\")"
    );

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_with_filter_ordered() -> anyhow::Result<()> {
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
async fn test_filter_ordered_with_latest_from() -> anyhow::Result<()> {
    // Arrange - filter primary stream, then combine with custom selector
    let (primary_tx, primary_rx) = test_channel::<Sequenced<TestData>>();
    let (secondary_tx, secondary_rx) = test_channel::<Sequenced<TestData>>();

    // Custom selector: extract name from person and combine with secondary info
    let name_combiner = |state: &CombinedState<TestData, u64>| -> TestWrapper<String> {
        let person_name = match &state.values()[0] {
            TestData::Person(p) => p.name.clone(),
            _ => String::from("Unknown"),
        };
        let secondary_info = match &state.values()[1] {
            TestData::Animal(a) => format!("with animal {} ({} legs)", a.name, a.legs),
            TestData::Person(p) => format!("with person {} (age {})", p.name, p.age),
            TestData::Plant(p) => format!("with plant {} (height {})", p.species, p.height),
        };
        TestWrapper::new(
            format!("{} {}", person_name, secondary_info),
            state.timestamp(),
        )
    };

    let mut stream = FluxionStream::new(primary_rx)
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .with_latest_from(FluxionStream::new(secondary_rx), name_combiner);

    // Act & Assert
    secondary_tx.send(Sequenced::new(animal_dog()))?;
    primary_tx.send(Sequenced::new(plant_rose()))?; // Filtered
    primary_tx.send(Sequenced::new(person_alice()))?; // Kept

    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    let combined_name = result.clone().into_inner();
    assert_eq!(combined_name, "Alice with animal Dog (4 legs)");

    // Update secondary to a person
    secondary_tx.send(Sequenced::new(person_bob()))?;
    primary_tx.send(Sequenced::new(person_charlie()))?;

    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    let combined_name = result.clone().into_inner();
    assert_eq!(combined_name, "Charlie with person Bob (age 30)");

    // Send animal (filtered) and plant (filtered)
    primary_tx.send(Sequenced::new(animal_dog()))?; // Filtered
    primary_tx.send(Sequenced::new(plant_rose()))?; // Filtered

    // Verify no emission yet by checking with a timeout
    assert_no_element_emitted(&mut stream, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_with_latest_from_in_middle_of_chain() -> anyhow::Result<()> {
    let (primary_tx, primary_rx) = test_channel::<Sequenced<TestData>>();
    let (secondary_tx, secondary_rx) = test_channel::<Sequenced<TestData>>();

    // Custom selector: combine ages
    let age_combiner = |state: &CombinedState<TestData, u64>| -> TestWrapper<u32> {
        let primary_age = match &state.values()[0] {
            TestData::Person(p) => p.age,
            _ => 0,
        };
        let secondary_age = match &state.values()[1] {
            TestData::Person(p) => p.age,
            _ => 0,
        };
        TestWrapper::new(primary_age + secondary_age, state.timestamp())
    };

    let mut stream = FluxionStream::new(primary_rx)
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .with_latest_from(FluxionStream::new(secondary_rx), age_combiner)
        .map_ordered(|stream_item| async move {
            let age_sum = stream_item.clone().into_inner();
            StreamItem::Value(format!("Combined age: {}", age_sum))
        });

    // Act & Assert
    secondary_tx.send(Sequenced::new(person_alice()))?; // 25
    primary_tx.send(Sequenced::new(animal_dog()))?; // Filtered
    primary_tx.send(Sequenced::new(person_bob()))?; // 30

    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).await;
    assert_eq!(result, StreamItem::Value("Combined age: 55".to_string())); // 30 + 25

    primary_tx.send(Sequenced::new(person_charlie()))?; // 35
    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).await;
    assert_eq!(result, StreamItem::Value("Combined age: 60".to_string())); // 35 + 25

    // Update secondary
    secondary_tx.send(Sequenced::new(person_diane()))?; // 40
    primary_tx.send(Sequenced::new(person_dave()))?; // 28

    let result = unwrap_value(Some(unwrap_stream(&mut stream, 500).await)).await;
    assert_eq!(result, StreamItem::Value("Combined age: 68".to_string())); // 28 + 40

    Ok(())
}

#[tokio::test]
async fn test_take_while_with_in_middle_of_chain() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_rx) = test_channel::<Sequenced<TestData>>();
    let (other_tx, other_rx) = test_channel::<Sequenced<TestData>>();
    let (predicate_tx, predicate_rx) = test_channel::<Sequenced<TestData>>();

    let source_stream = source_rx;
    let other_stream = other_rx;
    let predicate_stream = predicate_rx;

    // Chain ordered operations, then take_while_with at the end
    let mut stream = FluxionStream::new(source_stream)
        .ordered_merge(vec![FluxionStream::new(other_stream)])
        .filter_ordered(|test_data| matches!(test_data, TestData::Person(_)))
        .take_while_with(predicate_stream, |_| true);

    // Act & Assert
    predicate_tx.send(Sequenced::new(person_alice()))?;
    source_tx.send(Sequenced::new(animal_dog()))?; // Filtered by filter_ordered
    source_tx.send(Sequenced::new(person_bob()))?; // Kept
    other_tx.send(Sequenced::new(person_charlie()))?; // Kept
    let result1 = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    let result2 = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));

    assert_eq!(&result1.value, &person_bob());
    assert_eq!(&result2.value, &person_charlie());

    Ok(())
}

#[tokio::test]
async fn test_merge_with_chaining_with_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

    // Act: Chain merge_with with map_ordered that doubles the counter
    let mut result = MergedStream::seed::<Sequenced<usize>>(0)
        .merge_with(stream, |_item: TestData, state| {
            *state += 1;
            *state
        })
        .into_fluxion_stream()
        .map_ordered(|seq| {
            let value = seq.into_inner();
            Sequenced::new(value * 2)
        });

    // Send first value
    tx.send(Sequenced::new(person_alice()))?;

    // Assert first result: state=1, doubled=2
    let StreamItem::Value(first) = unwrap_stream(&mut result, 500).await else {
        panic!("Expected Value");
    };
    assert_eq!(first.into_inner(), 2, "First emission: (0+1)*2 = 2");

    // Send second value
    tx.send(Sequenced::new(person_bob()))?;

    // Assert second result: state=2, doubled=4
    let StreamItem::Value(second) = unwrap_stream(&mut result, 500).await else {
        panic!("Expected Value");
    };
    assert_eq!(second.into_inner(), 4, "Second emission: (1+1)*2 = 4");

    Ok(())
}

#[tokio::test]
async fn test_merge_with_chaining_with_filter_ordered() -> anyhow::Result<()> {
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
async fn test_merge_with_chaining_multiple_operators() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

    // Act: Chain merge_with with map, filter, and another map
    let mut result = MergedStream::seed::<Sequenced<usize>>(0)
        .merge_with(stream, |_item: TestData, state| {
            *state += 1;
            *state
        })
        .into_fluxion_stream()
        .map_ordered(|seq| {
            let value = seq.into_inner();
            Sequenced::new(value * 3)
        })
        .filter_ordered(|&value| value > 6)
        .map_ordered(|seq| {
            let value = seq.into_inner();
            Sequenced::new(value + 10)
        });

    // Send first value - state: 1, *3=3 (filtered out: 3 <= 6)
    tx.send(Sequenced::new(person_alice()))?;

    // Send second value - state: 2, *3=6 (filtered out: 6 <= 6)
    tx.send(Sequenced::new(person_bob()))?;

    // Send third value - state: 3, *3=9, +10=19 (kept: 9 > 6)
    tx.send(Sequenced::new(person_charlie()))?;

    // Assert: first kept value
    let StreamItem::Value(first_kept) = unwrap_stream(&mut result, 500).await else {
        panic!("Expected Value");
    };
    assert_eq!(
        first_kept.into_inner(),
        19,
        "Third emission: 3*3=9, 9+10=19"
    );

    // Send fourth value - state: 4, *3=12, +10=22 (kept: 12 > 6)
    tx.send(Sequenced::new(person_dave()))?;

    // Assert: second kept value
    let StreamItem::Value(second_kept) = unwrap_stream(&mut result, 500).await else {
        panic!("Expected Value");
    };
    assert_eq!(
        second_kept.into_inner(),
        22,
        "Fourth emission: 4*3=12, 12+10=22"
    );

    Ok(())
}

#[tokio::test]
async fn test_on_error_at_end_of_chain() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let error_count = Arc::new(Mutex::new(0));
    let error_count_clone = error_count.clone();

    let mut stream = FluxionStream::new(stream)
        .combine_with_previous()
        .map_ordered(|with_prev| with_prev.current)
        .on_error(move |_err| {
            *error_count_clone.lock().unwrap() += 1;
            true // Consume all errors at the end
        });

    // Act & Assert - send and verify each item
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&val.value, &person_alice());

    // Send error - should be consumed
    tx.send(StreamItem::Error(FluxionError::stream_error("error 1")))?;
    assert_no_element_emitted(&mut stream, 100).await;
    assert_eq!(*error_count.lock().unwrap(), 1, "First error handled");

    tx.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&val.value, &person_bob());

    // Send second error - should be consumed
    tx.send(StreamItem::Error(FluxionError::stream_error("error 2")))?;
    assert_no_element_emitted(&mut stream, 100).await;
    assert_eq!(*error_count.lock().unwrap(), 2, "Second error handled");

    tx.send(StreamItem::Value(Sequenced::new(person_charlie())))?;
    let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&val.value, &person_charlie());

    Ok(())
}

#[tokio::test]
async fn test_on_error_in_middle_of_chain() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let early_errors = Arc::new(Mutex::new(0));
    let late_errors = Arc::new(Mutex::new(0));
    let early_errors_clone = early_errors.clone();
    let late_errors_clone = late_errors.clone();

    let mut stream = FluxionStream::new(stream)
        .on_error(move |err| {
            // Handle specific errors early
            if err.to_string().contains("early") {
                *early_errors_clone.lock().unwrap() += 1;
                true // Consume early errors
            } else {
                false // Propagate other errors
            }
        })
        .combine_with_previous()
        .map_ordered(|with_prev| with_prev.current)
        .on_error(move |err| {
            // Handle remaining errors late
            if err.to_string().contains("late") {
                *late_errors_clone.lock().unwrap() += 1;
                true // Consume late errors
            } else {
                false // Propagate unhandled errors
            }
        });

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&val.value, &person_alice());

    // Send early error - should be consumed by first handler
    tx.send(StreamItem::Error(FluxionError::stream_error(
        "early error 1",
    )))?;
    assert_no_element_emitted(&mut stream, 100).await;
    assert_eq!(*early_errors.lock().unwrap(), 1);
    assert_eq!(*late_errors.lock().unwrap(), 0);

    tx.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&val.value, &person_bob());

    // Send late error - should be propagated and consumed by second handler
    tx.send(StreamItem::Error(fluxion_core::FluxionError::stream_error(
        "late error 1",
    )))?;
    assert_no_element_emitted(&mut stream, 100).await;
    assert_eq!(*early_errors.lock().unwrap(), 1);
    assert_eq!(*late_errors.lock().unwrap(), 1);

    // Send another early error - should be consumed by first handler
    tx.send(StreamItem::Error(fluxion_core::FluxionError::stream_error(
        "early error 2",
    )))?;
    assert_no_element_emitted(&mut stream, 100).await;
    assert_eq!(*early_errors.lock().unwrap(), 2);
    assert_eq!(*late_errors.lock().unwrap(), 1);

    tx.send(StreamItem::Value(Sequenced::new(person_charlie())))?;
    let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&val.value, &person_charlie());

    // Send another late error - should be propagated and consumed by second handler
    tx.send(StreamItem::Error(fluxion_core::FluxionError::stream_error(
        "late error 2",
    )))?;
    assert_no_element_emitted(&mut stream, 100).await;
    assert_eq!(
        *early_errors.lock().unwrap(),
        2,
        "Should have handled 2 early errors"
    );
    assert_eq!(
        *late_errors.lock().unwrap(),
        2,
        "Should have handled 2 late errors"
    );

    Ok(())
}

#[tokio::test]
async fn test_on_error_chain_of_responsibility() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let network_errors = Arc::new(Mutex::new(0));
    let validation_errors = Arc::new(Mutex::new(0));
    let other_errors = Arc::new(Mutex::new(0));
    let network_errors_clone = network_errors.clone();
    let validation_errors_clone = validation_errors.clone();
    let other_errors_clone = other_errors.clone();

    let mut stream = FluxionStream::new(stream)
        .on_error(move |err| {
            // First handler: network errors
            if err.to_string().contains("network") {
                *network_errors_clone.lock().unwrap() += 1;
                true
            } else {
                false
            }
        })
        .combine_with_previous()
        .on_error(move |err| {
            // Second handler: validation errors
            if err.to_string().contains("validation") {
                *validation_errors_clone.lock().unwrap() += 1;
                true
            } else {
                false
            }
        })
        .map_ordered(|with_prev| with_prev.current)
        .filter_ordered(|_| true)
        .on_error(move |_err| {
            // Final catch-all handler
            *other_errors_clone.lock().unwrap() += 1;
            true
        });

    // Act & Assert - send and verify each item with error handling at each stage
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&val.value, &person_alice());

    // Send network error - consumed by first handler
    tx.send(StreamItem::Error(FluxionError::stream_error(
        "network timeout",
    )))?;
    assert_no_element_emitted(&mut stream, 100).await;
    assert_eq!(*network_errors.lock().unwrap(), 1);
    assert_eq!(*validation_errors.lock().unwrap(), 0);
    assert_eq!(*other_errors.lock().unwrap(), 0);

    tx.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&val.value, &person_bob());

    // Send validation error - propagated past first, consumed by second handler
    tx.send(StreamItem::Error(FluxionError::stream_error(
        "validation failed",
    )))?;
    assert_no_element_emitted(&mut stream, 100).await;
    assert_eq!(*network_errors.lock().unwrap(), 1);
    assert_eq!(*validation_errors.lock().unwrap(), 1);
    assert_eq!(*other_errors.lock().unwrap(), 0);

    tx.send(StreamItem::Value(Sequenced::new(person_charlie())))?;
    let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&val.value, &person_charlie());

    // Send unknown error - propagated past first two, consumed by catch-all
    tx.send(StreamItem::Error(fluxion_core::FluxionError::stream_error(
        "unknown error",
    )))?;
    assert_no_element_emitted(&mut stream, 100).await;
    assert_eq!(*network_errors.lock().unwrap(), 1);
    assert_eq!(*validation_errors.lock().unwrap(), 1);
    assert_eq!(*other_errors.lock().unwrap(), 1);

    tx.send(StreamItem::Value(Sequenced::new(person_dave())))?;
    let val = unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
    assert_eq!(&val.value, &person_dave());

    // Send another network error - consumed by first handler
    tx.send(StreamItem::Error(FluxionError::stream_error(
        "network connection lost",
    )))?;
    assert_no_element_emitted(&mut stream, 100).await;
    assert_eq!(
        *network_errors.lock().unwrap(),
        2,
        "Should have handled 2 network errors"
    );
    assert_eq!(
        *validation_errors.lock().unwrap(),
        1,
        "Should have handled 1 validation error"
    );
    assert_eq!(
        *other_errors.lock().unwrap(),
        1,
        "Should have handled 1 unknown error"
    );

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_with_filter_composition() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();

    // Composition: filter -> distinct_until_changed
    let mut result = FluxionStream::new(stream)
        .filter_ordered(|x| *x >= 0) // Filter out negatives
        .distinct_until_changed();

    // Act & Assert
    tx.send(Sequenced::with_timestamp(-5, 1))?; // Filtered out
    tx.send(Sequenced::with_timestamp(1, 2))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 1);

    tx.send(Sequenced::with_timestamp(1, 3))?; // Duplicate, filtered by distinct
    tx.send(Sequenced::with_timestamp(2, 4))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 2);

    tx.send(Sequenced::with_timestamp(-10, 5))?; // Filtered out
    tx.send(Sequenced::with_timestamp(2, 6))?; // Duplicate, filtered by distinct
    tx.send(Sequenced::with_timestamp(3, 7))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 3);

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_with_map_composition() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();

    // Composition: map -> distinct_until_changed
    let mut result = FluxionStream::new(stream)
        .map_ordered(|s| {
            let abs_value = s.value.abs();
            Sequenced::new(abs_value)
        })
        .distinct_until_changed();

    // Act & Assert
    tx.send(Sequenced::with_timestamp(-5, 1))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 5);

    tx.send(Sequenced::with_timestamp(5, 2))?; // Same after abs()
    tx.send(Sequenced::with_timestamp(-10, 3))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 10);

    tx.send(Sequenced::with_timestamp(10, 4))?; // Same after abs()
    tx.send(Sequenced::with_timestamp(-10, 5))?; // Same after abs()
    tx.send(Sequenced::with_timestamp(7, 6))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 7);

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_with_combine_with_previous_composition() -> anyhow::Result<()>
{
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();

    // Composition: distinct_until_changed -> combine_with_previous
    let mut result = FluxionStream::new(stream)
        .distinct_until_changed()
        .combine_with_previous();

    // Act & Assert
    tx.send(Sequenced::with_timestamp(1, 1))?;
    let combined = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(combined.current.value, 1);
    assert_eq!(combined.previous, None);

    tx.send(Sequenced::with_timestamp(1, 2))?; // Filtered by distinct
    tx.send(Sequenced::with_timestamp(2, 3))?;
    let combined = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(combined.current.value, 2);
    assert_eq!(combined.previous.as_ref().unwrap().value, 1);

    tx.send(Sequenced::with_timestamp(2, 4))?; // Filtered by distinct
    tx.send(Sequenced::with_timestamp(2, 5))?; // Filtered by distinct
    tx.send(Sequenced::with_timestamp(3, 6))?;
    let combined = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(combined.current.value, 3);
    assert_eq!(combined.previous.as_ref().unwrap().value, 2);

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_combine_latest_with_distinct_until_changed_composition() -> anyhow::Result<()> {
    // Arrange
    let (stream1_tx, stream1) = test_channel::<Sequenced<i32>>();
    let (stream2_tx, stream2) = test_channel::<Sequenced<i32>>();

    // Composition: combine_latest -> map to sum -> distinct_until_changed
    let mut result = FluxionStream::new(stream1)
        .combine_latest(vec![stream2], |_| true)
        .map_ordered(|state| {
            let sum = state.values()[0] + state.values()[1];
            Sequenced::new(sum)
        })
        .distinct_until_changed();

    // Act & Assert
    // Initial values: 1 + 2 = 3
    stream1_tx.send(Sequenced::with_timestamp(1, 1))?;
    stream2_tx.send(Sequenced::with_timestamp(2, 2))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 3);

    // Update stream1: 2 + 2 = 4 (different, should emit)
    stream1_tx.send(Sequenced::with_timestamp(2, 3))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 4);

    // Update stream2: 2 + 3 = 5 (different, should emit)
    stream2_tx.send(Sequenced::with_timestamp(3, 4))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 5);

    // Update stream1: 3 + 3 = 6 (different, should emit)
    stream1_tx.send(Sequenced::with_timestamp(3, 5))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 6);

    // Update stream2: 3 + 4 = 7 (different, should emit)
    stream2_tx.send(Sequenced::with_timestamp(4, 6))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 7);

    // Update stream1: 4 + 4 = 8 (different, should emit)
    stream1_tx.send(Sequenced::with_timestamp(4, 7))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 8);

    // Update stream2: 4 + 5 = 9 (different, should emit)
    stream2_tx.send(Sequenced::with_timestamp(5, 8))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 9);

    // Update stream1 to make sum same as before: 5 + 5 = 10 (different, should emit)
    stream1_tx.send(Sequenced::with_timestamp(5, 9))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 10);

    // Update stream2: 5 + 6 = 11 (different, should emit)
    stream2_tx.send(Sequenced::with_timestamp(6, 10))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 11);

    // Update stream1 to get same sum: 6 + 6 = 12 (different, should emit)
    stream1_tx.send(Sequenced::with_timestamp(6, 11))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 12);

    // Update stream2 to make sum same as before: 6 + 6 = 12 (same! should NOT emit)
    stream2_tx.send(Sequenced::with_timestamp(6, 12))?;

    // Update stream1 to different sum: 7 + 6 = 13 (different, should emit)
    stream1_tx.send(Sequenced::with_timestamp(7, 13))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 13);

    drop(stream1_tx);
    drop(stream2_tx);

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_with_filter_composition() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();

    // Composition: filter -> distinct_until_changed_by (parity check)
    let mut result = FluxionStream::new(stream)
        .filter_ordered(|x| *x > 0) // Filter out negatives and zero
        .distinct_until_changed_by(|a, b| a % 2 == b % 2); // Only emit when parity changes

    // Act & Assert
    tx.send(Sequenced::with_timestamp(-5, 1))?; // Filtered out
    tx.send(Sequenced::with_timestamp(0, 2))?; // Filtered out
    tx.send(Sequenced::with_timestamp(1, 3))?; // Odd - emitted
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 1);

    tx.send(Sequenced::with_timestamp(3, 4))?; // Odd - filtered by distinct_by
    tx.send(Sequenced::with_timestamp(5, 5))?; // Odd - filtered by distinct_by
    assert_no_element_emitted(&mut result, 100).await;

    tx.send(Sequenced::with_timestamp(2, 6))?; // Even - emitted (parity changed)
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 2);

    tx.send(Sequenced::with_timestamp(4, 7))?; // Even - filtered by distinct_by
    assert_no_element_emitted(&mut result, 100).await;

    tx.send(Sequenced::with_timestamp(-1, 8))?; // Filtered by filter_ordered
    tx.send(Sequenced::with_timestamp(7, 9))?; // Odd - emitted (parity changed)
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 7);

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_with_map_composition() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<String>>();

    // Composition: map to length -> distinct_until_changed_by (threshold comparison)
    let mut result = FluxionStream::new(stream)
        .map_ordered(|s| {
            let len = s.value.len();
            Sequenced::new(len)
        })
        .distinct_until_changed_by(|a, b| {
            // Consider lengths "same" if difference < 2
            (*a as i32 - *b as i32).abs() < 2
        });

    // Act & Assert
    tx.send(Sequenced::with_timestamp("a".to_string(), 1))?; // len=1
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 1);

    tx.send(Sequenced::with_timestamp("ab".to_string(), 2))?; // len=2, diff=1 < 2 - filtered
    assert_no_element_emitted(&mut result, 100).await;

    tx.send(Sequenced::with_timestamp("abc".to_string(), 3))?; // len=3, diff=2 >= 2 - emitted
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 3);

    tx.send(Sequenced::with_timestamp("abcd".to_string(), 4))?; // len=4, diff=1 - filtered
    assert_no_element_emitted(&mut result, 100).await;

    tx.send(Sequenced::with_timestamp("abcdef".to_string(), 5))?; // len=6, diff=3 >= 2 - emitted
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 6);

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_with_combine_latest_composition() -> anyhow::Result<()> {
    // Arrange
    let (stream1_tx, stream1) = test_channel::<Sequenced<i32>>();
    let (stream2_tx, stream2) = test_channel::<Sequenced<i32>>();

    // Composition: combine_latest -> map to max -> distinct_until_changed_by (threshold)
    let mut result = FluxionStream::new(stream1)
        .combine_latest(vec![stream2], |_| true)
        .map_ordered(|state| {
            let max = *state.values().iter().max().unwrap();
            Sequenced::new(max)
        })
        .distinct_until_changed_by(|a, b| {
            // Only emit if max changes by at least 5
            (a - b).abs() < 5
        });

    // Act & Assert
    stream1_tx.send(Sequenced::with_timestamp(10, 1))?;
    stream2_tx.send(Sequenced::with_timestamp(5, 2))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 10); // max(10, 5) = 10

    // Small changes - filtered
    stream1_tx.send(Sequenced::with_timestamp(11, 3))?;
    stream2_tx.send(Sequenced::with_timestamp(7, 4))?;
    assert_no_element_emitted(&mut result, 100).await;

    // Large change - emitted
    stream1_tx.send(Sequenced::with_timestamp(20, 5))?;
    let item = unwrap_value(Some(unwrap_stream(&mut result, 100).await));
    assert_eq!(item.value, 20); // max(20, 7) = 20, diff=10 >= 5

    drop(stream1_tx);
    drop(stream2_tx);

    Ok(())
}

#[tokio::test]
async fn test_scan_ordered_composed_with_map() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let accumulator = |count: &mut i32, _: &TestData| {
        *count += 1;
        *count
    };

    let mut result = FluxionStream::new(stream)
        .scan_ordered(0, accumulator)
        .map_ordered(|count: Sequenced<i32>| Sequenced::new(count.into_inner() * 10));

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?;
    let value = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(value.value, 10);

    tx.send(Sequenced::new(person_bob()))?;
    let value = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(value.value, 20);

    tx.send(Sequenced::new(person_charlie()))?;
    let value = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(value.value, 30);

    drop(tx);

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
    let value = unwrap_value(Some(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 500).await,
    ));
    assert_eq!(value.value, 2);

    tx.send(Sequenced::new(person_charlie()))?; // count=3, filtered out
    tx.send(Sequenced::new(person_dave()))?; // count=4, emitted
    let value = unwrap_value(Some(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 500).await,
    ));
    assert_eq!(value.value, 4);

    drop(tx);

    Ok(())
}
#[tokio::test]
async fn test_scan_ordered_chained() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();

    // First scan: running sum
    // Second scan: count of emissions
    let mut result = FluxionStream::new(stream)
        .scan_ordered::<Sequenced<i32>, _, _>(0, |sum: &mut i32, value: &i32| {
            *sum += value;
            *sum
        })
        .scan_ordered(0, |count: &mut i32, _sum: &i32| {
            *count += 1;
            *count
        });

    // Act & Assert
    tx.send(Sequenced::new(10))?; // sum=10, count=1
    let value = unwrap_value(Some(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 500).await,
    ));
    assert_eq!(value.value, 1);

    tx.send(Sequenced::new(20))?; // sum=30, count=2
    let value = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(value.value, 2);

    tx.send(Sequenced::new(30))?; // sum=60, count=3
    let value = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    assert_eq!(value.value, 3);

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_start_with_then_take_items() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<_>>();

    let initial = vec![
        StreamItem::Value(Sequenced::new(person_alice())),
        StreamItem::Value(Sequenced::new(person_bob())),
    ];

    let mut result = FluxionStream::new(stream).start_with(initial).take_items(3); // Take 2 initial + 1 from stream

    // Act
    tx.send(Sequenced::new(person_charlie()))?;
    tx.send(Sequenced::new(person_dave()))?; // Should not be emitted

    // Assert - 2 initial values + 1 from stream
    let item1 = unwrap_stream(&mut result, 100).await.unwrap();
    assert_eq!(item1.into_inner(), person_alice());

    let item2 = unwrap_stream(&mut result, 100).await.unwrap();
    assert_eq!(item2.into_inner(), person_bob());

    let item3 = unwrap_stream(&mut result, 100).await.unwrap();
    assert_eq!(item3.into_inner(), person_charlie());

    fluxion_test_utils::helpers::assert_stream_ended(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_skip_items_then_take_items() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<_>>();

    let mut result = FluxionStream::new(stream)
        .skip_items(2) // Skip first 2
        .take_items(2); // Then take next 2

    // Act
    tx.send(Sequenced::new(person_alice()))?; // Skipped
    tx.send(Sequenced::new(person_bob()))?; // Skipped
    tx.send(Sequenced::new(person_charlie()))?; // Taken
    tx.send(Sequenced::new(person_dave()))?; // Taken
    tx.send(Sequenced::new(person_alice()))?; // Not emitted (take limit reached)

    // Assert - Only charlie and dave
    let item1 = unwrap_stream(&mut result, 100).await.unwrap();
    assert_eq!(item1.into_inner(), person_charlie());

    let item2 = unwrap_stream(&mut result, 100).await.unwrap();
    assert_eq!(item2.into_inner(), person_dave());

    fluxion_test_utils::helpers::assert_stream_ended(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_items_with_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();

    let mut result = FluxionStream::new(stream)
        .take_items(3)
        .map_ordered(|item| item.value * 2);

    // Act
    tx.send(Sequenced::new(10))?;
    tx.send(Sequenced::new(20))?;
    tx.send(Sequenced::new(30))?;
    tx.send(Sequenced::new(40))?; // Should not be emitted

    // Assert - First 3 values doubled
    let item1 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item1, StreamItem::Value(20)));

    let item2 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item2, StreamItem::Value(40)));

    let item3 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item3, StreamItem::Value(60)));

    fluxion_test_utils::helpers::assert_stream_ended(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_skip_items_with_filter_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();

    let mut result = FluxionStream::new(stream)
        .skip_items(2) // Skip first 2
        .filter_ordered(|x| *x > 15); // Then filter

    // Act
    tx.send(Sequenced::new(5))?; // Skipped
    tx.send(Sequenced::new(10))?; // Skipped
    tx.send(Sequenced::new(12))?; // Not skipped, but filtered (12 <= 15)
    tx.send(Sequenced::new(20))?; // Emitted
    tx.send(Sequenced::new(30))?; // Emitted

    // Assert - Only values > 15 after skip
    let item1 = unwrap_stream(&mut result, 100).await.unwrap();
    assert_eq!(item1.into_inner(), 20);

    let item2 = unwrap_stream(&mut result, 100).await.unwrap();
    assert_eq!(item2.into_inner(), 30);

    Ok(())
}

#[tokio::test]
async fn test_start_with_combine_with_previous() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();

    let initial = vec![
        StreamItem::Value(Sequenced::new(1)),
        StreamItem::Value(Sequenced::new(2)),
    ];

    let mut result = FluxionStream::new(stream)
        .start_with(initial)
        .combine_with_previous();

    // Act
    tx.send(Sequenced::new(3))?;
    tx.send(Sequenced::new(4))?;

    // Assert - First has no previous
    let item1 = unwrap_stream(&mut result, 100).await.unwrap();
    assert!(item1.previous.is_none());
    assert_eq!(item1.current.into_inner(), 1);

    // Second has previous = 1
    let item2 = unwrap_stream(&mut result, 100).await.unwrap();
    assert_eq!(item2.previous.unwrap().into_inner(), 1);
    assert_eq!(item2.current.into_inner(), 2);

    // Third has previous = 2
    let item3 = unwrap_stream(&mut result, 100).await.unwrap();
    assert_eq!(item3.previous.unwrap().into_inner(), 2);
    assert_eq!(item3.current.into_inner(), 3);

    // Fourth has previous = 3
    let item4 = unwrap_stream(&mut result, 100).await.unwrap();
    assert_eq!(item4.previous.unwrap().into_inner(), 3);
    assert_eq!(item4.current.into_inner(), 4);

    Ok(())
}

#[tokio::test]
async fn test_complex_chain_with_all_three_operators() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();

    let initial = vec![
        StreamItem::Value(Sequenced::new(0)),
        StreamItem::Value(Sequenced::new(1)),
    ];

    // Start with 2 initial values, skip 1, take 4, then map
    let mut result = FluxionStream::new(stream)
        .start_with(initial)
        .skip_items(1) // Skip the 0
        .take_items(4) // Take next 4: [1, 2, 3, 4]
        .map_ordered(|item| item.value * 10);

    // Act
    tx.send(Sequenced::new(2))?;
    tx.send(Sequenced::new(3))?;
    tx.send(Sequenced::new(4))?;
    tx.send(Sequenced::new(5))?; // Should not be emitted (take limit)

    // Assert - [1, 2, 3, 4] * 10 = [10, 20, 30, 40]
    let item1 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item1, StreamItem::Value(10)));

    let item2 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item2, StreamItem::Value(20)));

    let item3 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item3, StreamItem::Value(30)));

    let item4 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item4, StreamItem::Value(40)));

    fluxion_test_utils::helpers::assert_stream_ended(&mut result, 100).await;

    Ok(())
}
