use fluxion_test_utils::timestamped_channel::unbounded_channel;
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct TestData {
    id: u32,
    name: String,
}

impl TestData {
    fn new(id: u32, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
        }
    }
}

#[tokio::test]
async fn test_timestamped_channel_send_and_receive() {
    // Arrange
    let (sender, mut receiver) = unbounded_channel();
    let alice = TestData::new(1, "Alice");
    let bob = TestData::new(2, "Bob");

    // Act
    sender.send(alice.clone()).unwrap();
    sender.send(bob.clone()).unwrap();

    // Assert
    let msg1 = receiver.recv().await.unwrap();
    assert_eq!(msg1.value, alice, "First message should be Alice");

    let msg2 = receiver.recv().await.unwrap();
    assert_eq!(msg2.value, bob, "Second message should be Bob");

    // Verify relative ordering - msg2 should have higher sequence than msg1
    assert!(
        msg2.sequence() > msg1.sequence(),
        "Second message should have higher sequence number"
    );
}

#[tokio::test]
async fn test_timestamped_channel_ordering_across_channels() {
    // Arrange
    let (sender1, mut receiver1) = unbounded_channel();
    let (sender2, mut receiver2) = unbounded_channel();
    let alice = TestData::new(1, "Alice");
    let bob = TestData::new(2, "Bob");
    let charlie = TestData::new(3, "Charlie");

    // Act - Send in specific order
    sender1.send(alice.clone()).unwrap();
    sender2.send(bob.clone()).unwrap();
    sender1.send(charlie.clone()).unwrap();

    // Assert
    let msg1 = receiver1.recv().await.unwrap();
    let msg2 = receiver2.recv().await.unwrap();
    let msg3 = receiver1.recv().await.unwrap();

    assert_eq!(
        msg1.value, alice,
        "First message from channel 1 should be Alice"
    );
    assert_eq!(msg2.value, bob, "Message from channel 2 should be Bob");
    assert_eq!(
        msg3.value, charlie,
        "Second message from channel 1 should be Charlie"
    );

    assert!(
        msg1.sequence() < msg2.sequence(),
        "Channel 2 message should have higher sequence than first channel 1 message"
    );
    assert!(
        msg2.sequence() < msg3.sequence(),
        "Second channel 1 message should have highest sequence"
    );
}

#[tokio::test]
async fn test_timestamped_channel_send_after_receiver_dropped() {
    // Arrange
    let (sender, receiver) = unbounded_channel();
    let alice = TestData::new(1, "Alice");

    // Act - Drop receiver
    drop(receiver);

    // Assert
    let result = sender.send(alice.clone());
    assert!(result.is_err(), "Send should fail when receiver is dropped");
    assert_eq!(
        result.unwrap_err().0,
        alice,
        "Error should contain the unsent value"
    );
}

#[tokio::test]
async fn test_timestamped_channel_receiver_closes_channel() {
    // Arrange
    let (sender, mut receiver) = unbounded_channel();
    let alice = TestData::new(1, "Alice");
    let bob = TestData::new(2, "Bob");

    // Act
    sender.send(alice.clone()).unwrap();
    sender.send(bob.clone()).unwrap();
    receiver.close();

    // Assert
    assert_eq!(
        receiver.recv().await.unwrap().value,
        alice,
        "Should receive Alice from closed channel"
    );
    assert_eq!(
        receiver.recv().await.unwrap().value,
        bob,
        "Should receive Bob from closed channel"
    );
    assert!(
        receiver.recv().await.is_none(),
        "Should receive None after draining closed channel"
    );
    assert!(sender.is_closed(), "Sender should be marked as closed");
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
    let alice = TestData::new(1, "Alice");

    // Act
    sender.send(alice.clone()).unwrap();

    // Assert
    let result = receiver.try_recv();
    assert!(
        result.is_ok(),
        "try_recv should succeed when message is available"
    );
    assert_eq!(result.unwrap().value, alice, "Should receive Alice");
}

#[tokio::test]
async fn test_timestamped_channel_try_recv_after_sender_dropped() {
    // Arrange
    let (sender, mut receiver) = unbounded_channel();
    let cat = TestData::new(1, "Cat");

    // Act
    sender.send(cat.clone()).unwrap();
    drop(sender);

    // Assert
    assert_eq!(
        receiver.try_recv().unwrap().value,
        cat,
        "Should receive buffered Cat"
    );
    let result = receiver.try_recv();
    assert!(
        matches!(result, Err(mpsc::error::TryRecvError::Disconnected)),
        "Should return Disconnected error after sender is dropped"
    );
}

#[tokio::test]
async fn test_timestamped_channel_sender_clone() {
    // Arrange
    let (sender1, mut receiver) = unbounded_channel();
    let sender2 = sender1.clone();
    let alice = TestData::new(1, "Alice");
    let bob = TestData::new(2, "Bob");

    // Act
    sender1.send(alice.clone()).unwrap();
    sender2.send(bob.clone()).unwrap();

    // Assert
    let msg1 = receiver.recv().await.unwrap();
    let msg2 = receiver.recv().await.unwrap();

    assert_eq!(msg1.value, alice);
    assert_eq!(msg2.value, bob);

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
    let alice = TestData::new(1, "Alice");
    let bob = TestData::new(2, "Bob");
    let charlie = TestData::new(3, "Charlie");
    let dave = TestData::new(4, "Dave");

    // Act
    sender1.send(alice.clone()).unwrap();
    sender2.send(bob.clone()).unwrap();
    sender3.send(charlie.clone()).unwrap();
    sender1.send(dave.clone()).unwrap();

    // Assert
    let mut messages = vec![];
    for _ in 0..4 {
        messages.push(receiver.recv().await.unwrap());
    }

    assert_eq!(messages[0].value, alice);
    assert_eq!(messages[1].value, bob);
    assert_eq!(messages[2].value, charlie);
    assert_eq!(messages[3].value, dave);

    for i in 1..4 {
        assert!(messages[i - 1].sequence() < messages[i].sequence());
    }
}

#[tokio::test]
async fn test_timestamped_channel_receiver_none_after_all_senders_dropped() {
    // Arrange
    let (sender1, mut receiver) = unbounded_channel();
    let sender2 = sender1.clone();
    let cat = TestData::new(1, "Cat");
    let dog = TestData::new(2, "Dog");

    // Act
    sender1.send(cat.clone()).unwrap();
    sender2.send(dog.clone()).unwrap();

    drop(sender1);
    drop(sender2);

    // Assert
    assert_eq!(receiver.recv().await.unwrap().value, cat);
    assert_eq!(receiver.recv().await.unwrap().value, dog);
    assert!(receiver.recv().await.is_none());
}

#[tokio::test]
async fn test_timestamped_channel_high_volume_ordering() {
    // Arrange
    let (sender, mut receiver) = unbounded_channel();
    let test_items: Vec<TestData> = (0..1000)
        .map(|i| TestData::new(i, &format!("Person{}", i)))
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
async fn test_timestamped_channel_sender_error() {
    // Arrange
    let (sender, _receiver) = unbounded_channel::<TestData>();
    let charlie = TestData::new(3, "Charlie");

    // Drop receiver
    drop(_receiver);

    // Assert: Sending to a channel whose receiver is dropped results in an error
    let send_result = sender.send(charlie.clone());
    assert!(
        send_result.is_err(),
        "Sending should fail when receiver is dropped"
    );

    // Verify we can detect the error
    match send_result {
        Err(e) => {
            // The error contains the value that failed to send
            assert_eq!(e.0, charlie);
        }
        Ok(_) => panic!("Send should have failed"),
    }
}
