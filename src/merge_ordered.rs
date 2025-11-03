use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::{mpsc, oneshot};
use tokio_stream::StreamExt;
use tokio_stream::StreamMap;
use tokio_stream::wrappers::UnboundedReceiverStream;

pub trait VecStreamMergeExt<S> {
    fn merge_ordered<T>(self) -> impl Future<Output = MergedOrderedStream<T>>
    where
        S: Stream<Item = T> + Unpin + Send + 'static,
        T: Send + 'static;
}

impl<S> VecStreamMergeExt<S> for Vec<S>
where
    S: Stream + Send + 'static,
{
    async fn merge_ordered<T>(self) -> MergedOrderedStream<T>
    where
        S: Stream<Item = T> + Unpin + Send + 'static,
        T: Send + 'static,
    {
        MergedOrderedStream::new(self).await
    }
}

pub struct MergedOrderedStream<T> {
    receiver: UnboundedReceiverStream<T>,
    _handle: tokio::task::JoinHandle<()>,
}

impl<T> MergedOrderedStream<T>
where
    T: Send + 'static,
{
    async fn new<S>(streams: Vec<S>) -> Self
    where
        S: Stream<Item = T> + Send + Unpin + 'static,
    {
        let (tx, rx) = mpsc::unbounded_channel();
        let (ready_tx, ready_rx) = oneshot::channel();
        let tx_clone = tx.clone();

        let handle = tokio::spawn(async move {
            let mut stream_map = StreamMap::new();

            for (index, stream) in streams.into_iter().enumerate() {
                stream_map.insert(index, stream);
            }

            let _ = ready_tx.send(());

            while let Some((_stream_id, item)) = stream_map.next().await {
                if tx.send(item).is_err() {
                    break;
                }
            }
        });

        let _ = ready_rx.await;

        drop(tx_clone);

        Self {
            receiver: UnboundedReceiverStream::new(rx),
            _handle: handle,
        }
    }
}

impl<T> Stream for MergedOrderedStream<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.receiver).poll_next(cx)
    }
}
