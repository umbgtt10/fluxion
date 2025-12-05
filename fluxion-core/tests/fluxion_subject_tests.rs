// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, FluxionSubject, StreamItem};
use futures::StreamExt;

#[tokio::test]
async fn broadcasts_to_multiple_subscribers() {
    // Arrange
    let subject = FluxionSubject::<i32>::new();
    let mut a = subject.subscribe();
    let mut b = subject.subscribe();

    // Act
    subject.send(StreamItem::Value(1)).unwrap();

    // Assert
    assert_eq!(a.next().await, Some(StreamItem::Value(1)));
    assert_eq!(b.next().await, Some(StreamItem::Value(1)));
}

#[tokio::test]
async fn error_is_propagated_and_closes() {
    // Arrange
    let subject = FluxionSubject::<i32>::new();
    let mut stream = subject.subscribe();

    // Act
    subject.error(FluxionError::stream_error("boom")).unwrap();

    // Assert
    assert!(matches!(stream.next().await, Some(StreamItem::Error(_))));
    assert_eq!(stream.next().await, None);
}

#[tokio::test]
async fn send_after_close_returns_error() {
    // Arrange
    let subject = FluxionSubject::<i32>::new();
    let _stream = subject.subscribe();

    // Act
    subject.close();

    // Assert
    let err = subject.send(StreamItem::Value(1)).unwrap_err();
    assert!(matches!(err, FluxionError::StreamProcessingError { .. }));
}
