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
    /// ## Basic Usage
    ///
    /// Process all items sequentially:
    ///
    /// ```
    /// use fluxion_exec::SubscribeAsyncExt;
    /// use futures::stream;
    /// use std::sync::Arc;
    /// use tokio::sync::Mutex;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let results = Arc::new(Mutex::new(Vec::new()));
    /// let results_clone = results.clone();
    ///
    /// let stream = stream::iter(vec![1, 2, 3, 4, 5]);
    ///
    /// // Subscribe and process each item
    /// stream.subscribe_async(
    ///     move |item, _token| {
    ///         let results = results_clone.clone();
    ///         async move {
    ///             results.lock().await.push(item * 2);
    ///             Ok::<(), std::io::Error>(())
    ///         }
    ///     },
    ///     None, // No cancellation
    ///     None::<fn(std::io::Error)>  // No error callback
    /// ).await.unwrap();
    ///
    /// // Wait a bit for spawned tasks to complete
    /// tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    ///
    /// let processed = results.lock().await;
    /// assert!(processed.contains(&2));
    /// assert!(processed.contains(&4));
    /// # }
    /// ```
    ///
    /// ## With Error Handling
    ///
    /// Use an error callback to handle errors without stopping the stream:
    ///
    /// ```
    /// use fluxion_exec::SubscribeAsyncExt;
    /// use futures::stream;
    /// use std::sync::Arc;
    /// use tokio::sync::Mutex;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #[derive(Debug)]
    /// struct MyError(String);
    /// impl std::fmt::Display for MyError {
    ///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    ///         write!(f, "MyError: {}", self.0)
    ///     }
    /// }
    /// impl std::error::Error for MyError {}
    ///
    /// let error_count = Arc::new(Mutex::new(0));
    /// let error_count_clone = error_count.clone();
    ///
    /// let stream = stream::iter(vec![1, 2, 3, 4, 5]);
    ///
    /// stream.subscribe_async(
    ///     |item, _token| async move {
    ///         if item % 2 == 0 {
    ///             Err(MyError(format!("Even number: {}", item)))
    ///         } else {
    ///             Ok(())
    ///         }
    ///     },
    ///     None,
    ///     Some(move |_err| {
    ///         let count = error_count_clone.clone();
    ///         tokio::spawn(async move {
    ///             *count.lock().await += 1;
    ///         });
    ///     })
    /// ).await.unwrap();
    ///
    /// tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    /// assert_eq!(*error_count.lock().await, 2); // Items 2 and 4 errored
    /// # }
    /// ```
    ///
    /// ## With Cancellation
    ///
    /// Use a cancellation token to stop processing:
    ///
    /// ```
    /// use fluxion_exec::SubscribeAsyncExt;
    /// use tokio::sync::mpsc::unbounded_channel;
    /// use tokio_stream::wrappers::UnboundedReceiverStream;
    /// use futures::StreamExt;
    /// use tokio_util::sync::CancellationToken;
    /// use std::sync::Arc;
    /// use tokio::sync::Mutex;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let (tx, rx) = unbounded_channel();
    /// let stream = UnboundedReceiverStream::new(rx);
    ///
    /// let cancel_token = CancellationToken::new();
    /// let cancel_clone = cancel_token.clone();
    ///
    /// let processed = Arc::new(Mutex::new(Vec::new()));
    /// let processed_clone = processed.clone();
    ///
    /// let handle = tokio::spawn(async move {
    ///     stream.subscribe_async(
    ///         move |item, token| {
    ///             let vec = processed_clone.clone();
    ///             async move {
    ///                 if token.is_cancelled() {
    ///                     return Ok(());
    ///                 }
    ///                 tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    ///                 vec.lock().await.push(item);
    ///                 Ok::<(), std::io::Error>(())
    ///             }
    ///         },
    ///         Some(cancel_token),
    ///         None::<fn(std::io::Error)>
    ///     ).await
    /// });
    ///
    /// // Send a few items
    /// tx.send(1).unwrap();
    /// tx.send(2).unwrap();
    /// tx.send(3).unwrap();
    ///
    /// // Wait a bit then cancel
    /// tokio::time::sleep(tokio::time::Duration::from_millis(15)).await;
    /// cancel_clone.cancel();
    /// drop(tx);
    ///
    /// handle.await.unwrap().unwrap();
    ///
    /// // At least one item should be processed before cancellation
    /// assert!(!processed.lock().await.is_empty());
    /// # }
    /// ```
    ///
    /// ## Database Write Pattern
    ///
    /// Process events and persist to a database:
    ///
    /// ```
    /// use fluxion_exec::SubscribeAsyncExt;
    /// use futures::stream;
    /// use std::sync::Arc;
    /// use tokio::sync::Mutex;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #[derive(Clone, Debug)]
    /// struct Event { id: u32, data: String }
    ///
    /// // Simulated database
    /// let db = Arc::new(Mutex::new(Vec::new()));
    /// let db_clone = db.clone();
    ///
    /// let events = vec![
    ///     Event { id: 1, data: "event1".to_string() },
    ///     Event { id: 2, data: "event2".to_string() },
    /// ];
    ///
    /// let stream = stream::iter(events);
    ///
    /// stream.subscribe_async(
    ///     move |event, _token| {
    ///         let db = db_clone.clone();
    ///         async move {
    ///             // Simulate database write
    ///             tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    ///             db.lock().await.push(event);
    ///             Ok::<(), std::io::Error>(())
    ///         }
    ///     },
    ///     None,
    ///     Some(|err| eprintln!("DB Error: {}", err))
    /// ).await.unwrap();
    ///
    /// tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    /// assert_eq!(db.lock().await.len(), 2);
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
