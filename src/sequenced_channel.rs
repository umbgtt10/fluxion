//! Sequenced channels that automatically wrap values with temporal ordering.
//!
//! This module provides channel implementations that transparently add sequence numbers
//! to messages as they're sent, enabling temporal ordering when merging streams.

use crate::sequenced::Sequenced;
use tokio::sync::mpsc::{
    self, UnboundedReceiver as TokioUnboundedReceiver, UnboundedSender as TokioUnboundedSender,
};

/// An unbounded sender that automatically wraps values with sequence numbers.
///
/// Users send regular values of type `T`, but they're automatically wrapped
/// in `Sequenced<T>` with a sequence number assigned at send time.
#[derive(Debug)]
pub struct UnboundedSender<T> {
    inner: TokioUnboundedSender<Sequenced<T>>,
}

impl<T> Clone for UnboundedSender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> UnboundedSender<T> {
    /// Sends a value, automatically wrapping it with a sequence number.
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
    inner: TokioUnboundedReceiver<Sequenced<T>>,
}

impl<T> UnboundedReceiver<T> {
    /// Receives the next sequenced value.
    pub async fn recv(&mut self) -> Option<Sequenced<T>> {
        self.inner.recv().await
    }

    /// Attempts to receive without blocking.
    pub fn try_recv(&mut self) -> Result<Sequenced<T>, mpsc::error::TryRecvError> {
        self.inner.try_recv()
    }

    /// Polls to receive the next value.
    pub fn poll_recv(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Sequenced<T>>> {
        self.inner.poll_recv(cx)
    }

    /// Closes the receiving half of the channel.
    pub fn close(&mut self) {
        self.inner.close()
    }

    /// Converts this receiver into the underlying tokio receiver.
    ///
    /// This is useful for creating streams with `tokio_stream::wrappers::UnboundedReceiverStream`.
    pub fn into_inner(self) -> TokioUnboundedReceiver<Sequenced<T>> {
        self.inner
    }
}

/// Creates an unbounded sequenced channel.
///
/// The sender automatically wraps values with sequence numbers,
/// and the receiver receives `Sequenced<T>` values.
///
/// # Examples
///
/// ```
/// use fluxion::sequenced_channel::unbounded_channel;
/// use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
/// use fluxion::select_all_ordered::SelectAllExt;
///
/// #[tokio::main]
/// async fn main() {
///     let (tx1, rx1) = unbounded_channel();
///     let (tx2, rx2) = unbounded_channel();
///
///     // Merge streams - they'll be ordered by send time
///     let stream1 = UnboundedReceiverStream::new(rx1.into_inner());
///     let stream2 = UnboundedReceiverStream::new(rx2.into_inner());
///     let mut merged = vec![stream1, stream2].select_all_ordered();
///
///     // Users just send regular values - sequencing is automatic!
///     tx1.send("from stream 1").unwrap();
///     tx2.send("from stream 2").unwrap();
///     tx1.send("from stream 1 again").unwrap();
///
///     // Items come out in send order
///     assert_eq!(merged.next().await.unwrap().value, "from stream 1");
///     assert_eq!(merged.next().await.unwrap().value, "from stream 2");
///     assert_eq!(merged.next().await.unwrap().value, "from stream 1 again");
/// }
/// ```
pub fn unbounded_channel<T>() -> (UnboundedSender<T>, UnboundedReceiver<T>) {
    let (tx, rx) = mpsc::unbounded_channel();
    (
        UnboundedSender { inner: tx },
        UnboundedReceiver { inner: rx },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_send_and_receive() {
        let (tx, mut rx) = unbounded_channel();

        tx.send("hello").unwrap();
        tx.send("world").unwrap();

        let msg1 = rx.recv().await.unwrap();
        assert_eq!(msg1.value, "hello");

        let msg2 = rx.recv().await.unwrap();
        assert_eq!(msg2.value, "world");

        // Verify relative ordering - msg2 should have higher sequence than msg1
        assert!(msg2.sequence() > msg1.sequence());
    }

    #[tokio::test]
    async fn test_ordering_across_channels() {
        let (tx1, mut rx1) = unbounded_channel();
        let (tx2, mut rx2) = unbounded_channel();

        // Send in specific order
        tx1.send(1).unwrap();
        tx2.send(2).unwrap();
        tx1.send(3).unwrap();

        let msg1 = rx1.recv().await.unwrap();
        let msg2 = rx2.recv().await.unwrap();
        let msg3 = rx1.recv().await.unwrap();

        assert!(msg1.sequence() < msg2.sequence());
        assert!(msg2.sequence() < msg3.sequence());
    }
}
