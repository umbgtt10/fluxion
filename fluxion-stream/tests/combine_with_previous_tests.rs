use fluxion_stream::combine_with_previous::CombineWithPreviousExt;
use fluxion_stream::timestamped_channel::unbounded_channel;
use fluxion_test_utils::push;
use fluxion_test_utils::test_data::{person_alice, person_bob, person_charlie};
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
    assert_eq!(
        (result.0.map(|s| s.value), result.1.value),
        (None, person_alice())
    );
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

#[tokio::test]
async fn test_combine_with_previous_high_volume_sequential() {
    // Arrange
    let (sender, receiver) = unbounded_channel();
    let stream = UnboundedReceiverStream::new(receiver.into_inner());
    let mut stream = stream.combine_with_previous();

    // Act: send a sequence of 200 items (mix of known fixtures cycling Alice,Bob,Charlie)
    for i in 0..200 {
        match i % 3 {
            0 => push(person_alice(), &sender),
            1 => push(person_bob(), &sender),
            _ => push(person_charlie(), &sender),
        }
    }
    // Close input so the stream can finish
    drop(sender);

    // Assert: first has no previous
    let first = stream.next().await.unwrap();
    assert_eq!((first.0.map(|s| s.value), first.1.value), (None, person_alice()));

    // Then verify a few samples and the last pair to ensure previous tracking holds
    let second = stream.next().await.unwrap();
    assert_eq!(
        (second.0.map(|s| s.value), second.1.value),
        (Some(person_alice()), person_bob())
    );

    // Drain remaining 198 pairs (we already consumed 2 above)
    let mut last_prev = None;
    let mut last_curr = None;
    for _ in 0..198 {
        let (prev, curr) = stream.next().await.unwrap();
        last_prev = prev.map(|s| s.value);
        last_curr = Some(curr.value);
    }
    // Stream should now be ended
    assert!(stream.next().await.is_none());
    // Last pair should have a previous
    assert!(last_prev.is_some());
    assert!(last_curr.is_some());
}
