use fluxion_stream::timestamped::Timestamped;
use fluxion_stream::timestamped_channel;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

/// A test channel that combines the sender and stream into a single struct.
/// Automatically converts the receiver into an UnboundedReceiverStream.
pub struct TestChannel<T> {
    pub sender: timestamped_channel::UnboundedSender<T>,
    pub stream: UnboundedReceiverStream<Timestamped<T>>,
}

impl<T> TestChannel<T> {
    /// Creates a new test channel with unbounded capacity.
    pub fn new() -> Self {
        let (sender, receiver) = timestamped_channel::unbounded_channel();
        let stream = UnboundedReceiverStream::new(receiver.into_inner());
        Self { sender, stream }
    }

    /// Send a value through the channel.
    ///
    /// # Errors
    ///
    /// Returns an error if the receiver has been dropped.
    pub fn send(&self, value: T) -> Result<(), mpsc::error::SendError<T>> {
        self.sender.send(value)
    }

    /// Close the sender side of the channel.
    pub fn close(self) {
        drop(self.sender);
    }
}

impl<T> Default for TestChannel<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create multiple test channels at once.
pub struct TestChannels;

impl TestChannels {
    /// Creates three test channels.
    pub fn three<T>() -> (TestChannel<T>, TestChannel<T>, TestChannel<T>) {
        (TestChannel::new(), TestChannel::new(), TestChannel::new())
    }

    /// Creates two test channels.
    pub fn two<T>() -> (TestChannel<T>, TestChannel<T>) {
        (TestChannel::new(), TestChannel::new())
    }
}
