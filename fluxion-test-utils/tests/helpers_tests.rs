// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_test_utils::{
    assert_no_element_emitted, assert_stream_ended, test_channel, test_channel_with_errors,
    unwrap_stream, unwrap_value,
};

#[tokio::test]
async fn test_assert_no_element_emitted() {
    // Arrange
    let (_tx, mut stream) = test_channel::<i32>();

    // Act & Assert
    assert_no_element_emitted(&mut stream, 100).await;
}

#[tokio::test]
#[should_panic = "Timeout: No item received within 100 ms"]
async fn test_unwrap_stream_timeout() {
    // Arrange
    let (_tx, mut stream) = test_channel::<i32>();

    // Act & Assert
    unwrap_stream(&mut stream, 100).await;
}

#[tokio::test]
#[should_panic = "called `Option::unwrap()` on a `None` value"]
async fn test_unwrap_stream_empty() {
    // Arrange
    let (tx, mut stream) = test_channel::<i32>();

    // Act
    drop(tx);

    // Assert
    unwrap_stream(&mut stream, 500).await;
}

#[tokio::test]
#[should_panic = "Expected Value but got Error: Stream processing error: injected error"]
async fn test_unwrap_stream_error_injected() {
    // Arrange
    let (tx, mut stream) = test_channel_with_errors::<i32>();

    // Act
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "injected error",
    )))
    .unwrap();

    // Assert
    unwrap_value(Some(unwrap_stream(&mut stream, 500).await));
}

#[tokio::test]
async fn test_assert_stream_ended_success() {
    // Arrange
    let (tx, mut stream) = test_channel::<i32>();

    // Act
    drop(tx);

    // Assert
    assert_stream_ended(&mut stream, 500).await;
}

#[tokio::test]
#[should_panic = "Expected stream to end but it returned a value"]
async fn test_assert_stream_ended_returns_value() {
    // Arrange
    let (tx, mut stream) = test_channel::<i32>();

    // Act
    tx.unbounded_send(42).unwrap();

    // Assert
    assert_stream_ended(&mut stream, 500).await;
}

#[tokio::test]
#[should_panic = "Timeout: Stream did not end within 100 ms"]
async fn test_assert_stream_ended_timeout() {
    // Arrange
    let (_tx, mut stream) = test_channel::<i32>();

    // Act & Assert
    assert_stream_ended(&mut stream, 100).await;
}
