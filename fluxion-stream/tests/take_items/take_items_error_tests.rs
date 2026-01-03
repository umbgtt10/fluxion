// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use fluxion_core::{FluxionError, StreamItem};

use fluxion_stream::TakeItemsExt;
use fluxion_test_utils::{
    helpers::{assert_stream_ended, unwrap_stream},
    test_channel_with_errors, Sequenced,
};

#[tokio::test]
async fn test_take_propagates_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.take_items(3);

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(1)))?;
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error")))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(2)))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(3)))?; // This should not be emitted

    // Assert - take counts errors as items
    let item1 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item1, StreamItem::Value(_)));

    let item2 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item2, StreamItem::Error(_)));

    let item3 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item3, StreamItem::Value(_)));

    // Stream should end after 3 items (including error)
    assert_stream_ended(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_counts_errors_as_items() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.take_items(2);

    // Act - Send 2 errors, no values
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error1")))?;
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error2")))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(1)))?; // Should not be emitted

    // Assert - both errors emitted, then stream ends
    let item1 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item1, StreamItem::Error(_)));

    let item2 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item2, StreamItem::Error(_)));

    assert_stream_ended(&mut result, 100).await;

    Ok(())
}
