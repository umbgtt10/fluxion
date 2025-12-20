// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};

use fluxion_stream::SkipItemsExt;
use fluxion_test_utils::{helpers::unwrap_stream, test_channel_with_errors, Sequenced};

#[tokio::test]
async fn test_skip_propagates_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.skip_items(2);

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(1)))?; // Skipped
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error")))?; // Skipped
    tx.unbounded_send(StreamItem::Value(Sequenced::new(2)))?; // Emitted
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error2")))?; // Emitted

    // Assert - first 2 items skipped (value + error), rest emitted
    let item1 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item1, StreamItem::Value(_)));

    let item2 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item2, StreamItem::Error(_)));

    Ok(())
}

#[tokio::test]
async fn test_skip_counts_errors_as_items() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.skip_items(3);

    // Act - Skip 3 errors
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error1")))?;
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error2")))?;
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error3")))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(1)))?; // This should be emitted

    // Assert - all errors skipped, value emitted
    let item1 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item1, StreamItem::Value(_)));
    if let StreamItem::Value(v) = item1 {
        assert_eq!(v.into_inner(), 1);
    }

    Ok(())
}
