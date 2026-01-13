// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::DistinctUntilChangedByExt;
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    person::Person,
    test_channel_with_errors,
    test_data::{person_alice, person_bob, person_charlie, TestData},
    Sequenced,
};

#[tokio::test]
async fn test_distinct_until_changed_by_propagates_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut distinct = stream.distinct_until_changed_by(|a, b| match (a, b) {
        (TestData::Person(p1), TestData::Person(p2)) => p1.age == p2.age,
        _ => false,
    });

    // Act & Assert
    tx.try_send(StreamItem::Value(Sequenced::new(TestData::Person(
        Person::new("Alice".to_string(), 25),
    ))))?;
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.age, 25);
    } else {
        panic!("Expected Person");
    }

    // Send error - should be propagated
    tx.try_send(StreamItem::Error(FluxionError::stream_error("Test error")))?;
    assert!(matches!(
        unwrap_stream(&mut distinct, 500).await,
        StreamItem::Error(_)
    ));

    // Continue after error
    tx.try_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.age, 30);
    } else {
        panic!("Expected Person");
    }

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_error_at_start() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut distinct = stream.distinct_until_changed_by(|a, b| match (a, b) {
        (TestData::Person(p1), TestData::Person(p2)) => p1.age == p2.age,
        _ => false,
    });

    // Act & Assert: Error before any value
    tx.try_send(StreamItem::Error(FluxionError::stream_error("Early error")))?;
    assert!(matches!(
        unwrap_stream(&mut distinct, 500).await,
        StreamItem::Error(_)
    ));

    // First value after error
    tx.try_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.age, 25);
    } else {
        panic!("Expected Person");
    }

    // Duplicate - filtered
    tx.try_send(StreamItem::Value(Sequenced::new(TestData::Person(
        Person::new("Alice Updated".to_string(), 25),
    ))))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_multiple_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut distinct = stream.distinct_until_changed_by(|a, b| match (a, b) {
        (TestData::Person(p1), TestData::Person(p2)) => p1.age == p2.age,
        _ => false,
    });

    // Act & Assert
    tx.try_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.age, 25);
    } else {
        panic!("Expected Person");
    }

    // Multiple errors in a row
    tx.try_send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    assert!(matches!(
        unwrap_stream(&mut distinct, 500).await,
        StreamItem::Error(_)
    ));

    tx.try_send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;
    assert!(matches!(
        unwrap_stream(&mut distinct, 500).await,
        StreamItem::Error(_)
    ));

    // Continue with values
    tx.try_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.age, 30);
    } else {
        panic!("Expected Person");
    }

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_error_between_duplicates() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut distinct = stream.distinct_until_changed_by(|a, b| match (a, b) {
        (TestData::Person(p1), TestData::Person(p2)) => p1.age == p2.age,
        _ => false,
    });

    // Act & Assert
    tx.try_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.age, 25);
    } else {
        panic!("Expected Person");
    }

    // Duplicate - filtered
    tx.try_send(StreamItem::Value(Sequenced::new(TestData::Person(
        Person::new("Alice Updated".to_string(), 25),
    ))))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Error
    tx.try_send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut distinct, 500).await,
        StreamItem::Error(_)
    ));

    // Another duplicate after error - should still be filtered
    tx.try_send(StreamItem::Value(Sequenced::new(TestData::Person(
        Person::new("Alice v3".to_string(), 25),
    ))))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Different value
    tx.try_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.age, 30);
    } else {
        panic!("Expected Person");
    }

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_preserves_state_after_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut distinct = stream.distinct_until_changed_by(|a, b| match (a, b) {
        (TestData::Person(p1), TestData::Person(p2)) => p1.age == p2.age,
        _ => false,
    });

    // Act & Assert
    tx.try_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.age, 25);
    } else {
        panic!("Expected Person");
    }

    // Error should not reset state
    tx.try_send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut distinct, 500).await,
        StreamItem::Error(_)
    ));

    // Same value as before error - should be filtered
    tx.try_send(StreamItem::Value(Sequenced::new(TestData::Person(
        Person::new("Alice Updated".to_string(), 25),
    ))))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Different value - emitted
    tx.try_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.age, 30);
    } else {
        panic!("Expected Person");
    }

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_alternating_errors_and_values() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut distinct = stream.distinct_until_changed_by(|a, b| match (a, b) {
        (TestData::Person(p1), TestData::Person(p2)) => p1.age == p2.age,
        _ => false,
    });

    // Act & Assert: Interleave values and errors
    tx.try_send(StreamItem::Value(Sequenced::new(person_alice())))?;
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.age, 25);
    } else {
        panic!("Expected Person");
    }

    tx.try_send(StreamItem::Error(FluxionError::stream_error("E1")))?;
    assert!(matches!(
        unwrap_stream(&mut distinct, 500).await,
        StreamItem::Error(_)
    ));

    tx.try_send(StreamItem::Value(Sequenced::new(person_bob())))?;
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.age, 30);
    } else {
        panic!("Expected Person");
    }

    tx.try_send(StreamItem::Error(FluxionError::stream_error("E2")))?;
    assert!(matches!(
        unwrap_stream(&mut distinct, 500).await,
        StreamItem::Error(_)
    ));

    tx.try_send(StreamItem::Value(Sequenced::new(person_charlie())))?;
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.age, 35);
    } else {
        panic!("Expected Person");
    }

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_error_only_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let mut distinct = stream.distinct_until_changed_by(|a, b| match (a, b) {
        (TestData::Person(p1), TestData::Person(p2)) => p1.age == p2.age,
        _ => false,
    });

    // Act & Assert: Only send errors
    for i in 0..5 {
        tx.try_send(StreamItem::Error(FluxionError::stream_error(format!(
            "Error {}",
            i
        ))))?;
        assert!(matches!(
            unwrap_stream(&mut distinct, 500).await,
            StreamItem::Error(_)
        ));
    }

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_custom_comparer_with_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<TestData>>();
    // Case-insensitive name comparison
    let mut distinct = stream.distinct_until_changed_by(|a, b| match (a, b) {
        (TestData::Person(p1), TestData::Person(p2)) => {
            p1.name.to_lowercase() == p2.name.to_lowercase()
        }
        _ => false,
    });

    // Act & Assert
    tx.try_send(StreamItem::Value(Sequenced::new(TestData::Person(
        Person::new("hello".to_string(), 25),
    ))))?;
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.name, "hello");
    } else {
        panic!("Expected Person");
    }

    // Same (case-insensitive) - filtered
    tx.try_send(StreamItem::Value(Sequenced::new(TestData::Person(
        Person::new("HELLO".to_string(), 25),
    ))))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Error
    tx.try_send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut distinct, 500).await,
        StreamItem::Error(_)
    ));

    // Still same (case-insensitive) - filtered
    tx.try_send(StreamItem::Value(Sequenced::new(TestData::Person(
        Person::new("HeLLo".to_string(), 25),
    ))))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Different value
    tx.try_send(StreamItem::Value(Sequenced::new(TestData::Person(
        Person::new("world".to_string(), 30),
    ))))?;
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.name, "world");
    } else {
        panic!("Expected Person");
    }

    Ok(())
}
