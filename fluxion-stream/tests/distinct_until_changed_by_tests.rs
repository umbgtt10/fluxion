// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::HasTimestamp;
use fluxion_stream::DistinctUntilChangedByExt;
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, assert_stream_ended, unwrap_stream},
    test_channel, Sequenced,
};

#[derive(Clone, Debug, PartialEq)]
struct User {
    id: u32,
    name: String,
    version: u32,
}

#[tokio::test]
async fn test_distinct_until_changed_by_field_comparison() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<User>>();
    // Compare by ID only, ignoring name and version
    let mut distinct = stream.distinct_until_changed_by(|a, b| a.id == b.id);

    // Act & Assert: First value always emitted
    tx.send(Sequenced::new(User {
        id: 1,
        name: "Alice".into(),
        version: 1,
    }))?;
    let user = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    assert_eq!(user.id, 1);
    assert_eq!(user.name, "Alice");

    // Same ID, different name/version - filtered
    tx.send(Sequenced::new(User {
        id: 1,
        name: "Alice Updated".into(),
        version: 2,
    }))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Different ID - emitted
    tx.send(Sequenced::new(User {
        id: 2,
        name: "Bob".into(),
        version: 1,
    }))?;
    let user = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    assert_eq!(user.id, 2);

    // Same ID as previous - filtered
    tx.send(Sequenced::new(User {
        id: 2,
        name: "Bob v2".into(),
        version: 2,
    }))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Back to ID 1 - emitted (different from previous ID 2)
    tx.send(Sequenced::new(User {
        id: 1,
        name: "Alice v3".into(),
        version: 3,
    }))?;
    let user = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    assert_eq!(user.id, 1);
    assert_eq!(user.version, 3);

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_case_insensitive() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<String>>();
    // Case-insensitive comparison
    let mut distinct =
        stream.distinct_until_changed_by(|a, b| a.to_lowercase() == b.to_lowercase());

    // Act & Assert
    tx.send(Sequenced::new("hello".to_string()))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        "hello"
    );

    // Same string, different case - filtered
    tx.send(Sequenced::new("HELLO".to_string()))?;
    tx.send(Sequenced::new("HeLLo".to_string()))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Different string - emitted
    tx.send(Sequenced::new("world".to_string()))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        "world"
    );

    // Same string, different case - filtered
    tx.send(Sequenced::new("WORLD".to_string()))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Back to previous value with different case - filtered
    tx.send(Sequenced::new("World".to_string()))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Truly different value
    tx.send(Sequenced::new("rust".to_string()))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        "rust"
    );

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_threshold() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<f64>>();
    // Only emit if difference is >= 0.5
    let mut distinct = stream.distinct_until_changed_by(|a, b| (a - b).abs() < 0.5);

    // Act & Assert
    tx.send(Sequenced::new(1.0))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        1.0
    );

    // Small change - filtered
    tx.send(Sequenced::new(1.2))?;
    tx.send(Sequenced::new(1.4))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Large change - emitted
    tx.send(Sequenced::new(2.0))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        2.0
    );

    // Small change from 2.0 - filtered
    tx.send(Sequenced::new(2.3))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Large change - emitted
    tx.send(Sequenced::new(3.0))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        3.0
    );

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_custom_logic() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();
    // Consider values "same" if they have the same parity (both even or both odd)
    let mut distinct = stream.distinct_until_changed_by(|a, b| a % 2 == b % 2);

    // Act & Assert
    tx.send(Sequenced::new(1))?; // Odd
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        1
    );

    // Another odd - filtered
    tx.send(Sequenced::new(3))?;
    tx.send(Sequenced::new(5))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Even - emitted (different parity)
    tx.send(Sequenced::new(2))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        2
    );

    // Another even - filtered
    tx.send(Sequenced::new(4))?;
    tx.send(Sequenced::new(6))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Back to odd - emitted
    tx.send(Sequenced::new(7))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        7
    );

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_empty_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();
    let mut distinct = stream.distinct_until_changed_by(|a, b| a == b);

    // Act
    drop(tx);

    // Assert
    assert_stream_ended(&mut distinct, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_single_value() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();
    let mut distinct = stream.distinct_until_changed_by(|a, b| a == b);

    // Act
    tx.send(Sequenced::new(42))?;
    drop(tx);

    // Assert
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        42
    );
    assert_stream_ended(&mut distinct, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_preserves_timestamps() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();
    let mut distinct = stream.distinct_until_changed_by(|a, b| a == b);

    // Act & Assert
    tx.send(Sequenced::with_timestamp(1, 100))?;
    let first = unwrap_stream(&mut distinct, 500).await.unwrap();
    assert_eq!(first.timestamp(), 100);

    // Duplicate - filtered
    tx.send(Sequenced::with_timestamp(1, 200))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // New value with specific timestamp
    tx.send(Sequenced::with_timestamp(2, 300))?;
    let second = unwrap_stream(&mut distinct, 500).await.unwrap();
    assert_eq!(second.timestamp(), 300);
    assert_eq!(second.into_inner(), 2);

    // Another new value
    tx.send(Sequenced::with_timestamp(3, 400))?;
    let third = unwrap_stream(&mut distinct, 500).await.unwrap();
    assert_eq!(third.timestamp(), 400);

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_many_duplicates() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<String>>();
    // Compare by length
    let mut distinct = stream.distinct_until_changed_by(|a, b| a.len() == b.len());

    // Act & Assert
    tx.send(Sequenced::new("a".to_string()))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        "a"
    );

    // Many single-char strings - all filtered
    for c in 'b'..='z' {
        tx.send(Sequenced::new(c.to_string()))?;
    }
    assert_no_element_emitted(&mut distinct, 100).await;

    // Two-char string - emitted
    tx.send(Sequenced::new("aa".to_string()))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        "aa"
    );

    // More two-char strings - filtered
    for i in 0..10 {
        tx.send(Sequenced::new(format!("x{}", i)))?;
    }
    assert_no_element_emitted(&mut distinct, 100).await;

    // Three-char string - emitted
    tx.send(Sequenced::new("abc".to_string()))?;
    let result = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    assert_eq!(result.len(), 3);

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_alternating() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();
    let mut distinct = stream.distinct_until_changed_by(|a, b| a == b);

    // Act & Assert: Alternating values should all be emitted
    for i in 0..10 {
        tx.send(Sequenced::new(i % 2))?;
        let value = unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner();
        assert_eq!(value, i % 2);
    }

    Ok(())
}
