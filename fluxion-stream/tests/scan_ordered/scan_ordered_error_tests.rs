// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `scan_ordered` operator.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::ScanOrderedExt;
use fluxion_test_utils::{
    helpers::{test_channel_with_errors, unwrap_stream},
    sequenced::Sequenced,
    test_data::{person_alice, person_bob, person_charlie, TestData},
};

#[tokio::test]
async fn test_scan_ordered_propagates_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut result = stream.scan_ordered(0, |count, _| {
        *count += 1;
        *count
    });

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 1
    ));

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Test error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 2
    ));

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

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Initial error",
    )))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 101
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 102
    ));

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

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<Vec<String>>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == vec!["Alice"]
    ));

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<Vec<String>>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<Vec<String>>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 3")))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<Vec<String>>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<Vec<String>>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == vec!["Alice", "Bob"]
    ));

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

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<Vec<TestData>>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == vec![person_alice()]
    ));

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<Vec<TestData>>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<Vec<TestData>>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == vec![person_alice(), person_bob()]
    ));

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<Vec<TestData>>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_charlie())))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<Vec<TestData>>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == vec![person_alice(), person_bob(), person_charlie()]
    ));

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

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<(u32, u32)>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == (25, 25)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<(u32, u32)>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == (25, 30)
    ));

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<(u32, u32)>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_charlie())))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<(u32, u32)>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == (25, 35)
    ));

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

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 3")))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<i32>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

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

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<String>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == "Item #1: Person[name=Alice, age=25]"
    ));

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<String>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<String>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == "Item #2: Person[name=Bob, age=30]"
    ));

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

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<u32>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 25
    ));

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<u32>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<u32>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 55
    ));

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<u32>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_charlie())))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<u32>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 90
    ));

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

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_alice())))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<String>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == "Person[name=Alice, age=25]"
    ));

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<String>, _>(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(person_bob())))?;

    // Assert
    assert!(matches!(
        unwrap_stream::<Sequenced<String>, _>(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == "Person[name=Alice, age=25], Person[name=Bob, age=30]"
    ));

    Ok(())
}
