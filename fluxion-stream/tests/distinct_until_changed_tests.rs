// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::HasTimestamp;
use fluxion_stream::{DistinctUntilChangedExt, FluxionStream};
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, assert_stream_ended, unwrap_stream},
    test_channel, Sequenced,
};

#[tokio::test]
async fn test_distinct_until_changed_basic() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();
    let mut distinct = stream.distinct_until_changed();

    // Act & Assert: First value always emitted
    tx.send(Sequenced::new(1))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        1
    );

    // Duplicate - filtered
    tx.send(Sequenced::new(1))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // New value - emitted
    tx.send(Sequenced::new(2))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        2
    );

    // Another duplicate - filtered
    tx.send(Sequenced::new(2))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // New value - emitted
    tx.send(Sequenced::new(3))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        3
    );

    // Return to previous value - emitted (different from 3)
    tx.send(Sequenced::new(2))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        2
    );

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_boolean_toggle() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<bool>>();
    let mut distinct = stream.distinct_until_changed();

    // Act & Assert: Initial state
    tx.send(Sequenced::new(false))?;
    assert!(!unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner());

    // Same state - filtered
    tx.send(Sequenced::new(false))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Toggle to true
    tx.send(Sequenced::new(true))?;
    assert!(unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner());

    // Same state - filtered
    tx.send(Sequenced::new(true))?;
    tx.send(Sequenced::new(true))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Toggle back to false
    tx.send(Sequenced::new(false))?;
    assert!(!unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner());

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_many_duplicates() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();
    let mut distinct = stream.distinct_until_changed();

    // Act: Send many duplicates
    tx.send(Sequenced::new(42))?;
    for _ in 0..100 {
        tx.send(Sequenced::new(42))?;
    }
    tx.send(Sequenced::new(99))?;

    // Assert: Only two values emitted (42 and 99)
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        42
    );
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        99
    );
    assert_no_element_emitted(&mut distinct, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_alternating() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();
    let mut distinct = stream.distinct_until_changed();

    // Act: Alternating values - all should be emitted
    tx.send(Sequenced::new(1))?;
    tx.send(Sequenced::new(2))?;
    tx.send(Sequenced::new(1))?;
    tx.send(Sequenced::new(2))?;
    tx.send(Sequenced::new(1))?;

    // Assert: All values emitted (each different from previous)
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        1
    );
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        2
    );
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        1
    );
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        2
    );
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        1
    );

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_string_values() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<String>>();
    let mut distinct = stream.distinct_until_changed();

    // Act & Assert
    tx.send(Sequenced::new("hello".to_string()))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        "hello"
    );

    tx.send(Sequenced::new("hello".to_string()))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    tx.send(Sequenced::new("world".to_string()))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        "world"
    );

    tx.send(Sequenced::new("world".to_string()))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_fresh_timestamps() -> anyhow::Result<()> {
    use std::time::Duration;

    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();
    let mut distinct = stream.distinct_until_changed();

    // Act: Send first value
    tx.send(Sequenced::new(1))?;
    let first = unwrap_stream(&mut distinct, 500).await.unwrap();
    let ts1 = first.timestamp();

    // Wait a bit
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Send duplicate (should be filtered)
    tx.send(Sequenced::new(1))?;

    // Wait a bit more
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Send new value
    tx.send(Sequenced::new(2))?;
    let second = unwrap_stream(&mut distinct, 500).await.unwrap();
    let ts2 = second.timestamp();

    // Assert: Timestamps are different (fresh generated)
    assert!(
        ts2 > ts1,
        "Expected fresh timestamp on distinct value emission"
    );

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_with_filter_ordered() -> anyhow::Result<()> {
    // Arrange: Compose with filter_ordered
    let (tx, stream) = test_channel::<Sequenced<i32>>();
    let mut composed = FluxionStream::new(stream)
        .distinct_until_changed()
        .filter_ordered(|&x| x > 0);

    // Act & Assert
    tx.send(Sequenced::new(-1))?; // Emitted by distinct, filtered by filter_ordered
    tx.send(Sequenced::new(-1))?; // Filtered by distinct
    tx.send(Sequenced::new(5))?; // Emitted by both
    assert_eq!(
        unwrap_stream(&mut composed, 500)
            .await
            .unwrap()
            .into_inner(),
        5
    );

    tx.send(Sequenced::new(5))?; // Filtered by distinct
    assert_no_element_emitted(&mut composed, 100).await;

    tx.send(Sequenced::new(10))?; // Emitted by both
    assert_eq!(
        unwrap_stream(&mut composed, 500)
            .await
            .unwrap()
            .into_inner(),
        10
    );

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_empty_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();
    let mut distinct = stream.distinct_until_changed();

    // Act: Close stream without sending anything
    drop(tx);

    // Assert: Stream ends with no values
    assert_stream_ended(&mut distinct, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_single_value() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();
    let mut distinct = stream.distinct_until_changed();

    // Act
    tx.send(Sequenced::new(42))?;

    // Assert: Single value emitted
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        42
    );

    drop(tx);
    assert_stream_ended(&mut distinct, 100).await;

    Ok(())
}
