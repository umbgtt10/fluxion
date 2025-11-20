// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error propagation tests for `map_ordered` operator.

use fluxion_core::{FluxionError, StreamItem};
use fluxion_stream::FluxionStream;
use fluxion_test_utils::{Timestamped, test_channel_with_errors, unwrap_stream};
use fluxion_core::Timestamped as TimestampedTrait;

#[tokio::test]
async fn test_map_ordered_propagates_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Timestamped<i32>>();

    // Use combine_with_previous then map to string
    let mut result = FluxionStream::new(stream)
        .combine_with_previous()
        .map_ordered(|x| format!("Current: {}", x.current.inner()));

    // Act & Assert: Send value
    tx.send(StreamItem::Value(Timestamped::new(1)))?;
    assert!(
        matches!(unwrap_stream(&mut result, 100).await, StreamItem::Value(ref v) if v == "Current: 1")
    );

    // Send error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Continue
    tx.send(StreamItem::Value(Timestamped::new(2)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(_)
    ));

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_map_ordered_transformation_after_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Timestamped<i32>>();

    // Use combine_with_previous then map
    let mut result = FluxionStream::new(stream)
        .combine_with_previous()
        .map_ordered(|x| x.current.inner() * 2);

    // Act & AssertSend values
    tx.send(StreamItem::Value(Timestamped::with_timestamp(10, 1)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(20)
    ));

    tx.send(StreamItem::Value(Timestamped::with_timestamp(20, 2)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(40)
    ));

    // Send error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Continue after error
    tx.send(StreamItem::Value(Timestamped::with_timestamp(40, 4)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(80)
    ));

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_map_ordered_preserves_error_passthrough() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Timestamped<i32>>();

    let mut result = FluxionStream::new(stream)
        .combine_with_previous()
        .map_ordered(|x| x.current.inner() * 100);

    // Act & Assert: Error immediately
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;

    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ),);

    // Continue with value
    tx.send(StreamItem::Value(Timestamped::with_timestamp(2, 2)))?;

    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(200)
    ));

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_map_ordered_chain_after_error() -> anyhow::Result<()> {
    let (tx, stream) = test_channel_with_errors::<Timestamped<i32>>();

    // Chain combine_with_previous and map
    let mut result = FluxionStream::new(stream)
        .combine_with_previous()
        .map_ordered(|x| x.current.inner() * 2);

    // Act & Assert: Send value
    tx.send(StreamItem::Value(Timestamped::with_timestamp(5, 1)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(10)
    ));

    // Act & Assert: Send error
    tx.send(StreamItem::Error(FluxionError::stream_error("Error")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Continue
    tx.send(StreamItem::Value(Timestamped::with_timestamp(15, 3)))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Value(30)
    ));

    drop(tx);

    Ok(())
}


