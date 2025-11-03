use futures::Stream;
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::{Mutex, mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;

pub struct WrappedStreamUnboundedTokio<T> {
    sender: mpsc::UnboundedSender<T>,
    subscribers: Arc<Mutex<Vec<mpsc::UnboundedSender<T>>>>,
    stream: UnboundedReceiverStream<T>,
}

impl<T: Clone + Send + Debug + 'static> WrappedStreamUnboundedTokio<T> {
    pub async fn on_next(&self, item: T) {
        if self.sender.send(item.clone()).is_err() {
            eprintln!("No active subscribers to receive the item: {:?}", item);
        }

        let subscribers = self.subscribers.lock().await; // Async-friendly lock
        for subscriber in subscribers.iter() {
            if subscriber.send(item.clone()).is_err() {
                eprintln!(
                    "A subscriber has been dropped and cannot receive the item: {:?}",
                    item
                );
            }
        }
    }

    pub async fn copy(&mut self) -> UnboundedReceiverStream<T> {
        let (new_sender, new_receiver) = mpsc::unbounded_channel();
        let mut subscribers = self.subscribers.lock().await;
        subscribers.push(new_sender);
        UnboundedReceiverStream::new(new_receiver)
    }
}

impl<T: Clone + Send + Debug + 'static> Default for WrappedStreamUnboundedTokio<T> {
    fn default() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let receiver_stream = UnboundedReceiverStream::new(receiver);

        Self {
            sender,
            subscribers: Arc::new(Mutex::new(Vec::new())),
            stream: receiver_stream,
        }
    }
}

impl<T: Clone + Send + Debug + 'static> Stream for WrappedStreamUnboundedTokio<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().stream).poll_next(cx)
    }
}
