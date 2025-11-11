use fluxion_stream::combine_latest::{CombineLatestExt, CombinedState};
use fluxion_stream::timestamped::Timestamped;
use fluxion_stream::timestamped_channel::unbounded_channel;
use fluxion_test_utils::{
    person::Person,
    test_data::{
        TestData, animal_cat, animal_dog, person_alice, person_bob, person_charlie, person_dave,
    },
};
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[tokio::test]
async fn test_timestamped_channel_send_and_receive() {
    // Arrange
    let (sender, mut receiver) = unbounded_channel();

    // Act
    sender.send(person_alice()).unwrap();
    sender.send(person_bob()).unwrap();

    // Assert
    let msg1 = receiver.recv().await.unwrap();
    assert_eq!(msg1.value, person_alice());

    let msg2 = receiver.recv().await.unwrap();
    assert_eq!(msg2.value, person_bob());

    // Verify relative ordering - msg2 should have higher sequence than msg1
    assert!(msg2.sequence() > msg1.sequence());
}

#[tokio::test]
async fn test_timestamped_channel_ordering_across_channels() {
    // Arrange
    let (sender1, mut receiver1) = unbounded_channel();
    let (sender2, mut receiver2) = unbounded_channel();

    // Act - Send in specific order
    sender1.send(person_alice()).unwrap();
    sender2.send(person_bob()).unwrap();
    sender1.send(person_charlie()).unwrap();

    // Assert
    let msg1 = receiver1.recv().await.unwrap();
    let msg2 = receiver2.recv().await.unwrap();
    let msg3 = receiver1.recv().await.unwrap();

    assert_eq!(msg1.value, person_alice());
    assert_eq!(msg2.value, person_bob());
    assert_eq!(msg3.value, person_charlie());

    assert!(msg1.sequence() < msg2.sequence());
    assert!(msg2.sequence() < msg3.sequence());
}

#[tokio::test]
async fn test_timestamped_channel_send_after_receiver_dropped() {
    // Arrange
    let (sender, receiver) = unbounded_channel();

    // Act - Drop receiver
    drop(receiver);

    // Assert
    let result = sender.send(person_alice());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().0, person_alice());
}

#[tokio::test]
async fn test_timestamped_channel_receiver_closes_channel() {
    // Arrange
    let (sender, mut receiver) = unbounded_channel();

    // Act
    sender.send(person_alice()).unwrap();
    sender.send(person_bob()).unwrap();
    // Act
    receiver.close();

    // Assert
    assert_eq!(receiver.recv().await.unwrap().value, person_alice());
    assert_eq!(receiver.recv().await.unwrap().value, person_bob());
    assert!(receiver.recv().await.is_none());
    assert!(sender.is_closed());
}

#[tokio::test]
async fn test_timestamped_channel_try_recv_empty() {
    // Arrange
    let (_sender, mut receiver) = unbounded_channel::<TestData>();

    // Act & Assert
    let result = receiver.try_recv();
    assert!(matches!(result, Err(mpsc::error::TryRecvError::Empty)));
}

#[tokio::test]
async fn test_timestamped_channel_try_recv_success() {
    // Arrange
    let (sender, mut receiver) = unbounded_channel();

    // Act
    sender.send(person_alice()).unwrap();

    // Assert
    let result = receiver.try_recv();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, person_alice());
}

#[tokio::test]
async fn test_timestamped_channel_try_recv_after_sender_dropped() {
    // Arrange
    let (sender, mut receiver) = unbounded_channel();

    // Act
    sender.send(animal_cat()).unwrap();
    drop(sender);

    // Assert
    assert_eq!(receiver.try_recv().unwrap().value, animal_cat());
    let result = receiver.try_recv();
    assert!(matches!(
        result,
        Err(mpsc::error::TryRecvError::Disconnected)
    ));
}

#[tokio::test]
async fn test_timestamped_channel_sender_clone() {
    // Arrange
    let (sender1, mut receiver) = unbounded_channel();
    let sender2 = sender1.clone();

    // Act
    sender1.send(person_alice()).unwrap();
    sender2.send(person_bob()).unwrap();

    // Assert
    let msg1 = receiver.recv().await.unwrap();
    let msg2 = receiver.recv().await.unwrap();

    assert_eq!(msg1.value, person_alice());
    assert_eq!(msg2.value, person_bob());

    assert!(msg1.sequence() < msg2.sequence());
}

#[tokio::test]
async fn test_timestamped_channel_same_channel() {
    // Arrange
    let (sender1, _receiver1) = unbounded_channel::<TestData>();
    let sender2 = sender1.clone();
    let (sender3, _receiver2) = unbounded_channel::<TestData>();

    // act & Assert
    assert!(sender1.same_channel(&sender2));
    assert!(!sender1.same_channel(&sender3));
}

#[tokio::test]
async fn test_timestamped_channel_sender_closed_notification() {
    // Arrange
    let (sender, receiver) = unbounded_channel::<TestData>();

    // Act
    let closed_task = tokio::spawn(async move {
        sender.closed().await;
    });

    // Act
    drop(receiver);

    // Assert
    closed_task.await.unwrap();
}

#[tokio::test]
async fn test_timestamped_channel_is_closed_initially_false() {
    // Arrange
    let (sender, _receiver) = unbounded_channel::<TestData>();

    // Assert
    assert!(!sender.is_closed());
}

#[tokio::test]
async fn test_timestamped_channel_is_closed_after_receiver_dropped() {
    // Arrange
    let (sender, receiver) = unbounded_channel::<TestData>();

    // Assert
    assert!(!sender.is_closed());

    // Act
    drop(receiver);

    // Assert
    assert!(sender.is_closed());
}

#[tokio::test]
async fn test_timestamped_channel_multiple_senders_one_receiver() {
    // Arrange
    let (sender1, mut receiver) = unbounded_channel();
    let sender2 = sender1.clone();
    let sender3 = sender1.clone();

    // Act
    sender1.send(person_alice()).unwrap();
    sender2.send(person_bob()).unwrap();
    sender3.send(person_charlie()).unwrap();
    sender1.send(person_dave()).unwrap();

    // Assert
    let mut messages = vec![];
    for _ in 0..4 {
        messages.push(receiver.recv().await.unwrap());
    }

    assert_eq!(messages[0].value, person_alice());
    assert_eq!(messages[1].value, person_bob());
    assert_eq!(messages[2].value, person_charlie());
    assert_eq!(messages[3].value, person_dave());

    for i in 1..4 {
        assert!(messages[i - 1].sequence() < messages[i].sequence());
    }
}

#[tokio::test]
async fn test_timestamped_channel_receiver_none_after_all_senders_dropped() {
    // Arrange
    let (sender1, mut receiver) = unbounded_channel();
    let sender2 = sender1.clone();

    // Act
    sender1.send(animal_cat()).unwrap();
    sender2.send(animal_dog()).unwrap();

    drop(sender1);
    drop(sender2);

    // Assert
    assert_eq!(receiver.recv().await.unwrap().value, animal_cat());
    assert_eq!(receiver.recv().await.unwrap().value, animal_dog());
    assert!(receiver.recv().await.is_none());
}

#[tokio::test]
async fn test_timestamped_channel_high_volume_ordering() {
    // Arrange
    let (sender, mut receiver) = unbounded_channel();
    let test_items: Vec<TestData> = (0..1000)
        .map(|i| TestData::Person(Person::new(format!("Person{}", i), i as u32)))
        .collect();

    // Act
    for item in &test_items {
        sender.send(item.clone()).unwrap();
    }

    // Assert
    let mut prev_seq = None;
    for (i, expected) in test_items.iter().enumerate() {
        let msg = receiver.recv().await.unwrap();
        assert_eq!(msg.value, *expected, "Message {} should match", i);

        if let Some(prev) = prev_seq {
            assert!(msg.sequence() > prev, "Sequence should always increase");
        }
        prev_seq = Some(msg.sequence());
    }
}

#[tokio::test]
async fn test_timestamped_channel_sender_error_during_stream_processing() {
    // Arrange: Create two streams that will be combined
    let (sender1, receiver1) = unbounded_channel();
    let stream1 = UnboundedReceiverStream::new(receiver1.into_inner());

    let (sender2, receiver2) = unbounded_channel();
    let stream2 = UnboundedReceiverStream::new(receiver2.into_inner());

    static FILTER: fn(&CombinedState<Timestamped<TestData>>) -> bool = |_| true;

    let combined_stream = stream1.combine_latest(vec![stream2], FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act: Publish to both streams to get initial emission
    sender1.send(person_alice()).unwrap();
    sender2.send(animal_dog()).unwrap();

    // Assert: First emission succeeds
    let first = combined_stream.next().await;
    assert!(first.is_some(), "First emission should succeed");

    // Act: Drop sender2 to simulate sender error/closure
    drop(sender2);

    // Send more on sender1 - this should still work but stream may behave differently
    sender1.send(person_bob()).unwrap();

    // The stream should handle the closed sender gracefully
    // Since sender2 is dropped, stream2 will close, which should cause combine_latest to complete
    let _second = combined_stream.next().await;

    // Act: Try to send to dropped sender (demonstrates error handling)
    let (sender3, _receiver3) = unbounded_channel::<TestData>();
    drop(_receiver3);

    // Assert: Sending to a channel whose receiver is dropped results in an error
    let send_result = sender3.send(person_charlie());
    assert!(
        send_result.is_err(),
        "Sending should fail when receiver is dropped"
    );

    // Verify we can detect the error
    match send_result {
        Err(e) => {
            // The error contains the value that failed to send
            assert_eq!(e.0, person_charlie());
        }
        Ok(_) => panic!("Send should have failed"),
    }
}
