// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use fluxion_test_utils::{error_injection::ErrorInjectingStream, sequenced::Sequenced};
use futures::{stream, StreamExt};

#[tokio::test]
async fn test_error_injection_at_position() {
    // Arrange
    let items = vec![
        <Sequenced<_>>::with_timestamp(1, 1_000_000_000),
        <Sequenced<_>>::with_timestamp(2, 2_000_000_000),
        <Sequenced<_>>::with_timestamp(3, 3_000_000_000),
    ];

    let base_stream = stream::iter(items);
    let mut error_stream = ErrorInjectingStream::new(base_stream, 1);

    // Act
    let first = error_stream.next().await.unwrap();

    // Assert
    assert!(matches!(first, StreamItem::Value(_)));

    // Act
    let second = error_stream.next().await.unwrap();

    // Assert
    assert!(matches!(second, StreamItem::Error(_)));

    // Act
    let third = error_stream.next().await.unwrap();

    // Assert
    assert!(matches!(third, StreamItem::Value(_)));
}

#[tokio::test]
async fn test_error_injection_at_start() {
    // Arrange
    let items = vec![<Sequenced<_>>::with_timestamp(1, 1_000_000_000)];
    let base_stream = stream::iter(items);
    let mut error_stream = ErrorInjectingStream::new(base_stream, 0);

    // Act & Assert
    let first = error_stream.next().await.unwrap();
    match first {
        StreamItem::Error(e) => {
            assert!(matches!(e, FluxionError::StreamProcessingError { .. }));
        }
        StreamItem::Value(_) => panic!("Expected error at position 0"),
    }

    // Act
    let second = error_stream.next().await.unwrap();

    // Assert
    assert!(matches!(second, StreamItem::Value(_)));
}
