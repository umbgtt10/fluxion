// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Tests for the `on_error` operator implementing Chain of Responsibility pattern.

use fluxion_core::{FluxionError, StreamItem};

use fluxion_stream::OnErrorExt;
use fluxion_test_utils::{assert_no_element_emitted, assert_stream_ended};
use fluxion_test_utils::{test_channel_with_errors, unwrap_stream, unwrap_value, Sequenced};
use parking_lot::Mutex;
use std::sync::Arc;

#[tokio::test]
async fn test_on_error_consumes_all_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.on_error(|_err| {
        true // Consume all errors
    });

    // Act & Assert
    tx.unbounded_send(StreamItem::Value(Sequenced::new(1)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        1
    );

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("user error")))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Value(Sequenced::new(2)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        2
    );

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "stream error",
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Value(Sequenced::new(3)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        3
    );

    drop(tx);
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_on_error_propagates_all_errors() -> anyhow::Result<()> {
    //  Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.on_error(|_err| {
        false // Propagate all errors
    });

    // Act & Assert
    tx.unbounded_send(StreamItem::Value(Sequenced::new(1)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        1
    );

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error1")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.unbounded_send(StreamItem::Value(Sequenced::new(2)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        2
    );

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error2")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    drop(tx);
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_on_error_selective_by_message_content() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.on_error(|err| {
        // Only consume errors containing "user"
        err.to_string().contains("user")
    });

    // Act & Assert
    tx.unbounded_send(StreamItem::Value(Sequenced::new(1)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        1
    );

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("user error")))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "network error",
    )))?;
    // User error consumed, network error propagated
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.unbounded_send(StreamItem::Value(Sequenced::new(2)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        2
    );

    drop(tx);
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_on_error_chain_of_responsibility() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream
        .on_error(|err| err.to_string().contains("validation"))
        .on_error(|err| err.to_string().contains("network"))
        .on_error(|err| {
            // Catch-all handler
            err.to_string().contains("timeout")
        });

    // Act & Assert
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "validation failed",
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "network timeout",
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "timeout occurred",
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Value(Sequenced::new(1)))?;
    // All errors consumed by the chain
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        1
    );

    drop(tx);
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_on_error_with_side_effects() -> anyhow::Result<()> {
    // Arrange
    let error_log = Arc::new(Mutex::new(Vec::new()));
    let error_log_clone = Arc::clone(&error_log);

    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.on_error(move |err| {
        error_log_clone.lock().push(err.to_string());
        true // Consume after logging
    });

    // Act & Assert
    tx.unbounded_send(StreamItem::Value(Sequenced::new(1)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        1
    );

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error1")))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error2")))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Value(Sequenced::new(2)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        2
    );

    drop(tx);
    assert_stream_ended(&mut result, 500).await;

    let logs = error_log.lock();
    assert_eq!(logs.len(), 2);
    assert!(logs[0].contains("error1"));
    assert!(logs[1].contains("error2"));

    Ok(())
}

#[tokio::test]
async fn test_on_error_partial_chain() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream
        .on_error(|err| err.to_string().contains("validation"))
        .on_error(|err| {
            // Don't handle network errors - let them propagate
            err.to_string().contains("mutex")
        });

    // Act & Assert
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "validation failed",
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "network error",
    )))?;
    // Validation and mutex errors consumed, network error propagated
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "mutex poisoned",
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Value(Sequenced::new(42)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        42
    );

    drop(tx);
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_on_error_continues_after_consumed_error() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.on_error(|_| true);

    // Act & Assert
    tx.unbounded_send(StreamItem::Value(Sequenced::new(1)))?;
    // Stream continues normally after each consumed error
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        1
    );

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error1")))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Value(Sequenced::new(2)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        2
    );

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error2")))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Value(Sequenced::new(3)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        3
    );

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error3")))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Value(Sequenced::new(4)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        4
    );

    drop(tx);
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_on_error_multiple_consecutive_errors() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.on_error(|err| err.to_string().contains("user"));

    // Act & Assert
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("user error1")))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("user error2")))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "network error3",
    )))?;
    // First two user errors consumed, network error propagated
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("user error4")))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Value(Sequenced::new(99)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        99
    );

    drop(tx);
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_on_error_empty_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.on_error(|_| true);

    // Act & Assert
    drop(tx);

    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_on_error_only_errors_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.on_error(|_| true);

    // Act & Assert
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error1")))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error2")))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error3")))?;
    assert_no_element_emitted(&mut result, 100).await;

    drop(tx);
    // All errors consumed, stream ends
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_on_error_conditional_based_on_error_content() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.on_error(|err| {
        // Consume errors containing "transient"
        err.to_string().contains("transient")
    });

    // Act & Assert
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "transient network error",
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "permanent failure",
    )))?;
    // Transient error consumed, permanent error propagated
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.unbounded_send(StreamItem::Value(Sequenced::new(1)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        1
    );

    drop(tx);
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_on_error_three_level_chain() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream
        .on_error(|err| err.to_string().contains("level1"))
        .on_error(|err| err.to_string().contains("level2"))
        .on_error(|_| true); // Catch-all

    // Act & Assert
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "level1 error",
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "level2 error",
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("other error")))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Value(Sequenced::new(100)))?;
    // All errors consumed by the three-level chain
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        100
    );

    drop(tx);
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_on_error_preserves_value_order() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.on_error(|_| true);

    // Act & Assert
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(1, 1)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        1
    );

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error")))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(2, 2)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        2
    );

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error")))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(3, 3)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        3
    );

    drop(tx);
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}
