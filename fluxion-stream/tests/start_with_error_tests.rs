// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::FluxionStream;
use fluxion_test_utils::{helpers::unwrap_stream, test_channel_with_errors, Sequenced};

#[tokio::test]
async fn test_start_with_propagates_errors_from_initial_values() -> anyhow::Result<()> {
    // Arrange
    let (_tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let initial = vec![
        StreamItem::Value(Sequenced::new(1)),
        StreamItem::Error(FluxionError::stream_error("initial error")),
        StreamItem::Value(Sequenced::new(2)),
    ];

    let mut result = FluxionStream::new(stream).start_with(initial);

    // Act & Assert
    let item1 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item1, StreamItem::Value(_)));

    let item2 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item2, StreamItem::Error(_)));

    let item3 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item3, StreamItem::Value(_)));

    Ok(())
}

#[tokio::test]
async fn test_start_with_propagates_errors_from_source_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    let initial = vec![StreamItem::Value(Sequenced::new(1))];

    let mut result = FluxionStream::new(stream).start_with(initial);

    // Act
    tx.send(StreamItem::Value(Sequenced::new(2)))?;
    tx.send(StreamItem::Error(FluxionError::stream_error(
        "source error",
    )))?;
    tx.send(StreamItem::Value(Sequenced::new(3)))?;

    // Assert
    let item1 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item1, StreamItem::Value(_)));

    let item2 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item2, StreamItem::Value(_)));

    let item3 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item3, StreamItem::Error(_)));

    let item4 = unwrap_stream(&mut result, 100).await;
    assert!(matches!(item4, StreamItem::Value(_)));

    Ok(())
}
