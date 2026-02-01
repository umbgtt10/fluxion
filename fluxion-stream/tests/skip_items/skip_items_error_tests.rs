// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::SkipItemsExt;
use fluxion_test_utils::{
    helpers::{test_channel_with_errors, unwrap_stream},
    sequenced::Sequenced,
};

#[tokio::test]
async fn test_skip_propagates_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.skip_items(2);

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(1)))?;
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error")))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(2)))?;
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error2")))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    Ok(())
}

#[tokio::test]
async fn test_skip_counts_errors_as_items() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.skip_items(3);

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error1")))?;
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error2")))?;
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error3")))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(1)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(ref v) if v.value == 1
    ));

    Ok(())
}
