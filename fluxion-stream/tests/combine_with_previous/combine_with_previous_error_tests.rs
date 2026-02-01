// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `combine_with_previous` operator.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::CombineWithPreviousExt;
use fluxion_test_utils::{
    helpers::{test_channel_with_errors, unwrap_stream},
    sequenced::Sequenced,
};

#[tokio::test]
async fn test_combine_with_previous_propagates_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = stream.combine_with_previous();

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 2)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(4, 4)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_combine_with_previous_error_at_first_item() -> anyhow::Result<()> {
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = stream.combine_with_previous();

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("First error")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ),);

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 2)))?;

    // Assert
    assert!(
        matches!(unwrap_stream(&mut result, 100).await, StreamItem::Value(_)),
        "Should continue"
    );

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_combine_with_previous_multiple_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = stream.combine_with_previous();

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 1")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(3, 3)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error 2")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(5, 5)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_combine_with_previous_preserves_pairing_after_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = stream.combine_with_previous();

    // Act & Assert
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(10, 1)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(30, 3)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(40, 4)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_combine_with_previous_single_item_stream_with_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let mut result = stream.combine_with_previous();

    // Act & Assert
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    drop(tx);

    Ok(())
}
