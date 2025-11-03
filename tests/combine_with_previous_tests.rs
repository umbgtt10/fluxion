use futures::StreamExt;
use stream_processing::combine_with_previous::CombineWithPreviousExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

mod test_data;
use crate::test_data::person::Person;

const BUFFER_SIZE: usize = 10;

#[tokio::test]
async fn test_combine_with_previous_no_previous_value_emits() {
    // Arrange
    let (sender, receiver) = mpsc::channel(BUFFER_SIZE);
    let stream = ReceiverStream::new(receiver);
    let mut stream = stream.combine_with_previous();

    // Act
    sender.send(1).await.unwrap();

    // Assert
    let result = stream.next().await.unwrap();
    assert_eq!(result, (None, 1));
}

#[tokio::test]
async fn test_combine_with_previous_single_previous_value() {
    // Arrange
    let (sender, receiver) = mpsc::channel(BUFFER_SIZE);
    let stream = ReceiverStream::new(receiver);
    let mut stream = stream.combine_with_previous();

    // Act
    sender.send(1).await.unwrap();

    // Assert
    let first_result = stream.next().await.unwrap();
    assert_eq!(first_result, (None, 1));

    // Act
    sender.send(2).await.unwrap();

    // Assert
    let second_result = stream.next().await.unwrap();
    assert_eq!(second_result, (Some(1), 2));
}

#[tokio::test]
async fn test_combine_with_previous_multiple_values() {
    // Arrange
    let (sender, receiver) = mpsc::channel(BUFFER_SIZE);
    let stream = ReceiverStream::new(receiver);
    let mut stream = stream.combine_with_previous();

    // Act
    sender.send(1).await.unwrap();

    // Assert
    let first_result = stream.next().await.unwrap();
    assert_eq!(first_result, (None, 1));

    // Act
    sender.send(2).await.unwrap();

    // Assert
    let second_result = stream.next().await.unwrap();
    assert_eq!(second_result, (Some(1), 2));

    // Act
    sender.send(3).await.unwrap();

    // Assert
    let third_result = stream.next().await.unwrap();
    assert_eq!(third_result, (Some(2), 3));
}

#[tokio::test]
async fn test_combine_with_previous_stream_ends() {
    // Arrange
    let (sender, receiver) = mpsc::channel(BUFFER_SIZE);
    let stream = ReceiverStream::new(receiver);
    let mut stream = stream.combine_with_previous();

    // Act
    sender.send(1).await.unwrap();

    // Assert
    let first_result = stream.next().await.unwrap();
    assert_eq!(first_result, (None, 1));

    // Act
    sender.send(2).await.unwrap();

    // Assert
    let second_result = stream.next().await.unwrap();
    assert_eq!(second_result, (Some(1), 2));

    // Act
    drop(sender);

    // Assert
    let third_result = stream.next().await;
    assert!(third_result.is_none());
}

#[tokio::test]
async fn test_combine_with_previous_for_types() {
    // Arrange
    let (sender, receiver) = mpsc::channel(BUFFER_SIZE);
    let stream = ReceiverStream::new(receiver);
    let mut stream = stream.combine_with_previous();

    // Act
    sender
        .send(Person {
            name: "Alice".to_string(),
            age: 30,
        })
        .await
        .unwrap();

    // Assert
    let first_result = stream.next().await.unwrap();
    assert_eq!(
        first_result,
        (
            None,
            Person {
                name: "Alice".to_string(),
                age: 30
            }
        )
    );

    // Act
    sender
        .send(Person {
            name: "Bob".to_string(),
            age: 35,
        })
        .await
        .unwrap();

    // Assert
    let second_result = stream.next().await.unwrap();
    assert_eq!(
        second_result,
        (
            Some(Person {
                name: "Alice".to_string(),
                age: 30
            }),
            Person {
                name: "Bob".to_string(),
                age: 35
            }
        )
    );
}
