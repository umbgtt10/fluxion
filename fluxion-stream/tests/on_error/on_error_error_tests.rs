// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error behavior tests for the `on_error` handler itself.
//!
//! These tests verify the behavior of the `on_error` operator when the handler
//! function has specific behaviors like stateful tracking, conditional logic,
//! and interaction with different error types.

use fluxion_core::{FluxionError, StreamItem};

use fluxion_stream::OnErrorExt;
use fluxion_test_utils::{
    assert_no_element_emitted, assert_stream_ended, test_channel_with_errors, unwrap_stream,
    unwrap_value, Sequenced,
};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[tokio::test]
async fn test_on_error_handler_receives_correct_error_type() -> anyhow::Result<()> {
    // Arrange
    let received_errors = Arc::new(Mutex::new(Vec::new()));
    let received_errors_clone = Arc::clone(&received_errors);

    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.on_error(move |err| {
        received_errors_clone.lock().push(err.to_string());
        true // Consume all
    });

    // Act
    tx.unbounded_send(StreamItem::Value(Sequenced::new(1)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        1
    );

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Stream processing failed",
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "Another stream error",
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Value(Sequenced::new(2)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        2
    );

    drop(tx);
    assert_stream_ended(&mut result, 500).await;

    // Assert
    let errors = received_errors.lock();
    assert_eq!(errors.len(), 2);
    assert!(errors[0].contains("Stream processing failed"));
    assert!(errors[1].contains("Another stream error"));

    Ok(())
}

#[tokio::test]
async fn test_on_error_handler_with_counter_state() -> anyhow::Result<()> {
    // Arrange: Handler that counts errors and only consumes the first 2
    let error_count = Arc::new(AtomicUsize::new(0));
    let error_count_clone = Arc::clone(&error_count);

    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.on_error(move |_err| {
        let count = error_count_clone.fetch_add(1, Ordering::SeqCst);
        count < 2 // Consume first 2 errors, propagate rest
    });

    // Act & Assert
    // First error - consumed (count becomes 1)
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error1")))?;
    assert_no_element_emitted(&mut result, 100).await;

    // Second error - consumed (count becomes 2)
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error2")))?;
    assert_no_element_emitted(&mut result, 100).await;

    // Third error - propagated (count is now 2, not < 2)
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error3")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Fourth error - also propagated
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error4")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Values still work
    tx.unbounded_send(StreamItem::Value(Sequenced::new(42)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        42
    );

    drop(tx);
    assert_stream_ended(&mut result, 500).await;

    assert_eq!(error_count.load(Ordering::SeqCst), 4);

    Ok(())
}

#[tokio::test]
async fn test_on_error_handler_alternating_decision() -> anyhow::Result<()> {
    // Arrange: Handler that alternates between consuming and propagating
    let toggle = Arc::new(AtomicUsize::new(0));
    let toggle_clone = Arc::clone(&toggle);

    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.on_error(move |_err| {
        let count = toggle_clone.fetch_add(1, Ordering::SeqCst);
        count.is_multiple_of(2) // Consume even-numbered calls, propagate odd
    });

    // Act & Assert
    // First error (call 0) - consumed
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error0")))?;
    assert_no_element_emitted(&mut result, 100).await;

    // Second error (call 1) - propagated
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error1")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Third error (call 2) - consumed
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error2")))?;
    assert_no_element_emitted(&mut result, 100).await;

    // Fourth error (call 3) - propagated
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error3")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    drop(tx);
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_on_error_handler_decision_based_on_error_content() -> anyhow::Result<()> {
    // Arrange: Handler that makes different decisions based on error content
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.on_error(|err| {
        let msg = err.to_string();
        // Consume: transient, retry, timeout
        // Propagate: fatal, permanent, critical
        msg.contains("transient") || msg.contains("retry") || msg.contains("timeout")
    });

    // Act & Assert
    tx.unbounded_send(StreamItem::Value(Sequenced::new(1)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        1
    );

    // Transient error - consumed
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "transient network error",
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    // Fatal error - propagated
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "fatal: database corrupted",
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Retry error - consumed
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "retry: connection reset",
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    // Permanent error - propagated
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "permanent: resource not found",
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Timeout error - consumed
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "timeout: operation timed out",
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    // Critical error - propagated
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "critical: out of memory",
    )))?;
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
async fn test_on_error_handler_tracks_error_history() -> anyhow::Result<()> {
    // Arrange: Handler that tracks history and makes decisions based on patterns
    let history = Arc::new(Mutex::new(Vec::<String>::new()));
    let history_clone = Arc::clone(&history);

    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.on_error(move |err| {
        let msg = err.to_string();
        let mut hist = history_clone.lock();
        hist.push(msg.clone());

        // If we've seen 3+ errors, start propagating
        hist.len() < 3
    });

    // Act & Assert
    // First error - consumed (history: 1)
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error1")))?;
    assert_no_element_emitted(&mut result, 100).await;

    // Second error - consumed (history: 2)
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error2")))?;
    assert_no_element_emitted(&mut result, 100).await;

    // Third error - propagated (history: 3, >= 3)
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error3")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Fourth error - also propagated
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error4")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    drop(tx);
    assert_stream_ended(&mut result, 500).await;

    // Verify history
    let hist = history.lock();
    assert_eq!(hist.len(), 4);

    Ok(())
}

#[tokio::test]
async fn test_on_error_handler_with_rate_limiting_behavior() -> anyhow::Result<()> {
    // Arrange: Handler that consumes up to N errors per "window" (simulated)
    // In this test, we consume up to 2 consecutive errors, then propagate until a value comes
    let consecutive_errors = Arc::new(AtomicUsize::new(0));
    let consecutive_errors_clone = Arc::clone(&consecutive_errors);

    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();

    // We need to track values separately to reset counter
    // For simplicity, we'll just use the counter behavior
    let mut result = stream.on_error(move |_err| {
        let count = consecutive_errors_clone.fetch_add(1, Ordering::SeqCst);
        count < 2 // Allow first 2 consecutive errors
    });

    // Act & Assert
    // First error - consumed
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error1")))?;
    assert_no_element_emitted(&mut result, 100).await;

    // Second error - consumed
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error2")))?;
    assert_no_element_emitted(&mut result, 100).await;

    // Third error - propagated (burst limit reached)
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error3")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Value resets nothing in this simple implementation, but stream continues
    tx.unbounded_send(StreamItem::Value(Sequenced::new(42)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        42
    );

    // Next error - propagated (counter still high)
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error4")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    drop(tx);
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_on_error_handler_never_consumes() -> anyhow::Result<()> {
    // Arrange: Handler that always returns false (never consumes)
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.on_error(|_| false);

    // Act & Assert
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error1")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error2")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.unbounded_send(StreamItem::Value(Sequenced::new(1)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        1
    );

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error3")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    drop(tx);
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_on_error_handler_always_consumes() -> anyhow::Result<()> {
    // Arrange: Handler that always returns true (always consumes)
    let error_count = Arc::new(AtomicUsize::new(0));
    let error_count_clone = Arc::clone(&error_count);

    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.on_error(move |_| {
        error_count_clone.fetch_add(1, Ordering::SeqCst);
        true
    });

    // Act & Assert
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error1")))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error2")))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error3")))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Value(Sequenced::new(1)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        1
    );

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error4")))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error5")))?;
    assert_no_element_emitted(&mut result, 100).await;

    drop(tx);
    assert_stream_ended(&mut result, 500).await;

    assert_eq!(error_count.load(Ordering::SeqCst), 5);

    Ok(())
}

#[tokio::test]
async fn test_on_error_handler_with_external_flag() -> anyhow::Result<()> {
    // Arrange: Handler behavior controlled by external flag
    let should_consume = Arc::new(AtomicUsize::new(1)); // 1 = consume, 0 = propagate
    let should_consume_clone = Arc::clone(&should_consume);

    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.on_error(move |_| should_consume_clone.load(Ordering::SeqCst) == 1);

    // Act & Assert
    // Initially consuming
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error1")))?;
    assert_no_element_emitted(&mut result, 100).await;

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error2")))?;
    assert_no_element_emitted(&mut result, 100).await;

    // Change flag to propagate
    should_consume.store(0, Ordering::SeqCst);

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error3")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error4")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // Change flag back to consume
    should_consume.store(1, Ordering::SeqCst);

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("error5")))?;
    assert_no_element_emitted(&mut result, 100).await;

    drop(tx);
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_on_error_multiple_errors_same_message() -> anyhow::Result<()> {
    // Arrange: Multiple errors with identical messages
    let seen_messages = Arc::new(Mutex::new(Vec::<String>::new()));
    let seen_messages_clone = Arc::clone(&seen_messages);

    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.on_error(move |err| {
        let msg = err.to_string();
        let mut seen = seen_messages_clone.lock();
        // Consume first occurrence of each message, propagate duplicates
        if seen.contains(&msg) {
            false
        } else {
            seen.push(msg);
            true
        }
    });

    // Act & Assert
    // First "connection lost" - consumed
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "connection lost",
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    // First "timeout" - consumed
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("timeout")))?;
    assert_no_element_emitted(&mut result, 100).await;

    // Second "connection lost" - propagated (duplicate)
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "connection lost",
    )))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    // First "parse error" - consumed
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("parse error")))?;
    assert_no_element_emitted(&mut result, 100).await;

    // Second "timeout" - propagated (duplicate)
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("timeout")))?;
    assert!(matches!(
        unwrap_stream(&mut result, 100).await,
        StreamItem::Error(_)
    ));

    drop(tx);
    assert_stream_ended(&mut result, 500).await;

    // Verify seen messages
    let seen = seen_messages.lock();
    assert_eq!(seen.len(), 3);
    assert!(seen.contains(&"Stream processing error: connection lost".to_string()));
    assert!(seen.contains(&"Stream processing error: timeout".to_string()));
    assert!(seen.contains(&"Stream processing error: parse error".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_on_error_handler_preserves_error_details_on_propagate() -> anyhow::Result<()> {
    // Arrange: Verify that propagated errors retain their original content
    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.on_error(|err| {
        // Propagate errors containing "propagate"
        !err.to_string().contains("propagate")
    });

    // Act & Assert
    // Consumed error
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "should be consumed",
    )))?;
    assert_no_element_emitted(&mut result, 100).await;

    // Propagated error - verify content is preserved
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "propagate: important error details here",
    )))?;
    match unwrap_stream(&mut result, 100).await {
        StreamItem::Error(e) => {
            assert!(e.to_string().contains("propagate"));
            assert!(e.to_string().contains("important error details here"));
        }
        _ => panic!("Expected error"),
    }

    drop(tx);
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_on_error_rapid_error_sequence() -> anyhow::Result<()> {
    // Arrange: Many errors in rapid succession
    let error_count = Arc::new(AtomicUsize::new(0));
    let error_count_clone = Arc::clone(&error_count);

    let (tx, stream) = test_channel_with_errors::<Sequenced<i32>>();
    let mut result = stream.on_error(move |_| {
        error_count_clone.fetch_add(1, Ordering::SeqCst);
        true // Consume all
    });

    // Act: Send 100 errors
    for i in 0..100 {
        tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(format!(
            "error{}",
            i
        ))))?;
    }

    // Give time for processing
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Send a value to verify stream is still working
    tx.unbounded_send(StreamItem::Value(Sequenced::new(42)))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut result, 100).await)).value,
        42
    );

    drop(tx);
    assert_stream_ended(&mut result, 500).await;

    // Assert all errors were processed
    assert_eq!(error_count.load(Ordering::SeqCst), 100);

    Ok(())
}
