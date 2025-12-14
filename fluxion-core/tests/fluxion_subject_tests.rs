// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, FluxionSubject, StreamItem, SubjectError};
use futures::StreamExt;

#[tokio::test]
async fn broadcasts_to_multiple_subscribers() {
    // Arrange
    let subject = FluxionSubject::<i32>::new();
    let mut a = subject.subscribe().unwrap();
    let mut b = subject.subscribe().unwrap();

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
    let mut stream = subject.subscribe().unwrap();

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
    let _stream = subject.subscribe().unwrap();

    // Act
    subject.close();

    // Assert
    let err = subject.send(StreamItem::Value(1)).unwrap_err();
    assert!(matches!(err, SubjectError::Closed));
}

#[tokio::test]
async fn subscribe_after_close_returns_closed_error() {
    // Arrange
    let subject = FluxionSubject::<i32>::new();

    // Act
    subject.close();
    let result = subject.subscribe();

    // Assert - subscribe() returns SubjectError::Closed
    assert!(matches!(result, Err(SubjectError::Closed)));
}

#[tokio::test]
async fn error_after_close_returns_error() {
    // Arrange
    let subject = FluxionSubject::<i32>::new();

    // Act
    subject.close();

    // Assert - can't send error to closed subject
    let err = subject
        .error(FluxionError::stream_error("test"))
        .unwrap_err();
    assert!(matches!(err, SubjectError::Closed));
}

#[tokio::test]
async fn late_subscriber_does_not_receive_past_items() {
    // Arrange
    let subject = FluxionSubject::<i32>::new();

    // Act - send before subscribing
    subject.send(StreamItem::Value(1)).unwrap();
    subject.send(StreamItem::Value(2)).unwrap();

    let mut late_subscriber = subject.subscribe().unwrap();

    subject.send(StreamItem::Value(3)).unwrap();

    // Assert - late subscriber only sees item 3
    assert_eq!(late_subscriber.next().await, Some(StreamItem::Value(3)));
}

#[tokio::test]
async fn clone_shares_same_state() {
    // Arrange
    let subject = FluxionSubject::<i32>::new();
    let cloned = subject.clone();
    let mut stream = subject.subscribe().unwrap();

    // Act - send from clone
    cloned.send(StreamItem::Value(42)).unwrap();

    // Assert - original subject's subscriber receives it
    assert_eq!(stream.next().await, Some(StreamItem::Value(42)));
}

#[tokio::test]
async fn clone_can_close_subject() {
    // Arrange
    let subject = FluxionSubject::<i32>::new();
    let cloned = subject.clone();
    let mut stream = subject.subscribe().unwrap();

    // Act - close from clone
    cloned.close();

    // Assert - original subject is closed
    let err = subject.send(StreamItem::Value(1)).unwrap_err();
    assert!(matches!(err, SubjectError::Closed));

    // Assert - stream completes
    assert_eq!(stream.next().await, None);
}

#[tokio::test]
async fn multiple_clones_all_share_state() {
    // Arrange
    let subject1 = FluxionSubject::<i32>::new();
    let subject2 = subject1.clone();
    let subject3 = subject2.clone();

    let mut stream = subject1.subscribe().unwrap();

    // Act - send from each clone
    subject1.send(StreamItem::Value(1)).unwrap();
    subject2.send(StreamItem::Value(2)).unwrap();
    subject3.send(StreamItem::Value(3)).unwrap();

    // Assert - all items received
    assert_eq!(stream.next().await, Some(StreamItem::Value(1)));
    assert_eq!(stream.next().await, Some(StreamItem::Value(2)));
    assert_eq!(stream.next().await, Some(StreamItem::Value(3)));
}

#[tokio::test]
async fn dropped_subscribers_are_cleaned_up() {
    // Arrange
    let subject = FluxionSubject::<i32>::new();
    let mut stream1 = subject.subscribe().unwrap();
    let stream2 = subject.subscribe().unwrap();

    // Act - drop one subscriber
    drop(stream2);

    // Send item - should only go to remaining subscriber
    subject.send(StreamItem::Value(1)).unwrap();

    // Assert - stream1 still receives items
    assert_eq!(stream1.next().await, Some(StreamItem::Value(1)));
}

#[tokio::test]
async fn send_to_subject_with_no_subscribers_succeeds() {
    // Arrange
    let subject = FluxionSubject::<i32>::new();

    // Act - send with no subscribers
    let result = subject.send(StreamItem::Value(1));

    // Assert - succeeds (items are just dropped)
    assert!(result.is_ok());
}

#[tokio::test]
async fn error_closes_all_subscribers() {
    // Arrange
    let subject = FluxionSubject::<i32>::new();
    let mut stream1 = subject.subscribe().unwrap();
    let mut stream2 = subject.subscribe().unwrap();

    // Act
    subject.error(FluxionError::stream_error("error")).unwrap();

    // Assert - both subscribers receive error and complete
    assert!(matches!(stream1.next().await, Some(StreamItem::Error(_))));
    assert_eq!(stream1.next().await, None);

    assert!(matches!(stream2.next().await, Some(StreamItem::Error(_))));
    assert_eq!(stream2.next().await, None);
}

#[tokio::test]
async fn is_closed_reflects_subject_state() {
    // Arrange
    let subject = FluxionSubject::<i32>::new();

    // Assert - initially open
    assert!(!subject.is_closed());

    // Act
    subject.close();

    // Assert - now closed
    assert!(subject.is_closed());
}

#[tokio::test]
async fn subscriber_count_tracks_active_subscribers() {
    // Arrange
    let subject = FluxionSubject::<i32>::new();

    // Assert - no subscribers initially
    assert_eq!(subject.subscriber_count(), 0);

    // Act - add subscribers
    let _stream1 = subject.subscribe().unwrap();
    assert_eq!(subject.subscriber_count(), 1);

    let stream2 = subject.subscribe().unwrap();
    assert_eq!(subject.subscriber_count(), 2);

    // Act - drop one subscriber and send to trigger cleanup
    drop(stream2);
    subject.send(StreamItem::Value(1)).unwrap();

    // Assert - count decreases after dropped subscriber is cleaned up
    assert_eq!(subject.subscriber_count(), 1);
}

#[tokio::test]
async fn next_sends_value_to_subscribers() {
    // Arrange
    let subject = FluxionSubject::<i32>::new();
    let mut stream = subject.subscribe().unwrap();

    // Act - use next() convenience method
    subject.next(42).unwrap();
    subject.next(100).unwrap();

    // Assert
    assert_eq!(stream.next().await, Some(StreamItem::Value(42)));
    assert_eq!(stream.next().await, Some(StreamItem::Value(100)));
}
