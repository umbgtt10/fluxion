// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `scan_ordered` operator.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::ScanOrderedExt;
use fluxion_test_utils::{
    assert_stream_ended,
    helpers::unwrap_stream,
    test_channel_with_errors,
    test_data::{person_alice, person_bob, person_charlie, TestData},
    Sequenced,
};

#[tokio::test]
async fn test_scan_ordered_propagates_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.scan_ordered(0, |count, _| {
        *count += 1;
        *count
    });

    // Act & Assert: First value
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 1
    ));

    // Error should be propagated
    tx.send(StreamItem::Error(FluxionError::stream_error("Test error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // State should be preserved after error
    tx.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 2
    ));

    drop(tx);
    assert_stream_ended::<_, StreamItem<Sequenced<i32>>>(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_scan_ordered_error_at_start() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.scan_ordered(100, |count, _| {
        *count += 1;
        *count
    });

    // Act & Assert: Error before any values
    tx.send(StreamItem::Error(FluxionError::stream_error(
        "Initial error",
    )))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // First value should still work with initial accumulator
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 101
    ));

    // Second value continues accumulation
    tx.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 102
    ));

    drop(tx);
    assert_stream_ended::<_, StreamItem<Sequenced<i32>>>(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_scan_ordered_multiple_consecutive_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.scan_ordered(Vec::<String>::new(), |names, data| {
        if let TestData::Person(p) = data {
            names.push(p.name.clone());
        }
        names.clone()
    });

    // Act & Assert: Value
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<Vec<String>>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == vec!["Alice"]
    ));

    // Error 1
    tx.send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<Vec<String>>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Error 2
    tx.send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<Vec<String>>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Error 3
    tx.send(StreamItem::Error(FluxionError::stream_error("Error 3")))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<Vec<String>>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // State should still be preserved
    tx.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<Vec<String>>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == vec!["Alice", "Bob"]
    ));

    drop(tx);
    assert_stream_ended::<_, StreamItem<Sequenced<Vec<String>>>>(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_scan_ordered_error_between_values() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.scan_ordered(Vec::<TestData>::new(), |list, data| {
        list.push(data.clone());
        list.clone()
    });

    // Act & Assert: First value
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<Vec<TestData>>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == vec![person_alice()]
    ));

    // Error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<Vec<TestData>>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Second value - list should continue
    tx.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<Vec<TestData>>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == vec![person_alice(), person_bob()]
    ));

    // Another error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<Vec<TestData>>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Third value - list continues
    tx.send(StreamItem::Value(Sequenced::new(person_charlie())))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<Vec<TestData>>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == vec![person_alice(), person_bob(), person_charlie()]
    ));

    drop(tx);
    assert_stream_ended::<_, StreamItem<Sequenced<Vec<TestData>>>>(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_scan_ordered_error_doesnt_reset_complex_state() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.scan_ordered(
        (u32::MAX, u32::MIN),
        |state: &mut (u32, u32), data: &TestData| {
            if let TestData::Person(p) = data {
                state.0 = state.0.min(p.age);
                state.1 = state.1.max(p.age);
            }
            *state
        },
    );

    // Act & Assert: First value (age 25)
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<(u32, u32)>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == (25, 25)
    ));

    // Second value (age 30)
    tx.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<(u32, u32)>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == (25, 30)
    ));

    // Error - state should be preserved
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<(u32, u32)>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Third value (age 35) - min/max continues from (25, 30)
    tx.send(StreamItem::Value(Sequenced::new(person_charlie())))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<(u32, u32)>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == (25, 35)
    ));

    drop(tx);
    assert_stream_ended::<_, StreamItem<Sequenced<(u32, u32)>>>(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_scan_ordered_only_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.scan_ordered(0, |count, _| {
        *count += 1;
        *count
    });

    // Act & Assert: Only send errors
    tx.send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Error(FluxionError::stream_error("Error 3")))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    drop(tx);
    assert_stream_ended::<_, StreamItem<Sequenced<i32>>>(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_scan_ordered_error_with_type_transformation() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.scan_ordered(0, |count: &mut i32, data: &TestData| {
        *count += 1;
        format!("Item #{}: {}", count, data)
    });

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<String>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == "Item #1: Person[name=Alice, age=25]"
    ));

    // Error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<String>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Count should continue from 1
    tx.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<String>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == "Item #2: Person[name=Bob, age=30]"
    ));

    drop(tx);
    assert_stream_ended::<_, StreamItem<Sequenced<String>>>(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_scan_ordered_alternating_values_and_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.scan_ordered(0u32, |total, data| {
        if let TestData::Person(p) = data {
            *total += p.age;
        }
        *total
    });

    // Act & Assert: Alternate between values and errors
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?; // age 25
    assert!(matches!(
        unwrap_stream::<Sequenced<u32>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 25
    ));

    tx.send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<u32>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Value(Sequenced::new(person_bob())))?; // age 30
    assert!(matches!(
        unwrap_stream::<Sequenced<u32>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 55
    )); // 25 + 30

    tx.send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<u32>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Value(Sequenced::new(person_charlie())))?; // age 35
    assert!(matches!(
        unwrap_stream::<Sequenced<u32>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 90
    )); // 55 + 35

    drop(tx);
    assert_stream_ended::<_, StreamItem<Sequenced<u32>>>(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_scan_ordered_error_with_string_accumulation() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.scan_ordered(String::new(), |acc, data| {
        if !acc.is_empty() {
            acc.push_str(", ");
        }
        acc.push_str(&data.to_string());
        acc.clone()
    });

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<String>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == "Person[name=Alice, age=25]"
    ));

    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<String>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // String accumulation should continue
    tx.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    assert!(matches!(
        unwrap_stream::<Sequenced<String>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == "Person[name=Alice, age=25], Person[name=Bob, age=30]"
    ));

    drop(tx);
    assert_stream_ended::<_, StreamItem<Sequenced<String>>>(&mut result, 100).await;

    Ok(())
}
