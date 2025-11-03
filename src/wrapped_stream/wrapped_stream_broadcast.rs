use futures::Stream;
use std::fmt::Debug;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;

const BROADCASTCHANNELCAPACITY: usize = 100;

pub struct WrappedStreamBroadcast<T> {
    sender: broadcast::Sender<T>,
    stream: BroadcastStream<T>,
}

impl<T: Clone + Send + Debug + 'static> WrappedStreamBroadcast<T> {
    pub fn on_next(&self, item: T) {
        if self.sender.send(item.clone()).is_err() {
            eprintln!("No active subscribers to receive the item: {:?}", item);
        }
    }

    pub fn copy(&self) -> WrappedStreamBroadcast<T> {
        Self {
            sender: self.sender.clone(),
            stream: BroadcastStream::new(self.sender.subscribe()),
        }
    }
}

impl<T: Clone + Send + Debug + 'static> Default for WrappedStreamBroadcast<T> {
    fn default() -> Self {
        let (sender, _) = broadcast::channel(BROADCASTCHANNELCAPACITY);
        Self {
            sender: sender.clone(),
            stream: BroadcastStream::new(sender.subscribe()),
        }
    }
}

impl<T: Clone + Send + Debug + 'static> Stream for WrappedStreamBroadcast<T> {
    type Item = Result<T, RecvError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.stream).poll_next(cx).map(|option| {
            option.map(|result| {
                result.map_err(|e| match e {
                    BroadcastStreamRecvError::Lagged(n) => RecvError::Lagged(n),
                })
            })
        })
    }
}
