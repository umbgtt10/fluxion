use fluxion_stream::timestamped_channel::unbounded_channel;
use fluxion_test_utils::test_data::{person_alice, person_bob, TestData};
use tokio::sync::mpsc;

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
    sender1.send(1).unwrap();
    sender2.send(2).unwrap();
    sender1.send(3).unwrap();

    // Assert
    let msg1 = receiver1.recv().await.unwrap();
    let msg2 = receiver2.recv().await.unwrap();
    let msg3 = receiver1.recv().await.unwrap();

    assert!(msg1.sequence() < msg2.sequence());
    assert!(msg2.sequence() < msg3.sequence());
}

#[tokio::test]
async fn test_timestamped_channel_send_after_receiver_dropped() {
    // Arrange
    let (sender, receiver) = unbounded_channel();

    // Act - Drop receiver
    drop(receiver);

    // Assert - Send should fail with SendError
    let result = sender.send(person_alice());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().0, person_alice());
}

#[tokio::test]
async fn test_timestamped_channel_receiver_closes_channel() {
    // Arrange
    let (sender, mut receiver) = unbounded_channel();

    // Act - Send some messages
    sender.send(1).unwrap();
    sender.send(2).unwrap();

    // Act - Close receiver
    receiver.close();

    // Assert - Can still receive buffered messages
    assert_eq!(receiver.recv().await.unwrap().value, 1);
    assert_eq!(receiver.recv().await.unwrap().value, 2);

    // Assert - Next recv should return None
    assert!(receiver.recv().await.is_none());

    // Assert - Sender should detect closure
    assert!(sender.is_closed());
}

#[tokio::test]
async fn test_timestamped_channel_try_recv_empty() {
    // Arrange
    let (_sender, mut receiver) = unbounded_channel::<TestData>();

    // Act & Assert - try_recv on empty channel should return Empty
    let result = receiver.try_recv();
    assert!(matches!(result, Err(mpsc::error::TryRecvError::Empty)));
}

#[tokio::test]
async fn test_timestamped_channel_try_recv_success() {
    // Arrange
    let (sender, mut receiver) = unbounded_channel();

    // Act
    sender.send(person_alice()).unwrap();

    // Assert - try_recv should succeed immediately
    let result = receiver.try_recv();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, person_alice());
}

#[tokio::test]
async fn test_timestamped_channel_try_recv_after_sender_dropped() {
    // Arrange
    let (sender, mut receiver) = unbounded_channel();

    // Act
    sender.send(42).unwrap();
    drop(sender);

    // Assert - Can still try_recv buffered message
    assert_eq!(receiver.try_recv().unwrap().value, 42);

    // Assert - Next try_recv should return Disconnected
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

    // Act - Both senders can send
    sender1.send(person_alice()).unwrap();
    sender2.send(person_bob()).unwrap();

    // Assert
    let msg1 = receiver.recv().await.unwrap();
    let msg2 = receiver.recv().await.unwrap();

    assert_eq!(msg1.value, person_alice());
    assert_eq!(msg2.value, person_bob());

    // Verify ordering
    assert!(msg1.sequence() < msg2.sequence());
}

#[tokio::test]
async fn test_timestamped_channel_same_channel() {
    // Arrange
    let (sender1, _receiver1) = unbounded_channel::<TestData>();
    let sender2 = sender1.clone();
    let (sender3, _receiver2) = unbounded_channel::<TestData>();

    // Assert - sender1 and sender2 are the same channel (sender2 is cloned from sender1)
    assert!(sender1.same_channel(&sender2));

    // Assert - sender1 and sender3 are different channels
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

    // Act - Drop receiver to trigger closure
    drop(receiver);

    // Assert - The closed task should complete
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

    // Act - Send from multiple senders
    sender1.send(1).unwrap();
    sender2.send(2).unwrap();
    sender3.send(3).unwrap();
    sender1.send(4).unwrap();

    // Assert
    let mut messages = vec![];
    for _ in 0..4 {
        messages.push(receiver.recv().await.unwrap());
    }

    // Verify all messages received
    assert_eq!(messages[0].value, 1);
    assert_eq!(messages[1].value, 2);
    assert_eq!(messages[2].value, 3);
    assert_eq!(messages[3].value, 4);

    // Verify sequence ordering
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
    sender1.send(1).unwrap();
    sender2.send(2).unwrap();

    drop(sender1);
    drop(sender2);

    // Assert - Can still receive buffered messages
    assert_eq!(receiver.recv().await.unwrap().value, 1);
    assert_eq!(receiver.recv().await.unwrap().value, 2);

    // Assert - After buffer exhausted, should return None
    assert!(receiver.recv().await.is_none());
}

#[tokio::test]
async fn test_timestamped_channel_high_volume_ordering() {
    // Arrange
    let (sender, mut receiver) = unbounded_channel();

    // Act - Send 1000 messages
    for i in 0..1000 {
        sender.send(i).unwrap();
    }

    // Assert - Verify all received in sequence order
    let mut prev_seq = None;
    for i in 0..1000 {
        let msg = receiver.recv().await.unwrap();
        assert_eq!(msg.value, i);

        if let Some(prev) = prev_seq {
            assert!(msg.sequence() > prev, "Sequence should always increase");
        }
        prev_seq = Some(msg.sequence());
    }
}
