// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use async_trait::async_trait;
use futures::stream::Stream;
use futures::stream::StreamExt;
use std::future::Future;
use tokio_util::sync::CancellationToken;

/// Extension trait providing async subscription capabilities for streams.
///
/// This trait enables processing stream items with async handlers in a sequential manner.
#[async_trait]
pub trait SubscribeAsyncExt<T>: Stream<Item = T> + Sized {
    /// Subscribes to the stream with an async handler, processing items sequentially.
    ///
    /// This method consumes the stream and spawns async tasks to process each item.
    /// Items are processed in the order they arrive, with each item's handler running
    /// to completion before the next item is processed (though handlers run concurrently
    /// via tokio spawn).
    ///
    /// # Behavior
    ///
    /// - Processes each stream item with the provided async handler
    /// - Spawns a new task for each item (non-blocking)
    /// - Continues until stream ends or cancellation token is triggered
    /// - Errors from handlers are passed to the error callback
    /// - If no error callback provided, errors are logged
    ///
    /// # Arguments
    ///
    /// * `on_next_func` - Async function called for each stream item. Receives the item
    ///                    and a cancellation token. Returns `Result<(), E>`.
    /// * `cancellation_token` - Optional token to stop processing. If `None`, a default
    ///                          token is created that never cancels.
    /// * `on_error_callback` - Optional error handler called when `on_next_func` returns
    ///                         an error. If `None`, errors are logged.
    ///
    /// # Type Parameters
    ///
    /// * `F` - Function type for the item handler
    /// * `Fut` - Future type returned by the handler
    /// * `E` - Error type that implements `std::error::Error`
    /// * `OnError` - Function type for error handling
    ///
    /// # Thread Safety
    ///
    /// All spawned tasks run on the tokio runtime. The subscription completes
    /// when the stream ends, not when all spawned tasks complete.
    async fn subscribe_async<F, Fut, E, OnError>(
        self,
        on_next_func: F,
        cancellation_token: Option<CancellationToken>,
        on_error_callback: Option<OnError>,
    ) where
        F: Fn(T, CancellationToken) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Result<(), E>> + Send + 'static,
        OnError: Fn(E) + Clone + Send + Sync + 'static,
        T: std::fmt::Debug + Send + Clone + 'static,
        E: std::error::Error + Send + 'static;
}

#[async_trait]
impl<S, T> SubscribeAsyncExt<T> for S
where
    S: Stream<Item = T> + Send + Unpin + 'static,
    T: Send + 'static,
{
    async fn subscribe_async<F, Fut, E, OnError>(
        mut self,
        on_next_func: F,
        cancellation_token: Option<CancellationToken>,
        on_error_callback: Option<OnError>,
    ) where
        F: Fn(T, CancellationToken) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Result<(), E>> + Send + 'static,
        OnError: Fn(E) + Clone + Send + Sync + 'static,
        T: std::fmt::Debug + Send + Clone + 'static,
        E: std::error::Error + Send + 'static,
    {
        let cancellation_token = cancellation_token.unwrap_or_default();

        while let Some(item) = self.next().await {
            if cancellation_token.is_cancelled() {
                break;
            }

            let on_next_func = on_next_func.clone();
            let cancellation_token = cancellation_token.clone();
            let on_error_callback = on_error_callback.clone();

            tokio::spawn(async move {
                let result = on_next_func(item.clone(), cancellation_token).await;

                if let Err(error) = result {
                    if let Some(on_error_callback) = on_error_callback {
                        on_error_callback(error);
                    } else {
                        error!(
                            "Unhandled error in subscribe_async while processing item: {:?}, error: {}",
                            item, error
                        );
                    }
                }
            });
        }
    }
}
