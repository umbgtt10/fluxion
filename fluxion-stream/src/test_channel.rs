use crate::timestamped::Timestamped;
use crate::timestamped_channel;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

/// A channel utility (moved from test-utils) combining sender and timestamped stream.
/// Intended for ergonomic stream construction in tests and simple scenarios.
pub struct TestChannel<T> {
    pub sender: timestamped_channel::UnboundedSender<T>,
    pub stream: UnboundedReceiverStream<Timestamped<T>>,
}

impl<T> TestChannel<T> {
    /// Create a new unbounded timestamped channel pair.
    pub fn new() -> Self {
        let (sender, receiver) = timestamped_channel::unbounded_channel();
        let stream = UnboundedReceiverStream::new(receiver.into_inner());
        Self { sender, stream }
    }

    /// Create an empty channel whose stream is already closed.
    pub fn empty() -> Self {
        let (sender, receiver) = timestamped_channel::unbounded_channel();
        let stream = UnboundedReceiverStream::new(receiver.into_inner());
        // close original sender
        drop(sender);
        // provide a dummy sender to satisfy struct field; sends will just be ignored if used.
        let (dummy_sender, _) = timestamped_channel::unbounded_channel();
        Self { sender: dummy_sender, stream }
    }

    /// Send a value, returning an error if the receiver is gone.
    pub fn send(&self, value: T) -> Result<(), mpsc::error::SendError<T>> {
        self.sender.send(value)
    }

    /// Convenience push (panics on failure) for terse test code.
    pub fn push(&self, value: T) {
        self.sender.send(value).expect("receiver dropped");
    }

    /// Close the sender side explicitly.
    pub fn close(self) {
        drop(self.sender);
    }
}

impl<T> Default for TestChannel<T> {
    fn default() -> Self { Self::new() }
}
