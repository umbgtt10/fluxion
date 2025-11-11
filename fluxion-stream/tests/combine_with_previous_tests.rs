use fluxion_stream::combine_with_previous::CombineWithPreviousExt;
use fluxion_stream::sequenced_channel::unbounded_channel;
use fluxion_test_utils::push;
use fluxion_test_utils::test_value::{person_alice, person_bob, person_charlie};
use futures::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[tokio::test]
async fn test_combine_with_previous_no_previous_value_emits() {
    // Arrange
    let (sender, receiver) = unbounded_channel();
    let stream = UnboundedReceiverStream::new(receiver.into_inner());
    let mut stream = stream.combine_with_previous();

    // Act
    push(person_alice(), &sender);

    // Assert
    let result = stream.next().await.unwrap();
    assert_eq!((result.0.map(|s| s.value), result.1.value), (None, person_alice()));
}

#[tokio::test]
async fn test_combine_with_previous_single_previous_value() {
    // Arrange
    let (sender, receiver) = unbounded_channel();
    let stream = UnboundedReceiverStream::new(receiver.into_inner());
    let mut stream = stream.combine_with_previous();

    // Act
    push(person_alice(), &sender);

    // Assert
    let first_result = stream.next().await.unwrap();
    assert_eq!(
        (first_result.0.map(|s| s.value), first_result.1.value),
        (None, person_alice())
    );

    // Act
    push(person_bob(), &sender);

    // Assert
    let second_result = stream.next().await.unwrap();
    assert_eq!(
        (second_result.0.map(|s| s.value), second_result.1.value),
        (Some(person_alice()), person_bob())
    );
}

#[tokio::test]
async fn test_combine_with_previous_multiple_values() {
    // Arrange
    let (sender, receiver) = unbounded_channel();
    let stream = UnboundedReceiverStream::new(receiver.into_inner());
    let mut stream = stream.combine_with_previous();

    // Act
    push(person_alice(), &sender);

    // Assert
    let first_result = stream.next().await.unwrap();
    assert_eq!(
        (first_result.0.map(|s| s.value), first_result.1.value),
        (None, person_alice())
    );

    // Act
    push(person_bob(), &sender);

    // Assert
    let second_result = stream.next().await.unwrap();
    assert_eq!(
        (second_result.0.map(|s| s.value), second_result.1.value),
        (Some(person_alice()), person_bob())
    );

    // Act
    push(person_charlie(), &sender);

    // Assert
    let third_result = stream.next().await.unwrap();
    assert_eq!(
        (third_result.0.map(|s| s.value), third_result.1.value),
        (Some(person_bob()), person_charlie())
    );
}

#[tokio::test]
async fn test_combine_with_previous_stream_ends() {
    // Arrange
    let (sender, receiver) = unbounded_channel();
    let stream = UnboundedReceiverStream::new(receiver.into_inner());
    let mut stream = stream.combine_with_previous();

    // Act
    push(person_alice(), &sender);

    // Assert
    let first_result = stream.next().await.unwrap();
    assert_eq!(
        (first_result.0.map(|s| s.value), first_result.1.value),
        (None, person_alice())
    );

    // Act
    push(person_bob(), &sender);

    // Assert
    let second_result = stream.next().await.unwrap();
    assert_eq!(
        (second_result.0.map(|s| s.value), second_result.1.value),
        (Some(person_alice()), person_bob())
    );

    // Act
    drop(sender);

    // Assert
    let third_result = stream.next().await;
    assert!(third_result.is_none());
}

#[tokio::test]
async fn test_combine_with_previous_for_types() {
    // Arrange
    let (sender, receiver) = unbounded_channel();
    let stream = UnboundedReceiverStream::new(receiver.into_inner());
    let mut stream = stream.combine_with_previous();

    // Act
    push(person_alice(), &sender);

    // Assert
    let first_result = stream.next().await.unwrap();
    assert_eq!(
        (first_result.0.map(|s| s.value), first_result.1.value),
        (None, person_alice())
    );

    // Act
    push(person_bob(), &sender);

    // Assert
    let second_result = stream.next().await.unwrap();
    assert_eq!(
        (second_result.0.map(|s| s.value), second_result.1.value),
        (Some(person_alice()), person_bob())
    );
}
