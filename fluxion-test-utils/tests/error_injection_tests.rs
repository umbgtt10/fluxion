use fluxion_core::{FluxionError, StreamItem};
use fluxion_test_utils::{ErrorInjectingStream, Sequenced};
use futures::{stream, StreamExt};

#[tokio::test]
async fn test_error_injection_at_position() {
    let items = vec![
        <Sequenced<_>>::with_timestamp(1, 1_000_000_000),
        <Sequenced<_>>::with_timestamp(2, 2_000_000_000),
        <Sequenced<_>>::with_timestamp(3, 3_000_000_000),
    ];

    let base_stream = stream::iter(items);
    let mut error_stream = ErrorInjectingStream::new(base_stream, 1);

    // Position 0: value
    let first = error_stream.next().await.unwrap();
    assert!(matches!(first, StreamItem::Value(_)));

    // Position 1: injected error
    let second = error_stream.next().await.unwrap();
    assert!(matches!(second, StreamItem::Error(_)));

    // Position 2: value
    let third = error_stream.next().await.unwrap();
    assert!(matches!(third, StreamItem::Value(_)));
}

#[tokio::test]
async fn test_error_injection_at_start() {
    let items = vec![<Sequenced<_>>::with_timestamp(1, 1_000_000_000)];
    let base_stream = stream::iter(items);
    let mut error_stream = ErrorInjectingStream::new(base_stream, 0);

    // First emission is the error
    let first = error_stream.next().await.unwrap();
    match first {
        StreamItem::Error(e) => {
            assert!(matches!(e, FluxionError::StreamProcessingError { .. }));
        }
        StreamItem::Value(_) => panic!("Expected error at position 0"),
    }

    // Second emission is the value
    let second = error_stream.next().await.unwrap();
    assert!(matches!(second, StreamItem::Value(_)));
}
