// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::boxed::Box;
use alloc::vec::Vec;
use async_trait::async_trait;
use core::error::Error;
use core::fmt::Debug;
use core::future::Future;
use fluxion_core::{CancellationToken, FluxionError, Result};
use futures::channel::mpsc::unbounded;
use futures::stream::Stream;
use futures::stream::StreamExt;

/// Extension trait providing async subscription capabilities for streams.
///
/// This trait enables processing stream items with async handlers in a sequential manner.
#[async_trait]
pub trait SubscribeExt<T>: Stream<Item = T> + Sized {
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
    /// - [`subscribe_latest`](crate::SubscribeLatestExt::subscribe_latest) - Cancels old work for new items
    ///
    /// # Examples
    ///
    /// ## Basic Usage
    ///
    /// Process all items sequentially:
    ///
    /// ```
    /// use fluxion_exec::SubscribeExt;
    /// use futures::channel::mpsc::unbounded;
    /// use futures::stream;
    /// use futures::StreamExt;
    /// use std::sync::Arc;
    /// use futures::lock::Mutex;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let results = Arc::new(Mutex::new(Vec::new()));
    /// let results_clone = results.clone();
    /// let (notify_tx, mut notify_rx) = unbounded();
    ///
    /// let stream = stream::iter(vec![1, 2, 3, 4, 5]);
    ///
    /// // Subscribe and process each item
    /// stream.subscribe(
    ///     move |item, _token| {
    ///         let results = results_clone.clone();
    ///         let notify_tx = notify_tx.clone();
    ///         async move {
    ///             results.lock().await.push(item * 2);
    ///             let _ = notify_tx.unbounded_send(());
    ///             Ok::<(), std::io::Error>(())
    ///         }
    ///     },
    ///     None, // No cancellation
    ///     None::<fn(std::io::Error)>  // No error callback
    /// ).await.unwrap();
    ///
    /// // Wait for all 5 items to be processed
    /// for _ in 0..5 {
    ///     notify_rx.next().await.unwrap();
    /// }
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
    /// use fluxion_exec::SubscribeExt;
    /// use futures::channel::mpsc::unbounded;
    /// use futures::stream;
    /// use futures::StreamExt;
    /// use std::sync::Arc;
    /// use futures::lock::Mutex;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #[derive(Debug)]
    /// struct MyError(String);
    /// impl core::fmt::Display for MyError {
    ///     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    ///         write!(f, "MyError: {}", self.0)
    ///     }
    /// }
    /// impl std::error::Error for MyError {}
    ///
    /// let error_count = Arc::new(Mutex::new(0));
    /// let error_count_clone = error_count.clone();
    /// let (notify_tx, mut notify_rx) = unbounded();
    ///
    /// let stream = stream::iter(vec![1, 2, 3, 4, 5]);
    ///
    /// stream.subscribe(
    ///     move |item, _token| {
    ///         let notify_tx = notify_tx.clone();
    ///         async move {
    ///             let res = if item % 2 == 0 {
    ///                 Err(MyError(format!("Even number: {}", item)))
    ///             } else {
    ///                 Ok(())
    ///             };
    ///             // Signal completion regardless of success/failure
    ///             // Note: In real code, you might signal in the error callback too
    ///             // but here we just want to know the handler finished.
    ///             // However, subscribe spawns the handler. If it errors,
    ///             // the error callback is called.
    ///             // We need to signal completion in both paths.
    ///             // Since the handler returns the error, we can't signal *after* returning Err.
    ///             // So we signal before returning.
    ///             let _ = notify_tx.unbounded_send(());
    ///             res
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
    /// // Wait for 5 items
    /// for _ in 0..5 {
    ///     notify_rx.next().await.unwrap();
    /// }
    ///
    /// // Give a tiny bit of time for the error callback spawn to finish updating the count
    /// // (Since the callback spawns another task)
    /// // Alternatively, we could use a channel in the error callback too.
    /// tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    ///
    /// assert_eq!(*error_count.lock().await, 2); // Items 2 and 4 errored
    /// # }
    /// ```
    ///
    /// ## With Cancellation
    ///
    /// Use a cancellation token to stop processing:
    ///
    /// ```
    /// use fluxion_exec::SubscribeExt;
    /// use futures::channel::mpsc::unbounded;
    /// use futures::StreamExt;
    /// use fluxion_core::CancellationToken;
    /// use std::sync::Arc;
    /// use futures::lock::Mutex;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let (tx, rx) = unbounded();
    /// let stream = rx;
    ///
    /// let cancel_token = CancellationToken::new();
    /// let cancel_clone = cancel_token.clone();
    ///
    /// let processed = Arc::new(Mutex::new(Vec::new()));
    /// let processed_clone = processed.clone();
    /// let (notify_tx, mut notify_rx) = unbounded();
    ///
    /// let handle = tokio::spawn(async move {
    ///     stream.subscribe(
    ///         move |item, token| {
    ///             let vec = processed_clone.clone();
    ///             let notify_tx = notify_tx.clone();
    ///             async move {
    ///                 if token.is_cancelled() {
    ///                     return Ok(());
    ///                 }
    ///                 vec.lock().await.push(item);
    ///                 let _ = notify_tx.unbounded_send(());
    ///                 Ok::<(), std::io::Error>(())
    ///             }
    ///         },
    ///         Some(cancel_token),
    ///         None::<fn(std::io::Error)>
    ///     ).await
    /// });
    ///
    /// // Send items
    /// tx.unbounded_send(1).unwrap();
    /// tx.unbounded_send(2).unwrap();
    /// tx.unbounded_send(3).unwrap();
    ///
    /// // Wait for first item to be processed
    /// notify_rx.next().await.unwrap();
    ///
    /// // Cancel now
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
    /// use fluxion_exec::SubscribeExt;
    /// use futures::channel::mpsc::unbounded;
    /// use futures::stream;
    /// use futures::StreamExt;
    /// use std::sync::Arc;
    /// use futures::lock::Mutex;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// #[derive(Clone, Debug)]
    /// struct Event { id: u32, data: String }
    ///
    /// // Simulated database
    /// let db = Arc::new(Mutex::new(Vec::new()));
    /// let db_clone = db.clone();
    /// let (notify_tx, mut notify_rx) = unbounded();
    ///
    /// let events = vec![
    ///     Event { id: 1, data: "event1".to_string() },
    ///     Event { id: 2, data: "event2".to_string() },
    /// ];
    ///
    /// let stream = stream::iter(events);
    ///
    /// stream.subscribe(
    ///     move |event, _token| {
    ///         let db = db_clone.clone();
    ///         let notify_tx = notify_tx.clone();
    ///         async move {
    ///             // Simulate database write
    ///             db.lock().await.push(event);
    ///             let _ = notify_tx.unbounded_send(());
    ///             Ok::<(), std::io::Error>(())
    ///         }
    ///     },
    ///     None,
    ///     Some(|err| eprintln!("DB Error: {}", err))
    /// ).await.unwrap();
    ///
    /// // Wait for 2 events
    /// notify_rx.next().await.unwrap();
    /// notify_rx.next().await.unwrap();
    ///
    /// assert_eq!(db.lock().await.len(), 2);
    /// # }
    /// ```
    ///
    /// # Thread Safety
    ///
    /// All spawned tasks run on the tokio runtime. The subscription completes
    /// when the stream ends, not when all spawned tasks complete.
    async fn subscribe<F, Fut, E, OnError>(
        self,
        on_next_func: F,
        cancellation_token: Option<CancellationToken>,
        on_error_callback: Option<OnError>,
    ) -> Result<()>
    where
        F: Fn(T, CancellationToken) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = core::result::Result<(), E>> + Send + 'static,
        OnError: Fn(E) + Clone + Send + Sync + 'static,
        T: Debug + Send + Clone + 'static,
        E: Error + Send + Sync + 'static;
}

#[async_trait]
impl<S, T> SubscribeExt<T> for S
where
    S: Stream<Item = T> + Send + Unpin + 'static,
    T: Send + 'static,
{
    async fn subscribe<F, Fut, E, OnError>(
        mut self,
        on_next_func: F,
        cancellation_token: Option<CancellationToken>,
        on_error_callback: Option<OnError>,
    ) -> Result<()>
    where
        F: Fn(T, CancellationToken) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = core::result::Result<(), E>> + Send + 'static,
        OnError: Fn(E) + Clone + Send + Sync + 'static,
        T: Debug + Send + Clone + 'static,
        E: Error + Send + Sync + 'static,
    {
        let cancellation_token = cancellation_token.unwrap_or_default();
        let (error_tx, mut error_rx) = unbounded();

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
                        let _ = error_tx.unbounded_send(error);
                    }
                }
            });
        }

        // Drop the original sender so the channel closes
        drop(error_tx);

        // Collect all errors from the channel
        let mut collected_errors = Vec::new();
        while let Some(error) = error_rx.next().await {
            collected_errors.push(error);
        }

        if !collected_errors.is_empty() {
            Err(FluxionError::from_user_errors(collected_errors))
        } else {
            Ok(())
        }
    }
}
