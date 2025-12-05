use fluxion_core::{FluxionError, FluxionSubject, StreamItem};
use futures::StreamExt;

#[tokio::test]
async fn broadcasts_to_multiple_subscribers() {
    let subject = FluxionSubject::<i32>::new();
    let mut a = subject.subscribe();
    let mut b = subject.subscribe();

    subject.send(StreamItem::Value(1)).unwrap();

    assert_eq!(a.next().await, Some(StreamItem::Value(1)));
    assert_eq!(b.next().await, Some(StreamItem::Value(1)));
}

#[tokio::test]
async fn error_is_propagated_and_closes() {
    let subject = FluxionSubject::<i32>::new();
    let mut stream = subject.subscribe();

    subject.error(FluxionError::stream_error("boom")).unwrap();

    assert!(matches!(stream.next().await, Some(StreamItem::Error(_))));
    assert_eq!(stream.next().await, None);
}

#[tokio::test]
async fn send_after_close_returns_error() {
    let subject = FluxionSubject::<i32>::new();
    let _stream = subject.subscribe();

    subject.close();
    let err = subject.send(StreamItem::Value(1)).unwrap_err();
    assert!(matches!(err, FluxionError::StreamProcessingError { .. }));
}
