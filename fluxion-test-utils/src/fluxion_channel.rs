//! Sequenced channels and testing utilities.
//!
//! This module provides:
//! - Low-level sequenced channels (`sequenced_channel` submodule)
//! - High-level `FluxionChannel` for convenient testing

use crate::sequenced::Sequenced;
use tokio::sync::mpsc::{
    self, UnboundedReceiver as TokioUnboundedReceiver, UnboundedSender as TokioUnboundedSender,
};
use tokio_stream::wrappers::UnboundedReceiverStream;

pub struct FluxionChannel<T> {
    pub sender: sequenced_channel::UnboundedSender<T>,
    pub stream: UnboundedReceiverStream<Sequenced<T>>,
}

impl<T> FluxionChannel<T> {
    pub fn new() -> Self {
        let (sender, receiver) = sequenced_channel::unbounded_channel();
        let stream = UnboundedReceiverStream::new(receiver.into_inner());
        Self { sender, stream }
    }

    pub fn empty() -> Self {
        let (sender, receiver) = sequenced_channel::unbounded_channel();
        let stream = UnboundedReceiverStream::new(receiver.into_inner());
        drop(sender);

        let (dummy_sender, _) = sequenced_channel::unbounded_channel();
        Self {
            sender: dummy_sender,
            stream,
        }
    }

    pub fn push(&self, value: T) {
        self.sender.send(value).expect("receiver dropped");
    }

    pub fn close(self) {
        drop(self.sender);
    }
}

impl<T> Default for FluxionChannel<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create multiple test channels at once.
pub struct TestChannels;

impl TestChannels {
    /// Creates three test channels.
    pub fn three<T>() -> (FluxionChannel<T>, FluxionChannel<T>, FluxionChannel<T>) {
        (
            FluxionChannel::new(),
            FluxionChannel::new(),
            FluxionChannel::new(),
        )
    }

    /// Creates two test channels.
    pub fn two<T>() -> (FluxionChannel<T>, FluxionChannel<T>) {
        (FluxionChannel::new(), FluxionChannel::new())
    }
}

/// Low-level sequenced channel primitives.
///
/// For most testing scenarios, use `FluxionChannel` instead. This submodule
/// is useful when you need fine-grained control over sender/receiver handling.
pub(crate) mod sequenced_channel {
    use super::*;

    /// An unbounded sender that automatically wraps values with sequence numbers.
    ///
    /// Users send regular values of type `T`, but they're automatically wrapped
    /// in `Sequenced<T>` with a sequence number assigned at send time.
    #[derive(Debug)]
    pub struct UnboundedSender<T> {
        pub(super) inner: TokioUnboundedSender<Sequenced<T>>,
    }

    impl<T> Clone for UnboundedSender<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }

    impl<T> UnboundedSender<T> {
        /// Sends a value, automatically wrapping it with a timestamp.
        ///
        /// # Errors
        ///
        /// Returns an error if the receiver has been dropped.
        pub fn send(&self, value: T) -> Result<(), mpsc::error::SendError<T>> {
            self.inner
                .send(Sequenced::new(value))
                .map_err(|e| mpsc::error::SendError(e.0.value))
        }

        /// Checks if the channel has been closed.
        pub fn is_closed(&self) -> bool {
            self.inner.is_closed()
        }

        /// Completes when the receiver has dropped.
        pub async fn closed(&self) {
            self.inner.closed().await
        }

        /// Checks if the channel is ready to send more messages.
        pub fn same_channel(&self, other: &Self) -> bool {
            self.inner.same_channel(&other.inner)
        }
    }

    /// An unbounded receiver that receives sequenced values.
    ///
    /// Receives values wrapped in `Sequenced<T>` that were automatically
    /// sequenced by the sender.
    #[derive(Debug)]
    pub struct UnboundedReceiver<T> {
        pub(super) inner: TokioUnboundedReceiver<Sequenced<T>>,
    }

    impl<T> UnboundedReceiver<T> {
        #[cfg(test)]
        pub async fn recv(&mut self) -> Option<Sequenced<T>> {
            self.inner.recv().await
        }

        #[cfg(test)]
        pub fn try_recv(&mut self) -> Result<Sequenced<T>, mpsc::error::TryRecvError> {
            self.inner.try_recv()
        }

        #[cfg(test)]
        pub fn close(&mut self) {
            self.inner.close()
        }

        pub fn into_inner(self) -> TokioUnboundedReceiver<Sequenced<T>> {
            self.inner
        }
    }

    pub fn unbounded_channel<T>() -> (UnboundedSender<T>, UnboundedReceiver<T>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (
            UnboundedSender { inner: tx },
            UnboundedReceiver { inner: rx },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::sequenced_channel::unbounded_channel;
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
    async fn test_sequenced_channel_send_and_receive() {
        let (sender, mut receiver) = unbounded_channel();
        let alice = TestData::new(1, "Alice");
        let bob = TestData::new(2, "Bob");

        sender.send(alice.clone()).unwrap();
        sender.send(bob.clone()).unwrap();

        let msg1 = receiver.recv().await.unwrap();
        assert_eq!(msg1.value, alice);

        let msg2 = receiver.recv().await.unwrap();
        assert_eq!(msg2.value, bob);

        assert!(msg2.sequence() > msg1.sequence());
    }

    #[tokio::test]
    async fn test_sequenced_channel_ordering_across_channels() {
        let (sender1, mut receiver1) = unbounded_channel();
        let (sender2, mut receiver2) = unbounded_channel();
        let alice = TestData::new(1, "Alice");
        let bob = TestData::new(2, "Bob");
        let charlie = TestData::new(3, "Charlie");

        sender1.send(alice.clone()).unwrap();
        sender2.send(bob.clone()).unwrap();
        sender1.send(charlie.clone()).unwrap();

        let msg1 = receiver1.recv().await.unwrap();
        let msg2 = receiver2.recv().await.unwrap();
        let msg3 = receiver1.recv().await.unwrap();

        assert_eq!(msg1.value, alice);
        assert_eq!(msg2.value, bob);
        assert_eq!(msg3.value, charlie);

        assert!(msg1.sequence() < msg2.sequence());
        assert!(msg2.sequence() < msg3.sequence());
    }

    #[tokio::test]
    async fn test_sequenced_channel_send_after_receiver_dropped() {
        let (sender, receiver) = unbounded_channel();
        let alice = TestData::new(1, "Alice");

        drop(receiver);

        let result = sender.send(alice.clone());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().0, alice);
    }

    #[tokio::test]
    async fn test_sequenced_channel_receiver_closes_channel() {
        let (sender, mut receiver) = unbounded_channel();
        let alice = TestData::new(1, "Alice");
        let bob = TestData::new(2, "Bob");

        sender.send(alice.clone()).unwrap();
        sender.send(bob.clone()).unwrap();
        receiver.close();

        assert_eq!(receiver.recv().await.unwrap().value, alice);
        assert_eq!(receiver.recv().await.unwrap().value, bob);
        assert!(receiver.recv().await.is_none());
        assert!(sender.is_closed());
    }

    #[tokio::test]
    async fn test_sequenced_channel_try_recv_empty() {
        let (_sender, mut receiver) = unbounded_channel::<TestData>();

        let result = receiver.try_recv();
        assert!(matches!(result, Err(mpsc::error::TryRecvError::Empty)));
    }

    #[tokio::test]
    async fn test_sequenced_channel_try_recv_success() {
        let (sender, mut receiver) = unbounded_channel();
        let alice = TestData::new(1, "Alice");

        sender.send(alice.clone()).unwrap();

        let result = receiver.try_recv();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value, alice);
    }

    #[tokio::test]
    async fn test_sequenced_channel_try_recv_after_sender_dropped() {
        let (sender, mut receiver) = unbounded_channel();
        let cat = TestData::new(1, "Cat");

        sender.send(cat.clone()).unwrap();
        drop(sender);

        assert_eq!(receiver.try_recv().unwrap().value, cat);
        let result = receiver.try_recv();
        assert!(matches!(
            result,
            Err(mpsc::error::TryRecvError::Disconnected)
        ));
    }

    #[tokio::test]
    async fn test_sequenced_channel_sender_clone() {
        let (sender1, mut receiver) = unbounded_channel();
        let sender2 = sender1.clone();
        let alice = TestData::new(1, "Alice");
        let bob = TestData::new(2, "Bob");

        sender1.send(alice.clone()).unwrap();
        sender2.send(bob.clone()).unwrap();

        let msg1 = receiver.recv().await.unwrap();
        let msg2 = receiver.recv().await.unwrap();

        assert_eq!(msg1.value, alice);
        assert_eq!(msg2.value, bob);
        assert!(msg1.sequence() < msg2.sequence());
    }

    #[tokio::test]
    async fn test_sequenced_channel_same_channel() {
        let (sender1, _receiver1) = unbounded_channel::<TestData>();
        let sender2 = sender1.clone();
        let (sender3, _receiver2) = unbounded_channel::<TestData>();

        assert!(sender1.same_channel(&sender2));
        assert!(!sender1.same_channel(&sender3));
    }

    #[tokio::test]
    async fn test_sequenced_channel_sender_closed_notification() {
        let (sender, receiver) = unbounded_channel::<TestData>();

        let closed_task = tokio::spawn(async move {
            sender.closed().await;
        });

        drop(receiver);

        closed_task.await.unwrap();
    }

    #[tokio::test]
    async fn test_sequenced_channel_is_closed_initially_false() {
        let (sender, _receiver) = unbounded_channel::<TestData>();
        assert!(!sender.is_closed());
    }

    #[tokio::test]
    async fn test_sequenced_channel_is_closed_after_receiver_dropped() {
        let (sender, receiver) = unbounded_channel::<TestData>();
        assert!(!sender.is_closed());
        drop(receiver);
        assert!(sender.is_closed());
    }

    #[tokio::test]
    async fn test_sequenced_channel_multiple_senders_one_receiver() {
        let (sender1, mut receiver) = unbounded_channel();
        let sender2 = sender1.clone();
        let sender3 = sender1.clone();
        let alice = TestData::new(1, "Alice");
        let bob = TestData::new(2, "Bob");
        let charlie = TestData::new(3, "Charlie");
        let dave = TestData::new(4, "Dave");

        sender1.send(alice.clone()).unwrap();
        sender2.send(bob.clone()).unwrap();
        sender3.send(charlie.clone()).unwrap();
        sender1.send(dave.clone()).unwrap();

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
    async fn test_sequenced_channel_receiver_none_after_all_senders_dropped() {
        let (sender1, mut receiver) = unbounded_channel();
        let sender2 = sender1.clone();
        let cat = TestData::new(1, "Cat");
        let dog = TestData::new(2, "Dog");

        sender1.send(cat.clone()).unwrap();
        sender2.send(dog.clone()).unwrap();

        drop(sender1);
        drop(sender2);

        assert_eq!(receiver.recv().await.unwrap().value, cat);
        assert_eq!(receiver.recv().await.unwrap().value, dog);
        assert!(receiver.recv().await.is_none());
    }

    #[tokio::test]
    async fn test_sequenced_channel_high_volume_ordering() {
        let (sender, mut receiver) = unbounded_channel();
        let test_items: Vec<TestData> = (0..1000)
            .map(|i| TestData::new(i, &format!("Person{}", i)))
            .collect();

        for item in &test_items {
            sender.send(item.clone()).unwrap();
        }

        let mut prev_seq = None;
        for (i, expected) in test_items.iter().enumerate() {
            let msg = receiver.recv().await.unwrap();
            assert_eq!(msg.value, *expected, "Message {} should match", i);

            if let Some(prev) = prev_seq {
                assert!(msg.sequence() > prev);
            }
            prev_seq = Some(msg.sequence());
        }
    }

    #[tokio::test]
    async fn test_sequenced_channel_sender_error() {
        let (sender, _receiver) = unbounded_channel::<TestData>();
        let charlie = TestData::new(3, "Charlie");

        drop(_receiver);

        let send_result = sender.send(charlie.clone());
        assert!(send_result.is_err());

        match send_result {
            Err(e) => {
                assert_eq!(e.0, charlie);
            }
            Ok(_) => panic!("Send should have failed"),
        }
    }
}
