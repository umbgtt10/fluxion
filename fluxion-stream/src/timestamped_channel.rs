//! Timestamped channels that automatically wrap values with temporal ordering.
//!
//! This module provides channel implementations that transparently add sequence numbers
//! (logical timestamps) to messages as they're sent, enabling temporal ordering when merging streams.

use crate::timestamped::Timestamped;
use tokio::sync::mpsc::{
    self, UnboundedReceiver as TokioUnboundedReceiver, UnboundedSender as TokioUnboundedSender,
};

/// An unbounded sender that automatically wraps values with timestamps.
///
/// Users send regular values of type `T`, but they're automatically wrapped
/// in `Timestamped<T>` with a sequence number assigned at send time.
#[derive(Debug)]
pub struct UnboundedSender<T> {
    inner: TokioUnboundedSender<Timestamped<T>>,
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
            .send(Timestamped::new(value))
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

/// An unbounded receiver that receives timestamped values.
///
/// Receives values wrapped in `Timestamped<T>` that were automatically
/// timestamped by the sender.
#[derive(Debug)]
pub struct UnboundedReceiver<T> {
    inner: TokioUnboundedReceiver<Timestamped<T>>,
}

impl<T> UnboundedReceiver<T> {
    /// Receives the next timestamped value.
    pub async fn recv(&mut self) -> Option<Timestamped<T>> {
        self.inner.recv().await
    }

    /// Attempts to receive without blocking.
    pub fn try_recv(&mut self) -> Result<Timestamped<T>, mpsc::error::TryRecvError> {
        self.inner.try_recv()
    }

    /// Polls to receive the next value.
    pub fn poll_recv(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Timestamped<T>>> {
        self.inner.poll_recv(cx)
    }

    /// Closes the receiving half of the channel.
    pub fn close(&mut self) {
        self.inner.close()
    }

    /// Converts this receiver into the underlying tokio receiver.
    ///
    /// This is useful for creating streams with `tokio_stream::wrappers::UnboundedReceiverStream`.
    pub fn into_inner(self) -> TokioUnboundedReceiver<Timestamped<T>> {
        self.inner
    }
}

/// Creates an unbounded timestamped channel.
///
/// The sender automatically wraps values with timestamps (sequence numbers),
/// and the receiver receives `Timestamped<T>` values.
///
/// # Examples
///
/// ```
/// use fluxion_stream::timestamped_channel::unbounded_channel;
/// use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
/// use fluxion_stream::select_all_ordered::SelectAllExt;
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
///     // Users just send regular values - timestamping is automatic!
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
