// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::TakeItemsExt;
use fluxion_test_utils::{
    helpers::{assert_stream_ended, test_channel_with_errors, unwrap_stream},
    sequenced::Sequenced,
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
    tx.unbounded_send(StreamItem::Value(Sequenced::new(3)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));
    assert_stream_ended(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_counts_errors_as_items() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.take_items(2);

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error1")))?;
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error2")))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::new(1)))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));
    assert_stream_ended(&mut result, 100).await;

    Ok(())
}
