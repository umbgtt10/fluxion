// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use async_trait::async_trait;
use futures::stream::Stream;
use futures::stream::StreamExt;
use std::future::Future;
use tokio::sync::mpsc::unbounded_channel;
use tokio_util::sync::CancellationToken;

use fluxion_core::{FluxionError, Result};

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
    /// - Errors from handlers are passed to the error callback if provided
    /// - If no error callback provided, errors are collected and returned on completion
    ///
    /// # Arguments
    ///
    /// * `on_next_func` - Async function called for each stream item. Receives the item
    ///                    and a cancellation token. Returns `Result<(), E>`.
    /// * `cancellation_token` - Optional token to stop processing. If `None`, a default
    ///                          token is created that never cancels.
    /// * `on_error_callback` - Optional error handler called when `on_next_func` returns
    ///                         an error. If `None`, errors are collected and returned.
    ///
    /// # Type Parameters
    ///
    /// * `F` - Function type for the item handler
    /// * `Fut` - Future type returned by the handler
    /// * `E` - Error type that implements `std::error::Error`
    /// * `OnError` - Function type for error handling
    ///
    /// # Errors
    ///
    /// Returns `Err(FluxionError::MultipleErrors)` if any items failed to process and
    /// no error callback was provided. If an error callback is provided, errors are
    /// passed to it and the function returns `Ok(())` on stream completion.
    ///
    /// The subscription continues processing subsequent items even if individual items
    /// fail, unless the cancellation token is triggered.
    ///
    /// # See Also
    ///
    /// - [`subscribe_latest_async`](crate::SubscribeLatestAsyncExt::subscribe_latest_async) - Cancels old work for new items
    ///
    /// # Examples
    ///
    /// ```text
    /// use fluxion_exec::SubscribeAsyncExt;
    /// use futures::StreamExt;
    /// use tokio_stream::wrappers::UnboundedReceiverStream;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    /// let stream = UnboundedReceiverStream::new(rx);
    ///
    /// stream.subscribe_async(
    ///     |item, _token| async move {
    ///         // Process item
    ///         println!("Processing: {:?}", item);
    ///         Ok::<(), std::io::Error>(())
    ///     },
    ///     None,
    ///     Some(|err| eprintln!("Error: {}", err))
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # With Cancellation
    ///
    /// ```text
    /// # use fluxion_exec::SubscribeAsyncExt;
    /// # use futures::StreamExt;
    /// # use tokio_util::sync::CancellationToken;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let stream = futures::stream::iter(vec![1, 2, 3]);
    /// let cancel = CancellationToken::new();
    /// let cancel_clone = cancel.clone();
    ///
    /// tokio::spawn(async move {
    ///     tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    ///     cancel_clone.cancel();
    /// });
    ///
    /// stream.subscribe_async(
    ///     |item, token| async move {
    ///         if token.is_cancelled() {
    ///             return Ok(());
    ///         }
    ///         // Process item...
    ///         Ok::<(), std::io::Error>(())
    ///     },
    ///     Some(cancel),
    ///     None
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
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
    ) -> Result<()>
    where
        F: Fn(T, CancellationToken) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = std::result::Result<(), E>> + Send + 'static,
        OnError: Fn(E) + Clone + Send + Sync + 'static,
        T: std::fmt::Debug + Send + Clone + 'static,
        E: std::error::Error + Send + Sync + 'static;
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
    ) -> Result<()>
    where
        F: Fn(T, CancellationToken) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = std::result::Result<(), E>> + Send + 'static,
        OnError: Fn(E) + Clone + Send + Sync + 'static,
        T: std::fmt::Debug + Send + Clone + 'static,
        E: std::error::Error + Send + Sync + 'static,
    {
        let cancellation_token = cancellation_token.unwrap_or_default();
        let (error_tx, mut error_rx) = unbounded_channel();

        while let Some(item) = self.next().await {
            if cancellation_token.is_cancelled() {
                break;
            }

            let on_next_func = on_next_func.clone();
            let cancellation_token = cancellation_token.clone();
            let on_error_callback = on_error_callback.clone();
            let error_tx = error_tx.clone();

            tokio::spawn(async move {
                let result = on_next_func(item.clone(), cancellation_token).await;

                if let Err(error) = result {
                    if let Some(on_error_callback) = on_error_callback {
                        on_error_callback(error);
                    } else {
                        // Collect error for later aggregation
                        let _ = error_tx.send(error);
                    }
                }
            });
        }

        // Drop the original sender so the channel closes
        drop(error_tx);

        // Collect all errors from the channel
        let mut collected_errors = Vec::new();
        while let Some(error) = error_rx.recv().await {
            collected_errors.push(error);
        }

        if !collected_errors.is_empty() {
            Err(FluxionError::from_user_errors(collected_errors))
        } else {
            Ok(())
        }
    }
}
