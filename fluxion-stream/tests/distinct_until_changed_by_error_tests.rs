// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::DistinctUntilChangedByExt;
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, unwrap_stream},
    test_channel_with_errors, Sequenced,
};

#[tokio::test]
async fn test_distinct_until_changed_by_propagates_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut distinct = stream.distinct_until_changed_by(|a, b| a == b);

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::new(1)))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        1
    );

    // Send error - should be propagated
    tx.send(StreamItem::Error(FluxionError::stream_error("Test error")))?;
    assert!(matches!(
        unwrap_stream(&mut distinct, 500).await,
        StreamItem::Error(_)
    ));

    // Continue after error
    tx.send(StreamItem::Value(Sequenced::new(2)))?;
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
async fn test_distinct_until_changed_by_error_at_start() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut distinct = stream.distinct_until_changed_by(|a, b| a == b);

    // Act & Assert: Error before any value
    tx.send(StreamItem::Error(FluxionError::stream_error("Early error")))?;
    assert!(matches!(
        unwrap_stream(&mut distinct, 500).await,
        StreamItem::Error(_)
    ));

    // First value after error
    tx.send(StreamItem::Value(Sequenced::new(1)))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        1
    );

    // Duplicate - filtered
    tx.send(StreamItem::Value(Sequenced::new(1)))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_multiple_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut distinct = stream.distinct_until_changed_by(|a, b| a == b);

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::new(1)))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        1
    );

    // Multiple errors in a row
    tx.send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;
    assert!(matches!(
        unwrap_stream(&mut distinct, 500).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;
    assert!(matches!(
        unwrap_stream(&mut distinct, 500).await,
        StreamItem::Error(_)
    ));

    // Continue with values
    tx.send(StreamItem::Value(Sequenced::new(2)))?;
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
async fn test_distinct_until_changed_by_error_between_duplicates() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut distinct = stream.distinct_until_changed_by(|a, b| a == b);

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::new(1)))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        1
    );

    // Duplicate - filtered
    tx.send(StreamItem::Value(Sequenced::new(1)))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut distinct, 500).await,
        StreamItem::Error(_)
    ));

    // Another duplicate after error - should still be filtered
    tx.send(StreamItem::Value(Sequenced::new(1)))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Different value
    tx.send(StreamItem::Value(Sequenced::new(2)))?;
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
async fn test_distinct_until_changed_by_preserves_state_after_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut distinct = stream.distinct_until_changed_by(|a, b| a == b);

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::new(5)))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        5
    );

    // Error should not reset state
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut distinct, 500).await,
        StreamItem::Error(_)
    ));

    // Same value as before error - should be filtered
    tx.send(StreamItem::Value(Sequenced::new(5)))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Different value - emitted
    tx.send(StreamItem::Value(Sequenced::new(10)))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        10
    );

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_alternating_errors_and_values() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut distinct = stream.distinct_until_changed_by(|a, b| a == b);

    // Act & Assert: Interleave values and errors
    tx.send(StreamItem::Value(Sequenced::new(1)))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        1
    );

    tx.send(StreamItem::Error(FluxionError::stream_error("E1")))?;
    assert!(matches!(
        unwrap_stream(&mut distinct, 500).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Value(Sequenced::new(2)))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        2
    );

    tx.send(StreamItem::Error(FluxionError::stream_error("E2")))?;
    assert!(matches!(
        unwrap_stream(&mut distinct, 500).await,
        StreamItem::Error(_)
    ));

    tx.send(StreamItem::Value(Sequenced::new(3)))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        3
    );

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_error_only_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut distinct = stream.distinct_until_changed_by(|a, b| a == b);

    // Act & Assert: Only send errors
    for i in 0..5 {
        tx.send(StreamItem::Error(FluxionError::stream_error(format!(
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
    let (tx, stream) = test_channel_with_errors::<Sequenced<String>>();
    // Case-insensitive comparison
    let mut distinct =
        stream.distinct_until_changed_by(|a, b| a.to_lowercase() == b.to_lowercase());

    // Act & Assert
    tx.send(StreamItem::Value(Sequenced::new("hello".to_string())))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        "hello"
    );

    // Same (case-insensitive) - filtered
    tx.send(StreamItem::Value(Sequenced::new("HELLO".to_string())))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut distinct, 500).await,
        StreamItem::Error(_)
    ));

    // Still same (case-insensitive) - filtered
    tx.send(StreamItem::Value(Sequenced::new("HeLLo".to_string())))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Different value
    tx.send(StreamItem::Value(Sequenced::new("world".to_string())))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        "world"
    );

    Ok(())
}
