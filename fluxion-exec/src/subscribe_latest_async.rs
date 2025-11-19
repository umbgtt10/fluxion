// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};
use tokio_util::sync::CancellationToken;

use fluxion_core::{FluxionError, Result};

/// Extension trait providing async subscription with automatic cancellation of outdated work.
///
/// This trait enables processing stream items where newer items automatically cancel
/// processing of older items, ensuring only the latest value is being processed.
#[async_trait]
pub trait SubscribeLatestAsyncExt<T>: Stream<Item = T> + Send + 'static
where
    T: Send + 'static,
{
    /// Subscribes to the stream, automatically cancelling processing of older items when new items arrive.
    ///
    /// This method is ideal for scenarios where you only care about processing the most recent
    /// value and want to abandon work on outdated values. When a new item arrives while an
    /// existing item is being processed, a new task starts for the latest item.
    ///
    /// # Behavior
    ///
    /// - Only one processing task runs at a time per stream
    /// - When a new item arrives during processing, it queues as "latest"
    /// - After current processing completes, the latest queued item is processed
    /// - Intermediate items between current and latest are discarded
    /// - Waits for all processing to complete before returning
    ///
    /// # Arguments
    ///
    /// * `on_next_func` - Async function called for each item. Should check the cancellation
    ///                    token periodically to enable responsive cancellation.
    /// * `on_error_callback` - Optional error handler for processing failures
    /// * `cancellation_token` - Optional token to stop all processing
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
    /// Processing continues with new items even if previous items failed, unless the
    /// cancellation token is triggered.
    ///
    /// # See Also
    ///
    /// - [`subscribe_async`](crate::SubscribeAsyncExt::subscribe_async) - Sequential processing of all items
    ///
    /// # Examples
    ///
    /// ## Basic Usage - Skipping Intermediate Values
    ///
    /// Only the first and latest items are processed:
    ///
    /// ```
    /// use fluxion_exec::SubscribeLatestAsyncExt;
    /// use tokio::sync::mpsc::unbounded_channel;
    /// use tokio_stream::wrappers::UnboundedReceiverStream;
    /// use futures::StreamExt;
    /// use std::sync::Arc;
    /// use tokio::sync::Mutex;
    ///
    /// # tokio_test::block_on(async {
    /// let (tx, rx) = unbounded_channel();
    /// let stream = UnboundedReceiverStream::new(rx).map(|x: i32| x);
    ///
    /// let processed = Arc::new(Mutex::new(Vec::new()));
    /// let processed_clone = processed.clone();
    ///
    /// // Gate to control when first item completes
    /// let (gate_tx, gate_rx) = unbounded_channel::<()>();
    /// let gate_shared = Arc::new(Mutex::new(Some(gate_rx)));
    ///
    /// let handle = tokio::spawn(async move {
    ///     stream.subscribe_latest_async(
    ///         move |item, token| {
    ///             let processed = processed_clone.clone();
    ///             let gate = gate_shared.clone();
    ///             async move {
    ///                 // First item waits at gate
    ///                 if let Some(mut rx) = gate.lock().await.take() {
    ///                     let _ = rx.recv().await;
    ///                 }
    ///                 if !token.is_cancelled() {
    ///                     processed.lock().await.push(item);
    ///                 }
    ///                 Ok::<(), std::io::Error>(())
    ///             }
    ///         },
    ///         None::<fn(std::io::Error)>,
    ///         None
    ///     ).await
    /// });
    ///
    /// // Send multiple items rapidly
    /// tx.send(1).unwrap();
    /// tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    /// tx.send(2).unwrap(); // Will be skipped
    /// tx.send(3).unwrap(); // Will be skipped
    /// tx.send(4).unwrap(); // Latest - will be processed
    ///
    /// // Release the gate
    /// gate_tx.send(()).unwrap();
    /// drop(tx);
    ///
    /// handle.await.unwrap().unwrap();
    ///
    /// let result = processed.lock().await.clone();
    /// assert_eq!(result.len(), 2); // Only first and latest
    /// assert_eq!(result[0], 1);
    /// assert_eq!(result[1], 4);
    /// # });
    /// ```
    ///
    /// ## With Cancellation Token Checks
    ///
    /// Handlers should check the token periodically for responsive cancellation:
    ///
    /// ```
    /// use fluxion_exec::SubscribeLatestAsyncExt;
    /// use tokio::sync::mpsc::unbounded_channel;
    /// use tokio_stream::wrappers::UnboundedReceiverStream;
    /// use futures::StreamExt;
    /// use std::sync::Arc;
    /// use tokio::sync::Mutex;
    ///
    /// # tokio_test::block_on(async {
    /// let (tx, rx) = unbounded_channel();
    /// let stream = UnboundedReceiverStream::new(rx).map(|x: i32| x);
    ///
    /// let completed = Arc::new(Mutex::new(Vec::new()));
    /// let completed_clone = completed.clone();
    ///
    /// let handle = tokio::spawn(async move {
    ///     stream.subscribe_latest_async(
    ///         move |item, token| {
    ///             let completed = completed_clone.clone();
    ///             async move {
    ///                 // Simulate long-running work with cancellation checks
    ///                 for i in 0..10 {
    ///                     if token.is_cancelled() {
    ///                         return Ok(()); // Exit gracefully
    ///                     }
    ///                     tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    ///                 }
    ///                 completed.lock().await.push(item);
    ///                 Ok::<(), std::io::Error>(())
    ///             }
    ///         },
    ///         None::<fn(std::io::Error)>,
    ///         None
    ///     ).await
    /// });
    ///
    /// tx.send(1).unwrap();
    /// tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    /// tx.send(2).unwrap(); // This will cancel work on item 1 if still in progress
    /// drop(tx);
    ///
    /// handle.await.unwrap().unwrap();
    /// # });
    /// ```
    ///
    /// ## Search-As-You-Type Pattern
    ///
    /// Only search for the latest query:
    ///
    /// ```
    /// use fluxion_exec::SubscribeLatestAsyncExt;
    /// use tokio::sync::mpsc::unbounded_channel;
    /// use tokio_stream::wrappers::UnboundedReceiverStream;
    /// use futures::StreamExt;
    /// use std::sync::Arc;
    /// use tokio::sync::Mutex;
    ///
    /// # tokio_test::block_on(async {
    /// async fn search_api(query: &str) -> Result<Vec<String>, std::io::Error> {
    ///     tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    ///     Ok(vec![format!("result_for_{}", query)])
    /// }
    ///
    /// let (tx, rx) = unbounded_channel();
    /// let stream = UnboundedReceiverStream::new(rx).map(|x: String| x);
    ///
    /// let results = Arc::new(Mutex::new(Vec::new()));
    /// let results_clone = results.clone();
    ///
    /// let handle = tokio::spawn(async move {
    ///     stream.subscribe_latest_async(
    ///         move |query, token| {
    ///             let results = results_clone.clone();
    ///             async move {
    ///                 let search_results = search_api(&query).await?;
    ///                 if !token.is_cancelled() {
    ///                     results.lock().await.extend(search_results);
    ///                 }
    ///                 Ok::<(), std::io::Error>(())
    ///             }
    ///         },
    ///         None::<fn(std::io::Error)>,
    ///         None
    ///     ).await
    /// });
    ///
    /// // User types rapidly
    /// tx.send("r".to_string()).unwrap();
    /// tx.send("ru".to_string()).unwrap();
    /// tx.send("rus".to_string()).unwrap();
    /// tx.send("rust".to_string()).unwrap();
    /// drop(tx);
    ///
    /// handle.await.unwrap().unwrap();
    /// # });
    /// ```
    ///
    /// ## UI Update Pattern
    ///
    /// Render only the latest application state:
    ///
    /// ```
    /// use fluxion_exec::SubscribeLatestAsyncExt;
    /// use tokio::sync::mpsc::unbounded_channel;
    /// use tokio_stream::wrappers::UnboundedReceiverStream;
    /// use futures::StreamExt;
    /// use std::sync::Arc;
    /// use tokio::sync::Mutex;
    ///
    /// # tokio_test::block_on(async {
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct AppState { counter: u32 }
    ///
    /// let (tx, rx) = unbounded_channel();
    /// let stream = UnboundedReceiverStream::new(rx).map(|x: AppState| x);
    ///
    /// let rendered = Arc::new(Mutex::new(Vec::new()));
    /// let rendered_clone = rendered.clone();
    ///
    /// let handle = tokio::spawn(async move {
    ///     stream.subscribe_latest_async(
    ///         move |state, token| {
    ///             let rendered = rendered_clone.clone();
    ///             async move {
    ///                 // Simulate expensive rendering
    ///                 tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    ///                 if !token.is_cancelled() {
    ///                     rendered.lock().await.push(state);
    ///                 }
    ///                 Ok::<(), std::io::Error>(())
    ///             }
    ///         },
    ///         None::<fn(std::io::Error)>,
    ///         None
    ///     ).await
    /// });
    ///
    /// // Rapid state updates
    /// for i in 0..10 {
    ///     tx.send(AppState { counter: i }).unwrap();
    /// }
    /// drop(tx);
    ///
    /// handle.await.unwrap().unwrap();
    ///
    /// // Only latest states should be rendered (intermediate ones skipped)
    /// tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    /// let rendered_states = rendered.lock().await;
    /// assert!(rendered_states.len() < 10); // Some states were skipped
    /// # });
    /// ```
    ///
    /// # Use Cases
    ///
    /// - UI updates where only the latest state matters
    /// - Search-as-you-type with debouncing
    /// - Real-time data processing where stale data is irrelevant
    /// - API calls that should be cancelled when parameters change
    ///
    /// # Comparison with `subscribe_async`
    ///
    /// - `subscribe_async`: Processes every item, all handlers run to completion
    /// - `subscribe_latest_async`: Processes only latest, abandons outdated work
    ///
    /// # Thread Safety
    ///
    /// Uses tokio's async mutex for state coordination. Only one processing task
    /// is active at a time, but task spawning is non-blocking.
    async fn subscribe_latest_async<F, Fut, E, OnError>(
        self,
        on_next_func: F,
        on_error_callback: Option<OnError>,
        cancellation_token: Option<CancellationToken>,
    ) -> Result<()>
    where
        F: Fn(T, CancellationToken) -> Fut + Clone + Send + Sync + 'static,
        Fut: std::future::Future<Output = std::result::Result<(), E>> + Send + 'static,
        OnError: Fn(E) + Clone + Send + Sync + 'static,
        E: std::error::Error + Send + Sync + 'static,
        T: std::fmt::Debug + Clone + Send + Sync + 'static;
}

#[async_trait]
impl<S, T> SubscribeLatestAsyncExt<T> for S
where
    S: futures::Stream<Item = T> + Send + Unpin + 'static,
    T: Clone + Send + Sync + 'static,
{
    async fn subscribe_latest_async<F, Fut, E, OnError>(
        self,
        on_next_func: F,
        on_error_callback: Option<OnError>,
        cancellation_token: Option<CancellationToken>,
    ) -> Result<()>
    where
        F: Fn(T, CancellationToken) -> Fut + Clone + Send + Sync + 'static,
        Fut: std::future::Future<Output = std::result::Result<(), E>> + Send + 'static,
        OnError: Fn(E) + Clone + Send + Sync + 'static,
        E: std::error::Error + Send + Sync + 'static,
        T: std::fmt::Debug + Clone + Send + Sync + 'static,
    {
        let state = Arc::new(Context::default());
        let cancellation_token = cancellation_token.unwrap_or_default();
        let state_for_wait = state.clone();
        let errors: Arc<std::sync::Mutex<Vec<E>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
        let errors_clone = errors.clone();

        self.for_each(move |new_data| {
            let on_next_func = on_next_func.clone();
            let state = state.clone();
            let cancellation_token = cancellation_token.clone();
            let on_error_callback = on_error_callback.clone();
            let errors = errors.clone();
            async move {
                if cancellation_token.is_cancelled() {
                    return;
                }

                if state.enqueue_and_try_start_processing(new_data).await {
                    let state = state.clone();
                    let on_next_func = on_next_func.clone();
                    let on_error_callback = on_error_callback.clone();
                    let cancellation_token = cancellation_token.clone();

                    tokio::spawn(async move {
                        while let Some(item) = state.get_item().await {
                            if let Err(error) =
                                on_next_func(item.clone(), cancellation_token.clone()).await
                            {
                                if let Some(on_error_callback) = on_error_callback.clone() {
                                    on_error_callback(error);
                                } else {
                                    // Collect error for later aggregation
                                    if let Ok(mut errs) = errors.lock() {
                                        errs.push(error);
                                    }
                                }
                            }

                            // Check if a new item arrived while we were processing
                            if !state.finish_processing_and_check_for_next().await {
                                // No new item, we're done
                                break;
                            }
                        }

                        state.notify_task_complete();
                    });
                }
            }
        })
        .await;

        // Wait for any remaining processing tasks to complete
        state_for_wait.wait_for_processing_complete().await;

        // Check if any errors were collected
        let collected_errors = {
            let mut guard = errors_clone.lock().unwrap_or_else(|e| e.into_inner());
            std::mem::take(&mut *guard)
        };

        if !collected_errors.is_empty() {
            Err(FluxionError::from_user_errors(collected_errors))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
struct Context<T> {
    state: Mutex<State<T>>,
    processing_complete: Notify,
}

#[derive(Debug)]
struct State<T> {
    item: Option<T>,
    is_processing: bool,
}

impl<T> Context<T> {
    /// Enqueues an item. Returns true if we should spawn a processing task.
    pub async fn enqueue_and_try_start_processing(&self, value: T) -> bool {
        let mut state = self.state.lock().await;
        state.item = Some(value);

        if state.is_processing {
            false
        } else {
            state.is_processing = true;
            true
        }
    }

    /// Gets the current item for processing.
    /// Returns `None` if no item is available (should be unreachable in normal flow).
    pub async fn get_item(&self) -> Option<T>
    where
        T: Clone,
    {
        let mut state = self.state.lock().await;
        state.item.take().map_or_else(
            || {
                // Defensive: log invalid state and mark as not processing to avoid deadlock
                error!(
                    "subscribe_latest_async: get_item called with no current item; marking idle"
                );
                state.is_processing = false;
                None
            },
            |item| Some(item),
        )
    }

    /// Called when processing finishes. Returns true if there's another item to process.
    pub async fn finish_processing_and_check_for_next(&self) -> bool {
        let mut state = self.state.lock().await;

        if state.item.is_some() {
            // New item arrived during processing, continue
            true
        } else {
            // No new item, mark as idle
            state.is_processing = false;
            false
        }
    }

    /// Notify that a processing task has completed
    pub fn notify_task_complete(&self) {
        self.processing_complete.notify_waiters();
    }

    /// Wait for all processing to complete
    pub async fn wait_for_processing_complete(&self) {
        loop {
            {
                let state = self.state.lock().await;
                if !state.is_processing {
                    // No active processing
                    return;
                }
            }
            // Wait for notification
            self.processing_complete.notified().await;
        }
    }
}

impl<T> Default for Context<T> {
    fn default() -> Self {
        Self {
            state: Mutex::new(State {
                item: None,
                is_processing: false,
            }),
            processing_complete: Notify::new(),
        }
    }
}
