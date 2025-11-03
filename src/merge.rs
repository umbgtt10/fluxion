use futures::stream::FuturesUnordered;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tokio_stream::{Stream, wrappers::UnboundedReceiverStream};

pub trait MergeStreamsExt<T>
where
    T: Send + 'static,
{
    fn merge_sequentially(self) -> UnboundedReceiverStream<T>;
}

impl<T, S> MergeStreamsExt<T> for Vec<S>
where
    T: Send + 'static + std::fmt::Debug,
    S: Stream<Item = T> + Send + 'static + Unpin,
{
    fn merge_sequentially(self) -> UnboundedReceiverStream<T> {
        let (tx, rx) = mpsc::unbounded_channel::<T>();
        let mut futures_unordered = FuturesUnordered::new();

        // Add each stream as a future into `FuturesUnordered`
        for mut stream in self {
            let tx = tx.clone();

            futures_unordered.push(async move {
                // Process all items from a stream
                while let Some(item) = stream.next().await {
                    // Send item to the shared channel
                    println!("Merging item: {:?}", item);
                    if tx.send(item).is_err() {
                        break; // Stop if the receiver is dropped
                    }
                }
            });
        }

        // Spawn a single task to drive `FuturesUnordered`
        tokio::spawn(async move {
            while futures_unordered.next().await.is_some() {
                // Drain futures concurrently; each stream emits items into the receiver
            }
        });

        UnboundedReceiverStream::new(rx)
    }
}
